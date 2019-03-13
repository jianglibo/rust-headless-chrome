use futures::sync::mpsc as future_mpsc;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use tokio::prelude::future::loop_fn;
use tokio::prelude::IntoFuture;
use tokio::runtime::Runtime;

use failure;
use log::*;

use crate::protocol::page::methods::Navigate;
use serde;

use crate::protocol;
// use crate::protocol::{Message, Event, Method};
use crate::protocol::browser::methods::GetVersion;
pub use crate::protocol::browser::methods::VersionInformationReturnObject;
use crate::protocol::target::events as target_events;
use crate::protocol::target::methods::{CreateTarget, SetDiscoverTargets};
use crate::protocol::dom;

pub use crate::browser::process::LaunchOptionsBuilder;
use crate::browser::process::{LaunchOptions, Process};
pub use crate::browser::tab::Tab;
use futures::future::Loop;
use futures::sink::Send;
use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;
use tokio;
use websocket;
use websocket::futures::{Async, Future, Poll, Sink, Stream};
use websocket::message::OwnedMessage;
use websocket::r#async::client::{Client, ClientNew};
use websocket::r#async::futures::future::poll_fn;
use websocket::r#async::TcpStream;
use websocket::result::WebSocketError;
use websocket::ClientBuilder;

use crate::protocol::target;
use serde::{Deserialize, Serialize};
mod tt;

/// ["Browser" domain](https://chromedevtools.github.io/devtools-protocol/tot/Browser)
/// (such as for resizing the window in non-headless mode), we currently don't implement those.
///
///
///

type OwnedMessageSender = futures::sync::mpsc::Sender<websocket::message::OwnedMessage>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionId {
    fn from(session_id: String) -> Self {
        Self(session_id)
    }
}

pub enum MethodDestination {
    Target(SessionId),
    Browser,
}

#[derive(Debug)]
struct MethodUtil {
    counter: Arc<AtomicUsize>,
}

impl MethodUtil {
    // if get response by call_id, it's unnecessary to verify session and target_id.
    fn get_chrome_response(owned_message: &OwnedMessage) -> Option<protocol::Response> {
        match Self::get_any_message_from_chrome(owned_message) {
            Some(protocol::Message::Response(browser_response)) => {
                info!("got chrome response. {:?}", browser_response);
                Some(browser_response)
            },
            Some(protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(target_message_event))) => {
                let message = target_message_event.params.message;
                if let Ok(protocol::Message::Response(resp)) = protocol::parse_raw_message(&message) {
                    info!("got message from target response. {:?}", resp);
                    Some(resp)
                } else {
                    None
                }
            },
            _ => None
        }
    }

    fn get_chrome_event(owned_message: &OwnedMessage) -> Option<protocol::Event> {
        if let Some(protocol::Message::Event(browser_event)) =
            Self::get_any_message_from_chrome(owned_message)
        {
            info!("parsed chrome message: {:?}", browser_event);
            match browser_event {
                protocol::Event::ReceivedMessageFromTarget(target_message_event) => {
                    let session_id: SessionId = target_message_event.params.session_id.into();
                    let raw_message = target_message_event.params.message;

                    if let Ok(target_message) = protocol::parse_raw_message(&raw_message) {
                        match target_message {
                            protocol::Message::Event(target_event) => {
                                info!("get event {:?}", target_event);
                                return Some(target_event);
                            }
                            protocol::Message::Response(resp) => {
                                return None;
                            }
                            protocol::Message::ConnectionShutdown => None,
                        }
                    } else {
                        trace!(
                            "Message from target isn't recognised: {:?}",
                            &raw_message[..30]
                        );
                        return None;
                    }
                }
                _ => Some(browser_event),
            }
        } else {
            None
        }
    }
    // protocol::Message cover all possible messages from chrome.
    #[allow(clippy::single_match_else)]
    fn get_any_message_from_chrome(owned_message: &OwnedMessage) -> Option<protocol::Message> {
        match owned_message {
            OwnedMessage::Text(msg) => {
                if let Ok(m) = protocol::parse_raw_message(&msg) {
                    trace!("got protocol message catch all: {:?}", msg);
                    return Some(m);
                } else {
                    error!("got unparsable message from chrome. {}", msg);
                }
            }
            _ => {
                error!("got None text message from chrome. {:?}", owned_message);
                ()
            }
        };
        None
    }

    // fn create_owned_message<T: std::convert::AsRef<str>>(&self, txt: T) -> OwnedMessage {
    //     OwnedMessage::Text(txt.as_ref().to_string())
    // }

    fn create_attach_method(
        &self,
        target_info: &Option<protocol::target::TargetInfo>,
    ) -> Option<(usize, String)> {
        if let Some(ti) = target_info {
            Some(self.create_msg_to_send(
                target::methods::AttachToTarget {
                    target_id: &(ti.target_id),
                    flatten: None,
                },
                MethodDestination::Browser,
            ))
        } else {
            None
        }
    }

    // if you take self, you consume youself.
    fn create_msg_to_send<C>(&self, method: C, destination: MethodDestination) -> (usize, String)
    where
        C: protocol::Method + serde::Serialize,
    {
        let call_id = self.counter.fetch_add(1, Ordering::SeqCst);
        let call = method.to_method_call(call_id);
        let message_text = serde_json::to_string(&call).unwrap();

        match destination {
            MethodDestination::Target(session_id) => {
                let target_method = target::methods::SendMessageToTarget {
                    target_id: None,
                    session_id: Some(session_id.as_str()),
                    message: &message_text,
                };
                self.create_msg_to_send(target_method, MethodDestination::Browser)
            }
            MethodDestination::Browser => {
                info!("sending method: {}", message_text);
                (call_id, message_text)
            }
        }
    }
}

