use crate::protocol;

use failure;
use log::*;
use std::fmt;


pub use crate::protocol::browser::methods::VersionInformationReturnObject;

pub use crate::browser::process::{LaunchOptionsBuilder, LaunchOptions};
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
    Connecting(ClientNew<TcpStream>),
    Receiving,
    StartSend(String),
    Sending,
    // CheckPendingTask,
}

impl fmt::Debug for BrowserState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BrowserState::Connecting(_) => write!(f, "connecting"),
            BrowserState::Receiving => write!(f, "Receiving"),
            BrowserState::StartSend(content) => write!(f, "start sending: {}", content),
            BrowserState::Sending => write!(f, "Sending"),
        }
    }
}

pub struct ChromeBrowser {
    // connected: bool,
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

impl ChromeBrowser {
    pub fn new_visible() -> Self {
        let options = LaunchOptionsBuilder::default().headless(false).build().expect("LaunchOptionsBuilder should success.");
        Self::new(options)
    }
    pub fn new(launch_options: LaunchOptions) -> Self {
        let chrome_process = Process::new(launch_options).expect("process should created.");
        let web_socket_debugger_url = chrome_process.debug_ws_url.clone();
        let process = Some(chrome_process);
        let state = BrowserState::Connecting(
            ClientBuilder::new(&web_socket_debugger_url)
                .expect("client build should work.")
                .add_protocol("rust-websocket")
                .async_connect_insecure(),
        );

        Self {
            // connected: false,
            state,
            ws_client: None,
            process,
            last_be_polled: Instant::now(),
            waiting_to_send: VecDeque::new(),
        }
    }
    pub fn send_message(&mut self, method_str: String) {
        self.waiting_to_send.push_back(method_str);
    }

    pub fn check_pending_task(&mut self) {
        if let BrowserState::Receiving = self.state {
            trace!("enter check_pending_task state..........");
            if let Some(first) = self.waiting_to_send.pop_front() {
                trace!("take from waiting_to_send: {:?}", first);
                self.state = BrowserState::StartSend(first);
            } else {
                trace!("no pending task.");
            }
        }
        // if let Some(first) = self.waiting_to_send.pop_front() {
        //     trace!("take from waiting_to_send: {:?}", first);
        //     self.state = BrowserState::StartSend(first);
        // } else {
        //     trace!("no waiting task, switch to receiving state.");
        //     self.state = BrowserState::Receiving;
        // }
    }

    pub fn have_not_be_polled_for(&self, duration: Duration) -> bool {
        (self.last_be_polled - Instant::now()) > duration
    }
}

impl Stream for ChromeBrowser {
    type Item = protocol::Message;
    type Error = failure::Error;

    // this poll will be polled by out loop until not_ready fired.
    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.last_be_polled = Instant::now();
        // if self.connected {
            self.check_pending_task();
        // }
        loop {
            // trace!("browser loop {:?}", self.state);
            match &mut self.state {
                BrowserState::Connecting(client_new) => {
                    trace!("enter connecting state.");
                    let framed = try_ready!(client_new.poll());
                    info!("connected.");
                    self.ws_client = Some(framed.0);
                    self.state = BrowserState::Receiving;
                    // self.connected = true;
                    return Ok(Some(protocol::Message::Connected).into());
                }
                BrowserState::Receiving => {
                    match self.ws_client.as_mut().expect("obtain ws_client should success.").poll() {
                        Ok(Async::Ready(Some(message))) => {
                            if let OwnedMessage::Text(msg) = message {
                                if msg.contains("Network.requestIntercepted") || msg.len() < 1000 {
                                    trace!("got message (***every message***): {:?}", msg);
                                } else {
                                    let (short, _) = msg.split_at(1000);
                                    trace!("got message (***every message***): {:?}", short);
                                } 
                                let parsed_message = protocol::parse_raw_message(&msg);
                                match parsed_message {
                                    Ok(success_parsed_message) => {
                                        return Ok(Async::Ready(Some(success_parsed_message)));
                                    }
                                    Err(err) => {
                                        error!("parse message failed: {:?}", err);
                                    }
                                }
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
                            trace!("enter receiving not NotReady, the control hand over to tokio system. ");
                            return Ok(Async::NotReady);
                        }
                        Err(e) => {
                            trace!("enter receiving err");
                            return Err(e.into());
                        }
                    }
                }
                BrowserState::StartSend(message_to_send) => {
                    trace!("enter start_send state..........");
                    match self
                        .ws_client
                        .as_mut()
                        .expect("obtain ws_client should success.")
                        .start_send(OwnedMessage::Text(message_to_send.clone()))
                    {
                        Ok(AsyncSink::Ready) => {
                            trace!("entered start send. switching to **sending: {:?}", message_to_send);
                            self.state = BrowserState::Sending;
                        }
                        Ok(AsyncSink::NotReady(_)) => {
                            error!("StartSend doesn't ready yet.");
                            return Ok(Async::NotReady);
                        }
                        Err(e) => {
                            error!("StartSend error: {:?}", e);
                            return Err(e.into());
                        }
                    }
                }
                BrowserState::Sending => {
                    trace!("enter sending state..........");
                    match self.ws_client.as_mut().expect("obtain ws_client should success.").poll_complete() {
                        Ok(Async::Ready(v)) => {
                            trace!("sending {:?} completed. switch from sending to receiving state.", v);
                            self.state = BrowserState::Receiving;
                        }
                        Ok(Async::NotReady) => {
                            trace!("sending is not completed.");
                            return Ok(Async::NotReady);
                        }
                        Err(e) => {
                            error!("sending error: {:?}", e);
                            return Err(e.into());
                        }
                    }
                }
            }
        }
    }
}

