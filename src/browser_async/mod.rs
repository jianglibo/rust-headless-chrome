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

// target::methods::AttachToTarget {
//                 target_id: &target_id,
//                 flatten: None,
//             })



fn get_chrome_response(owned_message: Option<OwnedMessage>) -> Option<protocol::Response> {
    None
}



fn get_chrome_event(owned_message: Option<OwnedMessage>) -> Option<protocol::Event> {
    if let Some(protocol::Message::Event(browser_event)) = get_any_message_from_chrome(owned_message) {
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
                    },
                    _ => None
                }
            },

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
                    },

                    protocol::Message::Response(resp) => {
                        // if waiting_call_registry.resolve_call(resp).is_err()
                        // {
                        //     warn!("The browser registered a call but then closed its receiving channel");
                        //     break;
                        // }
                        return None;
                    },
                    protocol::Message::ConnectionShutdown => None,
                }
            } else {
                trace!(
                    "Message from target isn't recognised: {:?}",
                    &raw_message[..30]
                );
                return None;
            }
        },
        _ => None
        }
    } else {
        None
    }
}

fn get_any_message_from_chrome(owned_message: Option<OwnedMessage>) -> Option<protocol::Message> {
    if let Some(om) = owned_message {
        match om {
            OwnedMessage::Text(msg) => {
                info!("got raw message: {}", msg);
                if let Ok(m) = protocol::parse_raw_message(&msg) {
                    return Some(m);
                }
            }
            _ => (),
        }
    }
    None
}

type RawPollResult = std::result::Result<futures::Async<std::option::Option<websocket::message::OwnedMessage>>, websocket::result::WebSocketError>;

type LoopEventResult = std::result::Result<Loop<Option<protocol::Event>, Option<protocol::Event>>, failure::Error>;

fn poll_chrome_message(stream_poll_result: RawPollResult) ->  LoopEventResult {
        match stream_poll_result {
            Ok(Async::NotReady) => Ok(Loop::Continue(None)),
            Ok(Async::Ready(om_op)) => {
                if let Some(m) = get_chrome_event(om_op) {
                    Ok(Loop::Break(Some(m)))
                } else {
                    Ok(Loop::Continue(None))
                }
            }
            Err(e) => Err(failure::Error::from(e)),
        }
}


fn get_page_event_create(owned_message: Option<OwnedMessage>) -> Option<protocol::target::TargetInfo> {
        if let Some(protocol::Message::Event(any_event_from_server)) = get_any_message_from_chrome(owned_message) {
            if let protocol::Event::TargetCreated(target_created_event) = any_event_from_server {
                let target_type = &(target_created_event.params.target_info.target_type);
                match target_type {
                    protocol::target::TargetType::Page => {
                        trace!("i got it. {:?}", target_created_event.params.target_info);
                        return Some(target_created_event.params.target_info);
                    },
                    _ => (),
                }
            }
    }
    None
}

fn create_owned_message<T: std::convert::AsRef<str>>(txt: T) -> OwnedMessage {
    OwnedMessage::Text(txt.as_ref().to_string())
}

struct ChromePage {
    counter: Arc<AtomicUsize>,
}

impl ChromePage {
    fn create_attach_method(&self, target_info: protocol::target::TargetInfo) -> (usize, String) {
                            self.create_msg_to_send(target::methods::AttachToTarget {
                                target_id: &(target_info.target_id),
                                flatten: None,
                            }, 
                            MethodDestination::Browser,
                            )

    }
    // if you take self, you consume youself.
    fn create_msg_to_send<C>(&self,
    method: C,
    destination: MethodDestination,
) -> (usize, String)
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
        },
    }
}

fn poll_page_event_create(&self, stream_poll_result: RawPollResult) ->  std::result::Result<Loop<Option<protocol::target::TargetInfo>, u8>, failure::Error>
// fn poll_page_event_create<T>(stream_poll_result: RawPollResult) -> IntoFuture<Item = Loop<T, _>, Error=failure::Error>
{
        match stream_poll_result {
            Ok(Async::NotReady) => Ok(Loop::Continue(0)),
            Ok(Async::Ready(om_op)) => {
                if let Some(m) = get_page_event_create(om_op) {
                    Ok(Loop::Break(Some(m)))
                } else {
                    Ok(Loop::Continue(0))
                }
            }
            Err(e) => Err(failure::Error::from(e)),
        }
}

fn attach_page(&self, target_info: protocol::target::TargetInfo) -> (usize, String) {
    self.create_msg_to_send(target::methods::AttachToTarget {
                            target_id: &(target_info.target_id),
                            flatten: None,
                        }, 
                        MethodDestination::Browser,
                        )
}
}