#[derive(Debug)]
struct ChromePage {
    page_target_info: Option<protocol::target::TargetInfo>,
    method_util: Arc<MethodUtil>,
    waiting_method_id: usize,
    session_id: Option<String>,
}

impl ChromePage {
    fn is_page_event_create(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
        if let Some(protocol::Message::Event(any_event_from_server)) =
            MethodUtil::get_any_message_from_chrome(owned_message)
        {
            if let protocol::Event::TargetCreated(target_created_event) = any_event_from_server {
                let target_type = &(target_created_event.params.target_info.target_type);
                match target_type {
                    protocol::target::TargetType::Page => {
                        trace!(
                            "receive page create event. {:?}",
                            target_created_event.params.target_info
                        );
                        self.page_target_info = Some(target_created_event.params.target_info);
                        return Ok(false);
                    }
                    _ => (),
                }
            }
        }
        Ok(true)
    }

    // when got message {\"method\":\"Target.receivedMessageFromTarget\" from chrome, it has a params field, which has a 'message' field, 
    // it's the response to your early method call.
    fn get_document() -> Option<protocol::Response> {
        None
    }

    fn create_msg_to_send<C>(&mut self, method: C, destination: MethodDestination) -> String
    where
        C: protocol::Method + serde::Serialize, {
            let (mid, method_str) = self.method_util.create_msg_to_send(method, destination);
            self.waiting_method_id = mid;
            method_str
        }

    fn create_attach_method(&mut self) -> Option<String> {
        let mut target_id: Option<String> = None;
        if let Some(ti) = &mut self.page_target_info {
            target_id = Some(ti.target_id.clone());
        }
        
        if let Some(ti) = target_id {
            let r = self.create_msg_to_send(
                target::methods::AttachToTarget {
                    target_id: &ti,
                    flatten: None,
                },
                MethodDestination::Browser,
            );
            Some(r)
        } else {
            None
        }
    }

    fn create_msg_to_send_with_session_id<C>(&mut self, method: C, session_id: SessionId) -> String
    where
        C: protocol::Method + serde::Serialize, {
        self.create_msg_to_send(method, MethodDestination::Target(session_id))
    }

