use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;

use failure::Error;
use log::*;

use serde;
use crate::protocol::page::methods::Navigate;

use crate::protocol;
use crate::protocol::Method;
use crate::protocol::browser::methods::GetVersion;
pub use crate::protocol::browser::methods::VersionInformationReturnObject;
use crate::protocol::target::methods::{CreateTarget, SetDiscoverTargets};

pub use crate::browser::process::LaunchOptionsBuilder;
use crate::browser::process::{LaunchOptions, Process};
pub use crate::browser::tab::Tab;
use std::time::Duration;
// use futures::{Future, Async, Poll};
use websocket::futures::{Async, Future, Poll, Sink, Stream};
use websocket::message::OwnedMessage;
use websocket::r#async::client::{Client, ClientNew};
use websocket::r#async::TcpStream;
use websocket::ClientBuilder;
use websocket;
use futures::sink::Send;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::protocol::target;
use serde::{Deserialize, Serialize};

/// ["Browser" domain](https://chromedevtools.github.io/devtools-protocol/tot/Browser)
/// (such as for resizing the window in non-headless mode), we currently don't implement those.
///
/// 
///

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

pub enum MethodDestination {
    Target(SessionId),
    Browser,
}

pub struct RunningBrowser {
    ws_client: Client<TcpStream>,
    tabs: Arc<Mutex<Vec<Arc<Tab>>>>,
    call_id_counter: Arc<AtomicUsize>,
}

pub enum BrowserFutrue {
    Connecting(Process),
    Running(Arc<Mutex<RunningBrowser>>),
}

// type MethodSend = futures::sink::Send<&'a mut tokio_io::_tokio_codec::framed::Framed<tokio_tcp::stream::TcpStream, websocket::codec::ws::MessageCodec<websocket::message::OwnedMessage>>>;

impl Future for BrowserFutrue {
    type Item = Arc<Mutex<RunningBrowser>>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Arc<Mutex<RunningBrowser>>, Error> {
        use self::BrowserFutrue::*;
        loop {
            match self {
                Connecting(process) => {
                    // let process = Process::new(launch_options)?;
                    let web_socket_debugger_url = process.debug_ws_url.clone();
                    let mut client_future: ClientNew<TcpStream> =
                        ClientBuilder::new(&web_socket_debugger_url)
                            .unwrap()
                            .async_connect_insecure();

                    let (ws_client, _) = try_ready!(client_future.poll());

                    let tabs = Arc::new(Mutex::new(vec![]));
                    let call_id_counter = Arc::new(AtomicUsize::new(0));

                    let browser = RunningBrowser { ws_client, tabs, call_id_counter };
                    *self = Running(Arc::new(Mutex::new(browser)));
                }
                Running(running_browser) => {
                    return Ok(Async::Ready(Arc::clone(running_browser)));
                }
            }
        }
    }
}

pub fn call_method<C>(
    method: C,
    destination: MethodDestination,
    ws_client: Client<TcpStream>,
) -> futures::future::FutureResult<C::ReturnObject, Error>
where
    C: protocol::Method + serde::Serialize,
{
    futures::future::err(failure::err_msg(""))
}

// pub enum MethodInvoker<C>
// where
//     C: protocol::Method + serde::Serialize + std::clone::Clone,
// {
//     PrepareInvoke(C, MethodDestination, Arc<Mutex<RunningBrowser>>),
//     StartInvoke(protocol::CallId, String, Arc<Mutex<RunningBrowser>>),
//     // Sending(Arc<Send>, protocol::CallId, Arc<Mutex<RunningBrowser>>),
//     Invoking(C, protocol::CallId, Arc<Mutex<RunningBrowser>>),
//     Invoked(Arc<Result<C::ReturnObject, Error>>),
// }

// impl<C> Future for MethodInvoker<C>
// where
//     C: protocol::Method + serde::Serialize + std::clone::Clone,
// {
//     type Item = Arc<Result<C::ReturnObject, Error>>;
//     type Error = Error;

//     fn poll(&mut self) -> Poll<Arc<Result<C::ReturnObject, Error>>, Error> {
//         use self::MethodInvoker::*;

