use crate::protocol;
use futures::sync::mpsc as future_mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::runtime::Runtime;

use failure;
use log::*;

use serde;

pub use crate::protocol::browser::methods::VersionInformationReturnObject;
use crate::protocol::dom;
use crate::protocol::target::methods::{CreateTarget, SetDiscoverTargets};

pub use crate::browser::process::LaunchOptionsBuilder;
use crate::browser::process::{LaunchOptions, Process};
pub use crate::browser::tab::Tab;
use futures::future::Loop;
use futures::sink::Send;
use std::borrow::BorrowMut;
use std::fmt;
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

use crate::browser_async::chrome_page::{
    ChannelBridgeError, ChromePage, ChromePageError, MethodUtil,
};

enum BrowserState {
    Unconnected,
    Connecting(ClientNew<TcpStream>),
    Connected(
        future_mpsc::Sender<OwnedMessage>,
        future_mpsc::Receiver<OwnedMessage>,
    ),
}

struct ChromeBrowser {
    state: BrowserState,
    process: Option<Process>,
}

impl Stream for ChromeBrowser {
    // type Item = (Process, future_mpsc::Sender<OwnedMessage>, future_mpsc::Receiver<OwnedMessage>);
    type Item = protocol::Message;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            match &mut self.state {
                BrowserState::Unconnected => {
                    let options = LaunchOptionsBuilder::default()
                        .build()
                        .expect("Failed to find chrome");
                    let chrome_process = Process::new(options).unwrap();
                    let web_socket_debugger_url = chrome_process.debug_ws_url.clone();
                    self.process = Some(chrome_process);
                    self.state = BrowserState::Connecting(
                        ClientBuilder::new(&web_socket_debugger_url)
                            .unwrap()
                            .add_protocol("rust-websocket")
                            .async_connect_insecure(),
                    );
                }
                BrowserState::Connecting(client_new) => {
                    let framed = try_ready!(client_new.poll());
                    let (sink, stream) = framed.0.split();
                    info!("connected.");

                    let (out_side_sink, inner_stream) = future_mpsc::channel(1_024); // writer
                    let (inner_sink, out_side_stream) = future_mpsc::channel(1_024); // reader

                    let write_inner = sink.sink_from_err().send_all(inner_stream.map_err(|_|ChannelBridgeError::Sending));
                    let reader_inner = stream.from_err::<ChannelBridgeError>().forward(inner_sink);

                    tokio::spawn(reader_inner.from_err().join(write_inner).map_err(|_|()));

                    self.state = BrowserState::Connected(out_side_sink, out_side_stream);
                }
                BrowserState::Connected(out_side_sink, out_side_stream) => {
                    // return Ok(Async::Ready((self.process.unwrap(), out_side_sink, out_side_stream)));
                    if let Ok(Async::Ready(Some(message))) = out_side_stream.poll() {
                        if let OwnedMessage::Text(msg) = message {
                            let parsed_message = protocol::parse_raw_message(&msg);
                            return Ok(Async::Ready(Some(parsed_message.unwrap())));
                        } else {
                            error!("got unknown message: {:?}", message);
                        }
                    }
                }
            }
        }
    }
}


// extern crate tokio;
// extern crate futures;

// use futures::future::lazy;

// tokio::run(lazy(|| {
//     for i in 0..4 {
//         tokio::spawn(lazy(move || {
//             println!("Hello from task {}", i);
//             Ok(())
//         }));
//     }

//     Ok(())
// }));

pub struct Display10<T> {
    stream: T,
}

impl<T> Display10<T> {
    fn new(stream: T) -> Self {
        Self { stream }
    }
}

impl<T> Future for Display10<T>
where
    T: Stream,
    T::Item: fmt::Debug,
{
    type Item = ();
    type Error = T::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            let value = match try_ready!(self.stream.poll()) {
                Some(value) => value,
                // There were less than 10 values to display, terminate the
                // future.
                None => break,
            };
            println!("value {:?}", value);
        }
        Ok(Async::Ready(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::page::methods::Navigate;
    use crate::protocol::page::ScreenshotFormat;
    use futures::stream::Stream;
    use tokio;
    use tokio::runtime::Runtime;
    use websocket::futures::{Async, Future, Poll, Sink};
    use websocket::r#async::client::{Client, ClientNew};
    use websocket::r#async::TcpStream;
    use websocket::ClientBuilder;
    use websocket::Message;

    use crate::browser::process::{LaunchOptions, LaunchOptionsBuilder, Process};

    #[test]
    fn t_by_enum() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();

        let fib = ChromeBrowser {
            state: BrowserState::Unconnected,
            process: None,
        };
        let display = Display10::new(fib);

        tokio::run(display.map_err(|_| ()));

        // let mut rt = Runtime::new().unwrap();
        // rt.shutdown_on_idle().wait().unwrap();
    }
}
