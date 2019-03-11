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

struct MethodUtil {
    counter: Arc<AtomicUsize>,
}

impl MethodUtil {

fn get_chrome_event(&self, owned_message: &OwnedMessage) -> Option<protocol::Event> {
    if let Some(protocol::Message::Event(browser_event)) =
        self.get_any_message_from_chrome(owned_message)
    {
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

fn get_any_message_from_chrome(&self, owned_message: &OwnedMessage) -> Option<protocol::Message> {
        match owned_message {
            OwnedMessage::Text(msg) => {
                info!("got raw message: {}", msg);
                if let Ok(m) = protocol::parse_raw_message(&msg) {
                    return Some(m);
                }
            }
            _ => (),
        };
        None
}
fn get_page_event_create(&self,
    owned_message: &OwnedMessage,
) -> Option<protocol::target::TargetInfo> {
    if let Some(protocol::Message::Event(any_event_from_server)) =
        self.get_any_message_from_chrome(owned_message)
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

    fn create_attach_method(&self, target_info: protocol::target::TargetInfo) -> (usize, String) {
        self.create_msg_to_send(
            target::methods::AttachToTarget {
                target_id: &(target_info.target_id),
                flatten: None,
            },
            MethodDestination::Browser,
        )
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

// target::methods::AttachToTarget {
//                 target_id: &target_id,
//                 flatten: None,
//             })



// type RawPollResult = std::result::Result<
//     futures::Async<std::option::Option<websocket::message::OwnedMessage>>,
//     websocket::result::WebSocketError,
// >;

// type LoopEventResult =
//     std::result::Result<Loop<Option<protocol::Event>, Option<protocol::Event>>, failure::Error>;

// fn poll_chrome_message(stream_poll_result: RawPollResult) -> LoopEventResult {
//     match stream_poll_result {
//         Ok(Async::NotReady) => Ok(Loop::Continue(None)),
//         Ok(Async::Ready(om_op)) => {
//             if let Some(om) = om_op {
//                 if let Some(m) = get_chrome_event(&om) {
//                     Ok(Loop::Break(Some(m)))
//                 } else {
//                     Ok(Loop::Continue(None))
//                 }
//             } else {
//                 Ok(Loop::Continue(None))
//             }
//         }
//         Err(e) => Err(failure::Error::from(e)),
//     }
// }


struct ChromePage {}

// type SkipResult = Result<bool, ChannelBridgeError>;

// impl ChromePage {

    // fn catch_chrome_event(receiver: future_mpsc::Receiver<OwnedMessage>, method_util: Arc<MethodUtil>) -> futures::stream::SkipWhile<futures::sync::mpsc::Receiver<websocket::message::OwnedMessage>, Fn(&OwnedMessage) -> u8, std::result::Result<bool, ChannelBridgeError>> {
    //         receiver
    //                 .skip_while(move |msg| -> Result<_, ChannelBridgeError> {
    //                     if let Some(ti) = method_util.get_page_event_create(msg) {
    //                         info!("receive message: {:?}", msg);
    //                         // Ok(false)
    //                         Ok(false)
    //                     } else {
    //                         info!("skip message: {:?}", msg);
    //                         Ok(true)
    //                     }
    //                 })        
    // }

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
        let method_util_1 = Arc::clone(&method_util);
        let method_util_2 = Arc::clone(&method_util);
        let and_t = sender
            .send(OwnedMessage::Text(discover))
            .from_err()
            .and_then(|sender1| { // and_then take a function as parameter, this function must return a IntoFuture which take the same Error type as self (it's sender here.) or the Error implement from self::Error.
                let a = receiver
                    .skip_while(move |msg| {
                        if let Some(ti) = method_util_1.get_page_event_create(msg) {
                            info!("receive message: {:?}", msg);
                            Ok(false)
                        } else {
                            info!("skip message: {:?}", msg);
                            Ok(true)
                        }
                    })
                    .into_future()
                    // .from_err()
                    .and_then(|(item, s)| {
                        info!("*** {:?}", item); // this item is page create event.
                        // Ok(item)
                        // let r: Result<_, ChannelBridgeError> = Ok((sender1, s));
                        // r
                        Ok((sender1, s))
                        // if let Some(msg) = item {
                        //     // let (mid, method_str) = method_util_2.create_attach_method(method_util_2.get_page_event_create(&msg).unwrap());
                        //     Ok(true)
                        // } else {
                        //     Ok(false)
                        //     // Ok(())
                        // }
                    });
                a.map_err(|e|ChannelBridgeError::ReceivingError)
            });

        // and_t = and_t.from_err().and_then(|(sender, receiver)| {
        //     let r: Result<u8, ChannelBridgeError> = Ok(9_u8);
        //     r
        // });

        
            // and_t.and_then(|(sender, receiver)|{
            //     Ok("abc")
            // });
                
            // a.map_err(|_| {
            //     error!("here error.");
            //     ChannelBridgeError::ReceivingError
            // })

        rt.spawn(
            and_t
                // .map(|it| info!("spawn result: {}", it))
                .map(|_| ())
                .map_err(|e| error!("{:?}", e)),
        );
    }

//     fn poll_page_event_create(
//         &self,
//         stream_poll_result: RawPollResult,
//     ) -> std::result::Result<Loop<Option<protocol::target::TargetInfo>, u8>, failure::Error>
// // fn poll_page_event_create<T>(stream_poll_result: RawPollResult) -> IntoFuture<Item = Loop<T, _>, Error=failure::Error>
//     {
//         match stream_poll_result {
//             Ok(Async::NotReady) => Ok(Loop::Continue(0)),
//             Ok(Async::Ready(om_op)) => {
//                 if let Some(om) = om_op {
//                      if let Some(m) = get_page_event_create(&om) {
//                         return Ok(Loop::Break(Some(m)));
//                     }                   
//                 }
//                 Ok(Loop::Continue(0))
//             }
//             Err(e) => Err(failure::Error::from(e)),
//         }
//     }

    // fn attach_page(&self, target_info: protocol::target::TargetInfo) -> (usize, String) {
    //     self.create_msg_to_send(
    //         target::methods::AttachToTarget {
    //             target_id: &(target_info.target_id),
    //             flatten: None,
    //         },
    //         MethodDestination::Browser,
    //     )
    // }

// }

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

    // fn runner(&'static self, rt: &mut Runtime) {
    //     // let mut runtime = tokio::runtime::Builder::new().build().unwrap();
    //     let options = LaunchOptionsBuilder::default()
    //         .build()
    //         .expect("Failed to find chrome");
    //     let chrome_process = Process::new(options).unwrap();
    //     let web_socket_debugger_url = chrome_process.debug_ws_url.clone();
    //     // info!("wait 3 sec.");
    //     // thread::sleep(std::time::Duration::from_secs(3));

    //     // let chrome_page1 = Arc::clone(&chrome_page);
    //     // let chrome_page2 = Arc::clone(&chrome_page);

    //     // let (tx, rx) = future_mpsc::channel(1_024);

    //     let runner = ClientBuilder::new(&web_socket_debugger_url)
    //         .unwrap()
    //         .add_protocol("rust-websocket")
    //         .async_connect_insecure()
    //         .from_err()
    //         .and_then(move |(duplex, _)| {
    //             let (mut sink, mut stream) = duplex.split();
    //             let (mid, discover) = self.create_msg_to_send(
    //                 SetDiscoverTargets { discover: true },
    //                 MethodDestination::Browser,
    //             );

    //             // stream.for_each()

    //             let arc_stream = Arc::new(Mutex::new(stream));
    //             let arc_stream1 = Arc::clone(&arc_stream);
    //             // let arc_sink = Arc::new(Mutex::new(sink));
    //             info!("connected.");
    //             // let new_counter = Arc::clone(&start_counter);

    //             // let chrome_page1 = Arc::clone(&chrome_page);
    //             // let chrome_page2 = Arc::clone(&chrome_page);

    //             // let mut first_duplex = arc_duplex.lock().unwrap();
    //             sink.send(OwnedMessage::Text(discover))
    //                 .from_err()
    //                 .and_then(move |new_sink| {
    //                     loop_fn(0_u8, move |_| {
    //                         let poll_result = arc_stream.lock().unwrap().poll();
    //                         self.poll_page_event_create(poll_result)
    //                     })
    //                     .and_then(move |target_info| {
    //                         let (mid, new_command) = self.attach_page(target_info.unwrap());
    //                         new_sink.send(create_owned_message(new_command)).from_err()
    //                     })
    //                 })
    //                 .and_then(move |new_sink| {
    //                     loop_fn(0_u8, move |_| {
    //                         let poll_result = arc_stream1.lock().unwrap().poll();
    //                         self.poll_page_event_create(poll_result)
    //                     })
    //                 })
    //         });

    //     let rrr = runner.map(|v| println!("{:?}", v)).map_err(|_| ());
    //     rt.spawn(rrr);
    // }