// fn vvv(c: Arc<Mutex<Client<TcpStream>>>) {
//     c.lock().unwrap().poll();
// }

fn runner(chrome_page: Arc<Mutex<ChromePage>>, rt: &mut Runtime) {
    // let mut runtime = tokio::runtime::Builder::new().build().unwrap();
    let options = LaunchOptionsBuilder::default()
        .build()
        .expect("Failed to find chrome");
    let chrome_process = Process::new(options).unwrap();
    let web_socket_debugger_url = chrome_process.debug_ws_url.clone();
    // info!("wait 3 sec.");
    // thread::sleep(std::time::Duration::from_secs(3));

    let chrome_page1 = Arc::clone(&chrome_page);
    let chrome_page2 = Arc::clone(&chrome_page);

    let runner = ClientBuilder::new(&web_socket_debugger_url)
        .unwrap()
        .add_protocol("rust-websocket")
        .async_connect_insecure()
        .from_err()
        .and_then(move |(duplex, _)| {

            // let arc_duplex = Arc::new(Mutex::new(duplex));

            // let arc_duplex1 = Arc::clone(&arc_duplex);
            // let arc_duplex2 = Arc::clone(&arc_duplex);

            // vvv(arc_duplex1);

            // arc_duplex.lock().borrow_mut().unwrap().send(OwnedMessage::Text("abc".into()));
            // arc_duplex1.lock().unwrap().poll();
            // let start_counter = ;

            
            
            let (mut sink, mut stream) = duplex.split();
            let (mid, discover) = chrome_page1.lock().unwrap().create_msg_to_send(
                SetDiscoverTargets { discover: true },
                MethodDestination::Browser,
            );
            
            let arc_stream = Arc::new(Mutex::new(stream));
            let arc_stream1 = Arc::clone(&arc_stream);
            // let arc_sink = Arc::new(Mutex::new(sink));
            info!("connected.");
            // let new_counter = Arc::clone(&start_counter);

            let chrome_page1 = Arc::clone(&chrome_page);
            let chrome_page2 = Arc::clone(&chrome_page);

            // let mut first_duplex = arc_duplex.lock().unwrap();
            sink.send(OwnedMessage::Text(discover))
                .from_err()
                .and_then(move |new_sink| {
                    loop_fn(0_u8, move |_| {
                        let poll_result = arc_stream.lock().unwrap().poll();
                        chrome_page.lock().unwrap().poll_page_event_create(poll_result)
                    })
                    .and_then(move |target_info| {
                        let (mid, new_command) = chrome_page1.lock().unwrap().attach_page(target_info.unwrap());
                        new_sink.send(create_owned_message(new_command)).from_err()
                    })
                })
                .and_then(|new_sink| {
                    loop_fn(0_u8, move |_| {
                        let poll_result = arc_stream1.lock().unwrap().poll();
                        chrome_page2.lock().unwrap().poll_page_event_create(poll_result)
                    })
                })
                // .map_err(|_|()).map(|_|())
                // .map(|_| ())
                // Write any error to STDOUT
                // .map_err(|e| println!("socket error = {:?}", e))
        });

    let rrr = runner.map(|v| println!("{:?}", v)).map_err(|_|());
    // rrr
    rt.spawn(rrr);
    // rt.block_on(runner).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::page::methods::Navigate;
    use futures::stream::Stream;
    use protocol::page::ScreenshotFormat;
    use tokio;
    use websocket::futures::{Async, Future, Poll, Sink};
    use websocket::r#async::client::{Client, ClientNew};
    use websocket::r#async::TcpStream;
    use websocket::ClientBuilder;
    use websocket::Message;
    use tokio::runtime::Runtime;

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

    #[test]
    fn t_loop_fn() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();
                let chrome_page = Arc::new(Mutex::new(ChromePage {
                counter: Arc::new(AtomicUsize::new(0))
            }));

        // tokio::run(futures::lazy(move || {
        //     runner(chrome_page);
        //     Ok(())
        // }));

        // runner(chrome_page);
        let mut rt = Runtime::new().unwrap();
        runner(chrome_page, &mut rt);
        // // Spawn the server task
        // rt.spawn(futures::lazy(move || {
        //     runner(chrome_page);
        //     Ok(())
        // }));

        // Wait until the runtime becomes idle and shut it down.
        
        rt.shutdown_on_idle()
            .wait().unwrap();
    }
}
