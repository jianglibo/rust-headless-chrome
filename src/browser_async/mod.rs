mod dev_tools_method_util;
mod chrome_browser;
mod my_page;
mod one_page;


// pub use crate::protocol::browser::methods::VersionInformationReturnObject;
// use crate::protocol::target::methods::{CreateTarget, SetDiscoverTargets};
// use crate::protocol::dom;

// pub use crate::browser::process::LaunchOptionsBuilder;
// use crate::browser::process::{LaunchOptions, Process};
// pub use crate::browser::tab::Tab;
// use futures::future::Loop;
// use futures::sink::Send;
// use std::borrow::BorrowMut;
// use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
// use std::time::Duration;
// use tokio;
// use websocket;
// use websocket::futures::{Async, Future, Poll, Sink, Stream};
// use websocket::message::OwnedMessage;
// use websocket::r#async::client::{Client, ClientNew};
// use websocket::r#async::futures::future::poll_fn;
// use websocket::r#async::TcpStream;
// use websocket::result::WebSocketError;
// use websocket::ClientBuilder;

// use crate::protocol::target;


// use crate::browser_async::dev_tools_method_util::{MethodUtil, ChromePage, ChannelBridgeError, ChromePageError};

// /// ["Browser" domain](https://chromedevtools.github.io/devtools-protocol/tot/Browser)
// /// (such as for resizing the window in non-headless mode), we currently don't implement those.
// ///
// ///
// ///

// fn enable_discover_targets(
//     sender: future_mpsc::Sender<OwnedMessage>,
//     receiver: future_mpsc::Receiver<OwnedMessage>,
//     rt: &mut Runtime,
// ) {
//     let chrome_page = Arc::new(Mutex::new(ChromePage {
//         page_target_info: None,
//         waiting_call_id: None,
//         waiting_message_id: None,
//         session_id: None,
//         root_node: None,
//     }));

//     // let (out_side_sink, inner_stream) = future_mpsc::channel(1_024); // writer
//     // let (inner_sink, out_side_stream) = future_mpsc::channel(1_024); // reader


//     let chrome_page_clone_1 = Arc::clone(&chrome_page);
//     let chrome_page_clone_2 = Arc::clone(&chrome_page);
//     let send_and_receive = sender
//         .send(OwnedMessage::Text(
//             chrome_page.lock().unwrap().enable_discover_method().unwrap().1,
//         ))
//         .from_err()
//         .and_then(|sender| {
//             // and_then take a function as parameter, this function must return a IntoFuture which take the same Error type as self (it's sender here.) or the Error implement from self::Error.
//             let r = receiver
//                 .skip_while(move |msg| {
//                     chrome_page_clone_1
//                         .lock()
//                         .unwrap()
//                         .is_page_event_create(msg)
//                 })
//                 .into_future()
//                 .and_then(move |(_, s)| Ok((sender, s)))
//                 .map_err(|e| ChannelBridgeError::Receiving);
//             r
//         });

//     let chrome_page_clone_6 = Arc::clone(&chrome_page);
//     let send_and_receive = send_and_receive
//         .from_err()
//         .and_then(move |(sender, receiver)| {
//             let (_, method_str, _) = chrome_page_clone_6
//                 .lock()
//                 .unwrap()
//                 .enable_page_notifications()
//                 .unwrap();

//             sender
//                 .send(OwnedMessage::Text(method_str))
//                 .from_err()
//                 .and_then(|sender| {
//                     receiver
//                         .skip_while(move |msg| {
//                             Ok(false)
//                         })
//                         .into_future()
//                         .and_then(move |(_, stream)| Ok((sender, stream)))
//                         .map_err(|e| ChannelBridgeError::Receiving)
//                 })
//                 .map_err(|_| ChannelBridgeError::Receiving)
//         });


//     let send_and_receive = send_and_receive
//         .from_err()
//         .and_then(move |(sender, receiver)| {
//             let (_, method_str, _) = chrome_page_clone_2
//                 .lock()
//                 .unwrap()
//                 .create_attach_method()
//                 .unwrap();
//             sender
//                 .send(OwnedMessage::Text(method_str))
//                 .from_err()
//                 .and_then(|sender| {
//                     receiver
//                         .skip_while(move |msg| {
//                             let mut chrome = chrome_page_clone_2.lock().unwrap();
//                             chrome.is_page_attach_event(msg)
//                         })
//                         .into_future()
//                         .and_then(move |(_, stream)| Ok((sender, stream)))
//                         .map_err(|e| ChannelBridgeError::Receiving)
//                 })
//                 .map_err(|_| ChannelBridgeError::Receiving)
//         });

