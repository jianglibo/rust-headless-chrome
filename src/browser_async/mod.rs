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

// protocol::Message::Response is response for method call.
fn get_chrome_response(owned_message: &OwnedMessage) -> Option<protocol::Response> {
        if let Some(protocol::Message::Response(browser_response)) =
        Self::get_any_message_from_chrome(owned_message)
    {
        info!("got chrome response: {:?}", browser_response);
        Some(browser_response)
    } else {
        None
    }
}

fn get_chrome_event(owned_message: &OwnedMessage) -> Option<protocol::Event> {
    if let Some(protocol::Message::Event(browser_event)) =
        Self::get_any_message_from_chrome(owned_message)
    {
        info!("parsed chrome message: {:?}", browser_event);
        match browser_event {
            protocol::Event::TargetCreated(target_created_event) => {
                info!("final event: {:?}", target_created_event);
                // pub struct TargetInfo {
                // pub target_id: TargetId,
                // #[serde(rename = "type")]
                // pub target_type: TargetType,
                // pub title: String,
                // pub url: String,
                // pub attached: bool,
                // pub opener_id: Option<String>,
                // pub browser_context_id: Option<String>,
                // pub enum TargetType {
                //     Page,
                //     BackgroundPage,
                //     ServiceWorker,
                //     Browser,
                //     Other,

                let target_type = &(target_created_event.params.target_info.target_type);
                match target_type {
                    protocol::target::TargetType::Page => {
                        Some(protocol::Event::TargetCreated(target_created_event))
                    }
                    _ => None,
                }
            }

            protocol::Event::ReceivedMessageFromTarget(target_message_event) => {
                let session_id: SessionId = target_message_event.params.session_id.into();
                let raw_message = target_message_event.params.message;

                if let Ok(target_message) = protocol::parse_raw_message(&raw_message) {
                    match target_message {
                        protocol::Message::Event(target_event) => {
                            // if let Some(tx) = listeners
                            //     .lock()
                            //     .unwrap()
                            //     .get(&ListenerId::SessionId(session_id))
                            // {
                            //     tx.send(target_event)
                            //         .expect("Couldn't send event to listener");
                            // } else {
                            //     trace!("discard target_event {:?}", target_event);
                            // }
                            info!("get event {:?}", target_event);
                            return Some(target_event);
                        }

                        protocol::Message::Response(resp) => {
                            // if waiting_call_registry.resolve_call(resp).is_err()
                            // {
                            //     warn!("The browser registered a call but then closed its receiving channel");
                            //     break;
                            // }
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
            _ => None,
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
                    return Some(m);
                } else {
                    error!("got unparsable message from chrome. {}", msg);
                }
            },
            _ => {
                error!("got None text message from chrome. {:?}", owned_message);
                ()
            },
        };
        None
}
fn match_page_event_create(owned_message: &OwnedMessage) -> Option<protocol::target::TargetInfo> {
    if let Some(protocol::Message::Event(any_event_from_server)) =
        Self::get_any_message_from_chrome(owned_message)
    {
        if let protocol::Event::TargetCreated(target_created_event) = any_event_from_server {
            let target_type = &(target_created_event.params.target_info.target_type);
            match target_type {
                protocol::target::TargetType::Page => {
                    trace!("i got it. {:?}", target_created_event.params.target_info);
                    return Some(target_created_event.params.target_info);
                }
                _ => (),
            }
        }
    }
    None
}

// fn create_owned_message<T: std::convert::AsRef<str>>(&self, txt: T) -> OwnedMessage {
//     OwnedMessage::Text(txt.as_ref().to_string())
// }

    fn create_attach_method(&self, target_info: &Option<protocol::target::TargetInfo>) -> Option<(usize, String)> {
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
    target_info: Option<protocol::target::TargetInfo>,
    method_util: Arc<MethodUtil>,
    waiting_method_id: usize,
    session_id: Option<String>,
}

impl ChromePage {
    fn create_attach_method(&mut self) -> Option<String> {
        if let Some(ti) = &self.target_info {
            let (mid, method_str) = self.method_util.create_msg_to_send(
                target::methods::AttachToTarget {
                    target_id: &(ti.target_id),
                    flatten: None,
                },
                MethodDestination::Browser,
            );
            self.waiting_method_id = mid;
            Some(method_str)
        } else {
            None
        }
    }

    fn match_waiting_call_response(&self, owned_message: &OwnedMessage) -> Option<protocol::Response> {
        if let Some(response) = MethodUtil::get_chrome_response(owned_message) {
            if response.call_id == self.waiting_method_id {
                return Some(response);
            }
        }
        None
    }
}

    fn enable_discover_targets(
        method_util: Arc<MethodUtil>,
        sender: future_mpsc::Sender<OwnedMessage>,
        receiver: future_mpsc::Receiver<OwnedMessage>,
        rt: &mut Runtime,
    ) {
        let (mid, discover) = method_util.create_msg_to_send(
            SetDiscoverTargets { discover: true },
            MethodDestination::Browser,
        );

        let chrome_page = Arc::new(Mutex::new(ChromePage {
            target_info: None,
            method_util: Arc::clone(&method_util),
            waiting_method_id: 0,
            session_id: None,
        }));
        let chrome_page_clone_1 =  Arc::clone(&chrome_page);

        let method_util_clone_1 = Arc::clone(&method_util);
        let and_t = sender
            .send(OwnedMessage::Text(discover))
            .from_err()
            .and_then(|sender| { // and_then take a function as parameter, this function must return a IntoFuture which take the same Error type as self (it's sender here.) or the Error implement from self::Error.
                receiver
                    .skip_while(move |msg| {
                        if let Some(ti) = MethodUtil::match_page_event_create(msg) {
                            chrome_page_clone_1.lock().unwrap().target_info = Some(ti);
                            info!("waiting for page create message, got message: {:?}", msg);
                            Ok(false)
                        } else {
                            info!("waiting for page create message, skip message: {:?}", msg);
                            Ok(true)
                        }
                    })
                    .into_future()
                    .and_then(move |(_, s)| {
                        Ok((sender, s))
                    }).map_err(|e|ChannelBridgeError::ReceivingError)
            });

        let chrome_page_clone_2 =  Arc::clone(&chrome_page);
        let method_util_clone_2 = Arc::clone(&method_util);
        let and_t = and_t.from_err().and_then(move|(sender, receiver)| {
            let method_str = chrome_page_clone_2.lock().unwrap().create_attach_method().unwrap();
            sender.send(OwnedMessage::Text(method_str)).from_err().and_then(|sender|{
                receiver.skip_while(move |msg| {
                    info!("waiting attach success. {:?}", msg);
                    info!("waiting attach success. {:?}", MethodUtil::get_chrome_event(msg));
                    let mut chrome = chrome_page_clone_2.lock().unwrap();
                    if let Some(response) = chrome.match_waiting_call_response(msg) {
                        if let Some(serde_json::value::Value::Object(value)) = response.result {
                            if let Some(serde_json::value::Value::String(session_id)) = value.get("sessionId") {
                                chrome.session_id = Some(session_id.clone());
                                info!("{:?}", chrome);
                                return Ok(false);
                            }
                        }
                    }
                    // Text("{\"id\":1,\"result\":{\"sessionId\":\"582952E26A7216935DB42D97332EA591\"}}")
                    // AttachedToTarget(AttachedToTargetEvent { params: AttachedToTargetParams { session_id: "C0C21A585CB64F2DA76203D86D4A849B", target_info: TargetInfo { target_id: "37F3D648D4851AA1F203E03C77640B03", target_type: Page, title: "", url: "about:blank", attached: true, opener_id: None, browser_context_id: Some("CA26923A2EEFF72F824B257037870C7E") }, waiting_for_debugger: false } })
                    // TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams { target_info: TargetInfo { target_id: "37F3D648D4851AA1F203E03C77640B03", target_type: Page, title: "about:blank", url: "about:blank", attached: true, opener_id: None, browser_context_id: Some("CA26923A2EEFF72F824B257037870C7E") } } })

                    Ok(true)
                }).into_future().map_err(|e|ChannelBridgeError::ReceivingError)
            })
        });

        rt.spawn(
            and_t
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

// impl From<ChannelBridgeError> for Send

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