use crate::protocol;

use failure;
use log::*;
use std::fmt;


pub use crate::protocol::browser::methods::VersionInformationReturnObject;

pub use crate::browser::process::LaunchOptionsBuilder;
use crate::browser::process::Process;
pub use crate::browser::tab::Tab;
use futures::AsyncSink;
use std::default::Default;
use std::time::{Duration, Instant};
use websocket::futures::{Async, Future, Poll, Sink, Stream};
use websocket::message::OwnedMessage;
use websocket::r#async::client::{Client, ClientNew};
use websocket::r#async::TcpStream;
use websocket::ClientBuilder;
use std::collections::VecDeque;

type WsClient = Client<TcpStream>;

enum BrowserState {
    Unconnected,
    Connecting(ClientNew<TcpStream>),
    Receiving,
    StartSend(String),
    Sending,
}

impl fmt::Debug for BrowserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrowserState::Connecting(_) => write!(f, "connecting"),
            BrowserState::Unconnected => write!(f, "Unconnected"),
            BrowserState::Receiving => write!(f, "Receiving"),
            BrowserState::StartSend(content) => write!(f, "start sending: {}", content),
            BrowserState::Sending => write!(f, "Sending"),
        }
    }
}

pub struct ChromeBrowser {
    state: BrowserState,
    ws_client: Option<WsClient>,
    process: Option<Process>,
    last_be_polled: Instant,
    waiting_to_send: VecDeque<String>,
}

impl std::fmt::Debug for ChromeBrowser {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "ChromeBrowser {:?}, {:?}",
            self.state, self.last_be_polled
        )
    }
}

impl Default for ChromeBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl ChromeBrowser {
    pub fn new() -> Self {
        Self {
            state: BrowserState::Unconnected,
            ws_client: None,
            process: None,
            last_be_polled: Instant::now(),
            waiting_to_send: VecDeque::new(),
        }
    }
    pub fn send_message(&mut self, method_str: String) {
        trace!("**sending** : {:?}", method_str);
        match self.state {
            BrowserState::StartSend(_) | BrowserState::Sending => {
                self.waiting_to_send.push_back(method_str);
            }
            _ => {
                self.state = BrowserState::StartSend(method_str);
            }
        }
    }

    pub fn have_not_be_polled_for(&self, duration: Duration) -> bool {
        (self.last_be_polled - Instant::now()) > duration
    }
}

impl Stream for ChromeBrowser {
    type Item = protocol::Message;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.last_be_polled = Instant::now();
        loop {
            // trace!("browser loop {:?}", self.state);
            match &mut self.state {
                BrowserState::Unconnected => {
                    trace!("enter unconnected state.");
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
                    trace!("enter connecting state.");
                    let framed = try_ready!(client_new.poll());
                    info!("connected.");
                    self.ws_client = Some(framed.0);
                    self.state = BrowserState::Receiving;
                    return Ok(Some(protocol::Message::Connected).into());
                }
                BrowserState::Receiving => {
                    match self.ws_client.as_mut().unwrap().poll() {
                        Ok(Async::Ready(Some(message))) => {
                            if let OwnedMessage::Text(msg) = message {
                                let parsed_message = protocol::parse_raw_message(&msg);
                                trace!("got message (***every message***): {:?}", msg);
                                return Ok(Async::Ready(Some(parsed_message.expect("parsed_message"))));
                            } else {
                                error!("got unknown message: {:?}", message);
                            }
                        }
                        Ok(Async::Ready(None)) => {
                            trace!("enter receiving None, end?");
                            return Ok(Async::Ready(None));
                        }
                        Ok(Async::NotReady) => {
                            // if return not ready, when to pull again is job of underlying. is out of our controls.
                            // trace!("enter receiving not NotReady");
                            return Ok(Async::NotReady);
                        }
                        Err(e) => {
                            trace!("enter receiving err");
                            return Err(e.into());
                        }
                    }
                }
                BrowserState::StartSend(message_to_send) => {
                    trace!("enter start send.");
                    match self
                        .ws_client
                        .as_mut()
                        .unwrap()
                        .start_send(OwnedMessage::Text(message_to_send.clone()))
                    {
                        Ok(AsyncSink::Ready) => {
                            self.state = BrowserState::Sending;
                        }
                        Ok(AsyncSink::NotReady(_)) => {
                            return Ok(Async::NotReady);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                BrowserState::Sending => {
                    trace!("enter sending.");
                    match self.ws_client.as_mut().unwrap().poll_complete() {
                        Ok(Async::Ready(_)) => {
                            trace!("switch to receiving state.");
                            if let Some(first) = self.waiting_to_send.pop_front() {
                                self.state = BrowserState::StartSend(first);
                            } else {
                                self.state = BrowserState::Receiving;
                            }
                        }
                        Ok(Async::NotReady) => {
                            trace!("sending not ready.");
                            return Ok(Async::NotReady);
                        }
                        Err(e) => {
                            error!("{:?}", e);
                            return Err(e.into());
                        }
                    }
                }
            }
        }
    }
}