//     let chrome_page_clone_3 = Arc::clone(&chrome_page);
//     let send_and_receive = send_and_receive
//         .from_err()
//         .and_then(move |(sender, receiver)| {
//             let nav_method = chrome_page_clone_3
//                 .lock()
//                 .unwrap()
//                 .navigate_to(
//                     "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html",
//                 )
//                 .unwrap();

//             // this send will return Text("{\"id\":3,\"result\":{}}"), Nothing of result.
//             // We should waiting for TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams { target_info: TargetInfo { target_id: "58D612AF212A5A1BE0138AF2971562C6", target_type: Page, title: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", url: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", attached: true, opener_id: None, browser_context_id: Some("11B2711DB1E72BFDC0D91DF5D5C859BC") } } })
//             //
//             sender
//                 .send(OwnedMessage::Text(nav_method.1))
//                 .from_err()
//                 .and_then(|sender| {
//                     //navigate to new page.
//                     receiver
//                         .skip_while(move |msg| {
//                             info!("waiting navigate success raw_message. {:?}", msg);
//                             info!(
//                                 "waiting navigate success. {:?}",
//                                 MethodUtil::get_chrome_event(msg)
//                             );
//                             chrome_page_clone_3.lock().unwrap().is_page_url_changed(msg)
//                         })
//                         .into_future()
//                         .and_then(move |(_, stream)| Ok((sender, stream)))
//                         .map_err(|_| ChannelBridgeError::Receiving)
//                 })
//                 .map_err(|_| ChannelBridgeError::Receiving)
//         });

// // 

//     let chrome_page_clone_4 = Arc::clone(&chrome_page);
//     let send_and_receive = send_and_receive
//         .from_err()
//         .and_then(move |(sender, receiver)| {
//             let (_, method_str, option_call_id) = chrome_page_clone_4
//                 .lock()
//                 .unwrap()
//                 .query_document_method().unwrap();

//             let call_id = option_call_id.unwrap();
//             // this send will return Text("{\"id\":3,\"result\":{}}"), Nothing of result.
//             // We should waiting for TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams { target_info: TargetInfo { target_id: "58D612AF212A5A1BE0138AF2971562C6", target_type: Page, title: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", url: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", attached: true, opener_id: None, browser_context_id: Some("11B2711DB1E72BFDC0D91DF5D5C859BC") } } })
//             //
//                         sender
//                             .send(OwnedMessage::Text(method_str))
//                             .from_err()
//                             .and_then(move |sender| {
//                                 //navigate to new page.
//                                 receiver
//                                     .skip_while(move |msg| {
//                                         info!("waiting document raw_message. {:?}", msg);
//                                         info!(
//                                             "waiting document success. {:?}",
//                                             MethodUtil::get_chrome_event(msg)
//                                         );
//                                         let resp = chrome_page_clone_4.lock().unwrap().match_document_by_call_id(&msg, call_id);
//                                         let b = !resp.is_some();
//                                         if !b {
//                                             chrome_page_clone_4.lock().unwrap().root_node = resp;
//                                         }
//                                         // info!("query document response: {:?}", resp);
//                                         Ok(b)
//                                     })
//                                     .into_future()
//                                     .and_then(move |(_, stream)| Ok((sender, stream)))
//                                     .map_err(|_| ChannelBridgeError::Receiving)
//                             })
//                             .map_err(|_| ChannelBridgeError::Receiving)
//         });


//     let chrome_page_clone_5 = Arc::clone(&chrome_page);
//     let send_and_receive = send_and_receive
//         .from_err()
//         .and_then(move |(sender, receiver)| {
//             let (_, method_str, option_call_id) = chrome_page_clone_5
//                 .lock()
//                 .unwrap()
//                 .find_node_method("#ddlogin").unwrap();

