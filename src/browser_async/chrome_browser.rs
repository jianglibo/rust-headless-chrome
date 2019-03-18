use crate::protocol;

use failure;
use log::*;


pub use crate::protocol::browser::methods::VersionInformationReturnObject;

pub use crate::browser::process::LaunchOptionsBuilder;
use crate::browser::process::{Process};
pub use crate::browser::tab::Tab;
use futures::AsyncSink;
use websocket;
use websocket::futures::{Async, Future, Poll, Sink, Stream};
use websocket::message::OwnedMessage;
use websocket::r#async::client::{Client, ClientNew};
use websocket::r#async::TcpStream;
use websocket::ClientBuilder;

use crate::protocol::target;

use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination,
};

type WsClient = Client<TcpStream>;

enum BrowserState {
    Unconnected,
    Connecting(ClientNew<TcpStream>),
    EnableDiscover,
    Receiving,
    StartSend(String),
    Sending,
}

pub struct ChromeBrowser {
    state: BrowserState,
    ws_client: Option<WsClient>,
    process: Option<Process>,
}

impl ChromeBrowser {
    pub fn new() -> Self {
        Self {
            state: BrowserState::Unconnected,
            ws_client: None,
            process: None,
        }
    }
    pub fn send_message(&mut self, md: String) {
        self.state = BrowserState::StartSend(md);
    }
}

impl Stream for ChromeBrowser {
    type Item = protocol::Message;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
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
                    self.state = BrowserState::EnableDiscover;
                }
                BrowserState::EnableDiscover => {
                    trace!("enter enable discover state.");
                    let (_, md, _) = MethodUtil::create_msg_to_send(target::methods::SetDiscoverTargets { discover: true }, MethodDestination::Browser,None).unwrap();
                    self.state = BrowserState::StartSend(md);
                }
                BrowserState::Receiving => {
                    if let Ok(Async::Ready(Some(message))) = self.ws_client.as_mut().unwrap().poll() {
                        if let OwnedMessage::Text(msg) = message {
                            let parsed_message = protocol::parse_raw_message(&msg);
                            return Ok(Async::Ready(Some(parsed_message.unwrap())));
                        } else {
                            error!("got unknown message: {:?}", message);
                        }
                    }
                },
                BrowserState::StartSend(message_to_send) => {
                    trace!("enter start send.");
                    match self.ws_client.as_mut().unwrap().start_send(OwnedMessage::Text(message_to_send.clone())) {
                        Ok(AsyncSink::Ready) => {
                            self.state = BrowserState::Sending;
                        },
                        Ok(AsyncSink::NotReady(_)) => {
                            return Ok(Async::NotReady);
                        },
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                },
                BrowserState::Sending => {
                    trace!("enter sending.");
                    match self.ws_client.as_mut().unwrap().poll_complete() {
                        Ok(Async::Ready(_)) => {
                            info!("swith to receiving state.");
                            self.state = BrowserState::Receiving;
                        },
                        Ok(Async::NotReady) => {
                            info!("sending not ready.");
                            return Ok(Async::NotReady);
                        },
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