    fn query_document_method(&mut self) -> String {
        self.create_msg_to_send_with_session_id(
                dom::methods::GetDocument {
                    depth: Some(0),
                    pierce: Some(false),
                }, self.session_id.clone().unwrap().into())
    }

    fn enable_discover_method(&mut self) -> String {
        self.create_msg_to_send(SetDiscoverTargets { discover: true }, MethodDestination::Browser)
    }

    fn match_waiting_call_response(
        &self,
        owned_message: &OwnedMessage,
    ) -> Option<protocol::Response> {
        if let Some(response) = MethodUtil::get_chrome_response(owned_message) {
            if response.call_id == self.waiting_method_id {
                return Some(response);
            } else {
                info!("got response with call_id: {}, but waiting call_id is: {}", response.call_id, self.waiting_method_id);
            }
        }
        None
    }

    // return Ok(true) if keeping skip.
    fn is_response_for_attach_page(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
        if let Some(response) = self.match_waiting_call_response(owned_message) {
            if let Some(serde_json::value::Value::Object(value)) = response.result {
                if let Some(serde_json::value::Value::String(session_id)) = value.get("sessionId") {
                    self.session_id = Some(session_id.clone());
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn is_page_attach_event(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
        if let Some(protocol::Event::AttachedToTarget(event)) =
            MethodUtil::get_chrome_event(owned_message)
        {
            let attach_to_target_params: protocol::target::events::AttachedToTargetParams =
                event.params;
            let target_info: protocol::target::TargetInfo = attach_to_target_params.target_info;

            match target_info.target_type {
                protocol::target::TargetType::Page => {
                    info!(
                        "got attach to page event and sessionId: {}",
                        attach_to_target_params.session_id
                    );
                    self.session_id = Some(attach_to_target_params.session_id);
                    self.page_target_info = Some(target_info);
                    return Ok(false);
                }
                _ => (),
            }
        }
        Ok(true)
    }

    fn navigate_to(&mut self, url: &str) -> Option<String> {
        let c = Navigate { url };
        let md = MethodDestination::Target(self.session_id.clone().unwrap().into());
        let (mid, method_str) = self.method_util.create_msg_to_send(c, md);
        self.waiting_method_id = mid;
        Some(method_str)
    }

    fn is_page_url_changed(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
        if let Some(protocol::Event::TargetInfoChanged(event)) =
            MethodUtil::get_chrome_event(owned_message)
        {
            let target_info: protocol::target::TargetInfo = event.params.target_info;
            if let Some(self_ti) = &self.page_target_info {
                if (self_ti.target_id == target_info.target_id) && (target_info.url != self_ti.url)
                {
                    info!(
                        "got same target_id: {}, type: {:?}, url: {}",
                        self_ti.target_id, target_info.target_type, target_info.url
                    );
                    self.page_target_info = Some(target_info);
                    return Ok(false);
                } else {
                    info!(
                        "got different target_id1: {}, target_id2: {}",
                        self_ti.target_id, target_info.target_id
                    );
                }
            }
        }
        Ok(true)
    }
}

fn enable_discover_targets(
    method_util: Arc<MethodUtil>,
    sender: future_mpsc::Sender<OwnedMessage>,
    receiver: future_mpsc::Receiver<OwnedMessage>,
    rt: &mut Runtime,
) {
    let chrome_page = Arc::new(Mutex::new(ChromePage {
        page_target_info: None,
        method_util: Arc::clone(&method_util),
        waiting_method_id: 0,
        session_id: None,
    }));

    let chrome_page_clone_1 = Arc::clone(&chrome_page);
    let chrome_page_clone_2 = Arc::clone(&chrome_page);
    let send_and_receive = sender
        .send(OwnedMessage::Text(
            chrome_page.lock().unwrap().enable_discover_method(),
        ))
        .from_err()
        .and_then(|sender| {
            // and_then take a function as parameter, this function must return a IntoFuture which take the same Error type as self (it's sender here.) or the Error implement from self::Error.
            receiver
                .skip_while(move |msg| {
                    chrome_page_clone_1
                        .lock()
                        .unwrap()
                        .is_page_event_create(msg)
                })
                .into_future()
                .and_then(move |(_, s)| Ok((sender, s)))
                .map_err(|e| ChannelBridgeError::ReceivingError)
        });

    let send_and_receive = send_and_receive
        .from_err()
        .and_then(move |(sender, receiver)| {
            let method_str = chrome_page_clone_2
                .lock()
                .unwrap()
                .create_attach_method()
                .unwrap();
            sender
                .send(OwnedMessage::Text(method_str))
                .from_err()
                .and_then(|sender| {
                    receiver
                        .skip_while(move |msg| {
                            let mut chrome = chrome_page_clone_2.lock().unwrap();
                            chrome.is_page_attach_event(msg)
                        })
                        .into_future()
                        .and_then(move |(_, stream)| Ok((sender, stream)))
                        .map_err(|e| ChannelBridgeError::ReceivingError)
                })
                .map_err(|_| ChannelBridgeError::ReceivingError)
        });

    let chrome_page_clone_3 = Arc::clone(&chrome_page);
    let send_and_receive = send_and_receive
        .from_err()
        .and_then(move |(sender, receiver)| {
            let nav_method = chrome_page_clone_3
                .lock()
                .unwrap()
                .navigate_to(
                    "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html",
                )
                .unwrap();

            // this send will return Text("{\"id\":3,\"result\":{}}"), Nothing of result.
            // We should waiting for TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams { target_info: TargetInfo { target_id: "58D612AF212A5A1BE0138AF2971562C6", target_type: Page, title: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", url: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", attached: true, opener_id: None, browser_context_id: Some("11B2711DB1E72BFDC0D91DF5D5C859BC") } } })
            //
            sender
                .send(OwnedMessage::Text(nav_method))
                .from_err()
                .and_then(|sender| {
                    //navigate to new page.
                    receiver
                        .skip_while(move |msg| {
                            info!("waiting navigate success raw_message. {:?}", msg);
                            info!(
                                "waiting navigate success. {:?}",
                                MethodUtil::get_chrome_event(msg)
                            );
                            chrome_page_clone_3.lock().unwrap().is_page_url_changed(msg)
                        })
                        .into_future()
                        .and_then(move |(_, stream)| Ok((sender, stream)))
                        .map_err(|_| ChannelBridgeError::ReceivingError)
                })
                .map_err(|_| ChannelBridgeError::ReceivingError)
        });


    let chrome_page_clone_4 = Arc::clone(&chrome_page);
    let send_and_receive = send_and_receive
        .from_err()
        .and_then(move |(sender, receiver)| {
            let query_document = chrome_page_clone_4
                .lock()
                .unwrap()
                .query_document_method();
            // this send will return Text("{\"id\":3,\"result\":{}}"), Nothing of result.
            // We should waiting for TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams { target_info: TargetInfo { target_id: "58D612AF212A5A1BE0138AF2971562C6", target_type: Page, title: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", url: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", attached: true, opener_id: None, browser_context_id: Some("11B2711DB1E72BFDC0D91DF5D5C859BC") } } })
            //
            sender
                .send(OwnedMessage::Text(query_document))
                .from_err()
                .and_then(|sender| {
                    //navigate to new page.
                    receiver
                        .skip_while(move |msg| {
                            info!("waiting document raw_message. {:?}", msg);
                            info!(
                                "waiting document success. {:?}",
                                MethodUtil::get_chrome_event(msg)
                            );
                            let resp = chrome_page_clone_4.lock().unwrap().match_waiting_call_response(&msg);
                            let b = resp.is_some();
                            info!("query document response: {:?}", resp);
                            Ok(b)
                        })
                        .into_future()
                        .and_then(move |(_, stream)| Ok((sender, stream)))
                        .map_err(|_| ChannelBridgeError::ReceivingError)
                })
                .map_err(|_| ChannelBridgeError::ReceivingError)
        });

    rt.spawn(
        send_and_receive
            // .map(|it| info!("spawn result: {}", it))
            .map(|_| ())
            .map_err(|e| error!("{:?}", e)),
    );
}

#[derive(Debug, failure::Fail)]
enum ChannelBridgeError {
    // #[fail(display = "invalid toolchain name: {}", name)]
    #[fail(display = "send to error")]
    SendingError,
    // #[fail(display = "unknown toolchain version: {}", version)]
    #[fail(display = "receiving error.")]
    ReceivingError,
}

impl std::convert::From<futures::sync::mpsc::SendError<websocket::message::OwnedMessage>>
    for ChannelBridgeError
{
    fn from(t: futures::sync::mpsc::SendError<websocket::message::OwnedMessage>) -> Self {
        ChannelBridgeError::ReceivingError
    }
}


fn runner1(
    rt: &mut Runtime,
) -> (
    Process,
    future_mpsc::Sender<OwnedMessage>,
    future_mpsc::Receiver<OwnedMessage>,
) {
    let options = LaunchOptionsBuilder::default()
        .build()
        .expect("Failed to find chrome");
    let chrome_process = Process::new(options).unwrap();
    let web_socket_debugger_url = chrome_process.debug_ws_url.clone();

    let (writer, rx) = future_mpsc::channel(1_024); // writer
    let (tx1, reader) = future_mpsc::channel(1_024); // reader

    let runner = ClientBuilder::new(&web_socket_debugger_url)
        .unwrap()
        .add_protocol("rust-websocket")
        .async_connect_insecure()
        .from_err::<failure::Error>()
        .map(|(duplex, _)| duplex.split())
        .and_then(|(sink, stream)| {
            // type annotations required: cannot resolve `_: std::convert::From<websocket::result::WebSocketError>`
            let writer_inner = sink
                .sink_from_err::<failure::Error>()
                .send_all(rx.map_err(|()| ChannelBridgeError::SendingError));
            let reader_inner = stream.from_err().forward(tx1);
            reader_inner.join(writer_inner).from_err()
        });

    rt.spawn(
        runner
            .map_err(|e| error!("{:?}", e))
            .map(|_| info!("spawned websocket pairs done.")),
    );

    (chrome_process, writer, reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::page::methods::Navigate;
    use futures::stream::Stream;
    use protocol::page::ScreenshotFormat;
    use tokio;
    use tokio::runtime::Runtime;
    use websocket::futures::{Async, Future, Poll, Sink};
    use websocket::r#async::client::{Client, ClientNew};
    use websocket::r#async::TcpStream;
    use websocket::ClientBuilder;
    use websocket::Message;

    use crate::browser::process::{LaunchOptions, LaunchOptionsBuilder, Process};

    // , Browser, LaunchOptionsBuilder};

    // cd "C:\Program Files (x86)\Google\Chrome\Application\"
    // .\chrome.exe --remote-debugging-port=9222
    // .\chrome.exe --user-data-dir=e:
    // http://localhost:9222/json/version

    // Page
    // Target.targetCreated -> "targetId":"52DEFEF71C5424C72D993A658B55D851"
    // Target.targetInfoChanged" -> "targetId":"52DEFEF71C5424C72D993A658B55D851"
    // Target.attachedToTarget -> "targetId":"52DEFEF71C5424C72D993A658B55D851" , "sessionId":"FCF32E9DD66C89F6246EF9D832D385D1"

    // static chrome_page: ChromePage = ChromePage {
    //             counter: Arc::new(AtomicUsize::new(0))
    //     };

    #[test]
    fn t_loop_fn() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();

        let mut rt = Runtime::new().unwrap();
        let (chrome_process, writer, reader) = runner1(&mut rt);
        // let chrome_page = ChromePage {
        //     counter: Arc::new(AtomicUsize::new(0)),
        // };
        let mu = Arc::new(MethodUtil {
            counter: Arc::new(AtomicUsize::new(0)),
        });

        enable_discover_targets(Arc::clone(&mu), writer, reader, &mut rt);

        // rt.block_on(s).unwrap();
        info!("chrome process: {}", chrome_process.debug_ws_url);

        rt.shutdown_on_idle().wait().unwrap();
    }
}