//         match self {
//             PrepareInvoke(method_description, destination, running_browser) => {
//                 let call_id = running_browser.lock().unwrap().call_id_counter.fetch_add(1, Ordering::SeqCst);
//                 let call = method_description.clone().to_method_call(call_id);
//                 let mut message_text = serde_json::to_string(&call)?;
//                 match destination {
//                     MethodDestination::Target(session_id) => {
//                         let target_method = target::methods::SendMessageToTarget {
//                             target_id: None,
//                             session_id: Some(session_id.0.as_str()),
//                             message: &message_text,
//                         };
//                         let call_id = running_browser.lock().unwrap().call_id_counter.fetch_add(1, Ordering::SeqCst);
//                         let call = target_method.to_method_call(call_id);
//                         message_text = serde_json::to_string(&call)?;
//                     },
//                     _ => ()
//                 };
//                 *self = StartInvoke(call_id, message_text, Arc::clone(running_browser));
//             },
//             StartInvoke(call_id, message_text, running_browser) => {
//                 let ws_client = &mut running_browser.lock().unwrap().ws_client;
//                 let (sink, stream) = ws_client.split();
                
//                 let sd = sink.send(websocket::Message::text(message_text).into());
                
                
//                 // *self = Sending(Arc::new(sd), *call_id, Arc::clone(running_browser));
//                 // try_ready!(sd.poll());
//             },
//             Invoking(method_description, call_id, running_browser) => {
//                 let mut method_call_result_op: Option<Result<C::ReturnObject, Error>> = None;
//                 {
//                     let ws_client = &mut running_browser.lock().unwrap().ws_client;
//                     if let Some(ws_message) = try_ready!(ws_client.poll()) {
//                         if let OwnedMessage::Text(message_string) = ws_message {
//                             if let Ok(message) = protocol::parse_raw_message(&message_string) {
//                                 match message {
//                                     protocol::Message::Response(response) => {
//                                         if response.call_id == *call_id {
//                                             let return_object =
//                                                 protocol::parse_response::<C::ReturnObject>(
//                                                     response,
//                                                 );
//                                             method_call_result_op = Some(return_object);
//                                         }
//                                     }
//                                     _ => (),
//                                 }
//                             } else {
//                                 debug!(
//                                         "Incoming message isn't recognised as event or method response: {}",
//                                         message_string
//                                     );
//                             }
//                         } else {
//                             panic!("Got a weird message: {:?}", ws_message)
//                         }
//                     }
//                 }
//                 if let Some(method_call_result) = method_call_result_op {
//                     *self = Invoked(Arc::new(method_call_result));
//                 }
//             }
//             Invoked(call_result) => {
//                 return Ok(Async::Ready(Arc::clone(call_result)));
//             }
//         }
//         Ok(Async::NotReady)
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::Stream;
    use protocol::page::ScreenshotFormat;
    use tokio;
    use websocket::futures::{Async, Future, Poll, Sink};
    use websocket::r#async::client::{Client, ClientNew};
    use websocket::r#async::TcpStream;
    use websocket::ClientBuilder;
    use websocket::Message;
    use crate::protocol::page::methods::Navigate;

    use crate::browser::process::{LaunchOptions, LaunchOptionsBuilder, Process};

    // , Browser, LaunchOptionsBuilder};

    // cd "C:\Program Files (x86)\Google\Chrome\Application\"
    // .\chrome.exe --remote-debugging-port=9222
    // .\chrome.exe --user-data-dir=e:
    // http://localhost:9222/json/version

    #[test]
    fn t_listener() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,transport_async=debug");
        env_logger::init();

        let mut runtime = tokio::runtime::Builder::new().build().unwrap();

        let options = LaunchOptionsBuilder::default()
            .build()
            .expect("Failed to find chrome");
        let process = Process::new(options).unwrap();

        let browser_future =
            BrowserFutrue::Connecting(process).and_then(|bw| futures::future::ok(()));

        runtime.block_on(browser_future).unwrap();
    }

    #[test]
    fn t_future() {

        // futures::future::ok()
    // const CONNECTION: &'static str = "ws://127.0.0.1:2794";
    // let runner = ClientBuilder::new(CONNECTION)
	// 	.unwrap()
	// 	.add_protocol("rust-websocket")
	// 	.async_connect_insecure()
	// 	.and_then(|(duplex, _)| {
	// 		let (sink: websocket::sender:Sender, stream) = duplex.split();
	// 		stream
	// 			.filter_map(|message| {
	// 				println!("Received Message: {:?}", message);
	// 				match message {
	// 					OwnedMessage::Close(e) => Some(OwnedMessage::Close(e)),
	// 					OwnedMessage::Ping(d) => Some(OwnedMessage::Pong(d)),
	// 					_ => None,
	// 				}
	// 			})
	// 			.select(stdin_ch.map_err(|_| WebSocketError::NoDataAvailable))
	// 			.forward(sink)
	// 	});
        // let running_browser = RunningBrowser {

        // };
        // let m = MethodInvoker::StartInvoke(Navigate {url: "http://abc"}, Arc::new(Mutex::new(running_browser)));
    }
}