//             let call_id = option_call_id.unwrap();
//             // this send will return Text("{\"id\":3,\"result\":{}}"), Nothing of result.
//             // We should waiting for TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams { target_info: TargetInfo { target_id: "58D612AF212A5A1BE0138AF2971562C6", target_type: Page, title: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", url: "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/index.html", attached: true, opener_id: None, browser_context_id: Some("11B2711DB1E72BFDC0D91DF5D5C859BC") } } })
//             //
//                         sender
//                             .send(OwnedMessage::Text(method_str))
//                             .from_err()
//                             .and_then(move |sender| {
//                                 //navigate to new page.
//                                 receiver
//                                     .skip_while(move |msg| {
//                                         info!("waiting document raw_message. {:?}", msg);
//                                         info!(
//                                             "waiting document success. {:?}",
//                                             MethodUtil::get_chrome_event(msg)
//                                         );
//                                         let resp = chrome_page_clone_5.lock().unwrap().match_query_selector_by_call_id(&msg, call_id);
//                                         let b = !resp.is_some();
//                                         info!("query document response: {:?}", resp);
//                                         // Ok(b)
//                                         Ok(true)
//                                     })
//                                     .into_future()
//                                     .and_then(move |(_, stream)| Ok((sender, stream)))
//                                     .map_err(|_| ChannelBridgeError::Receiving)
//                             })
//                             .map_err(|_| ChannelBridgeError::Receiving)

//         });

//     rt.spawn(
//         send_and_receive
//             // .map(|it| info!("spawn result: {}", it))
//             .map(|_| ())
//             .map_err(|e| error!("{:?}", e)),
//     );
// }



// fn runner1(
//     rt: &mut Runtime,
// ) -> (
//     Process,
//     future_mpsc::Sender<OwnedMessage>,
//     future_mpsc::Receiver<OwnedMessage>,
// ) {
//     let options = LaunchOptionsBuilder::default()
//         .build()
//         .expect("Failed to find chrome");
//     let chrome_process = Process::new(options).unwrap();
//     let web_socket_debugger_url = chrome_process.debug_ws_url.clone();

//     let (out_side_sink, inner_stream) = future_mpsc::channel(1_024); // writer
//     let (inner_sink, out_side_stream) = future_mpsc::channel(1_024); // reader

//     let runner = ClientBuilder::new(&web_socket_debugger_url)
//         .unwrap()
//         .add_protocol("rust-websocket")
//         .async_connect_insecure()
//         .from_err::<failure::Error>()
//         .map(|(duplex, _)| duplex.split())
//         .and_then(|(sink, stream)| {
//             // type annotations required: cannot resolve `_: std::convert::From<websocket::result::WebSocketError>`
//             let writer_inner = sink
//                 .sink_from_err::<failure::Error>()
//                 .send_all(inner_stream.map_err(|()| ChannelBridgeError::Sending));
//             let reader_inner = stream.from_err().forward(inner_sink);
//             reader_inner.join(writer_inner).from_err()
//         });

//     rt.spawn(
//         runner
//             .map_err(|e| error!("{:?}", e))
//             .map(|_| info!("spawned websocket pairs done.")),
//     );

//     (chrome_process, out_side_sink, out_side_stream)
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::protocol::page::methods::Navigate;
//     use futures::stream::Stream;
//     use crate::protocol::page::ScreenshotFormat;
//     use tokio;
//     use tokio::runtime::Runtime;
//     use websocket::futures::{Async, Future, Poll, Sink};
//     use websocket::r#async::client::{Client, ClientNew};
//     use websocket::r#async::TcpStream;
//     use websocket::ClientBuilder;
//     use websocket::Message;

//     use crate::browser::process::{LaunchOptions, LaunchOptionsBuilder, Process};

//     // , Browser, LaunchOptionsBuilder};

//     // cd "C:\Program Files (x86)\Google\Chrome\Application\"
//     // .\chrome.exe --remote-debugging-port=9222
//     // .\chrome.exe --user-data-dir=e:
//     // http://localhost:9222/json/version

//     // Page
//     // Target.targetCreated -> "targetId":"52DEFEF71C5424C72D993A658B55D851"
//     // Target.targetInfoChanged" -> "targetId":"52DEFEF71C5424C72D993A658B55D851"
//     // Target.attachedToTarget -> "targetId":"52DEFEF71C5424C72D993A658B55D851" , "sessionId":"FCF32E9DD66C89F6246EF9D832D385D1"

//     // static chrome_page: ChromePage = ChromePage {
//     //             counter: Arc::new(AtomicUsize::new(0))
//     //     };

//     #[test]
//     fn t_loop_fn() {
//         ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
//         env_logger::init();

//         let mut rt = Runtime::new().unwrap();
//         let (chrome_process, writer, reader) = runner1(&mut rt);
//         // let chrome_page = ChromePage {
//         //     counter: Arc::new(AtomicUsize::new(0)),
//         // };
//         // let mu = Arc::new(MethodUtil::new());

//         enable_discover_targets(writer, reader, &mut rt);

//         // rt.block_on(s).unwrap();
//         info!("chrome process: {}", chrome_process.debug_ws_url);

//         rt.shutdown_on_idle().wait().unwrap();
//     }
// }
