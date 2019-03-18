use crate::protocol;
use crate::protocol::page;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult,
};
use crate::protocol::page::methods::{Navigate};
use crate::protocol::target;
use websocket::futures::{Async, Future, Poll, Stream};
use log::*;

#[derive(Debug)]
enum OnePageState {
    WaitingPageCreate,
    PageCreated,
    WaitingPageAttach,
    PageAttached,
    Navigating,
    WaitingPageLoadEvent,
    Steady,
}

pub struct OnePage {
    stream: ChromeBrowser,
    state: OnePageState,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    entry_url: &'static str,
}

impl OnePage {
    fn new(stream: ChromeBrowser, entry_url: &'static str) -> Self {
        Self { stream, state: OnePageState::WaitingPageCreate, target_info: None, session_id: None, entry_url: entry_url }
    }

    pub fn create_attach_method(&mut self) -> MethodBeforSendResult {
        MethodUtil::create_msg_to_send(target::methods::AttachToTarget {
                target_id: &(self.target_info.as_mut().unwrap().target_id),
                flatten: None,
            },
            MethodDestination::Browser, None)
    }

    fn create_msg_to_send_with_session_id<C>(&mut self, method: C) -> MethodBeforSendResult
    where
        C: protocol::Method + serde::Serialize,
    {   
        let session_id = self.session_id.as_mut().unwrap();
        MethodUtil::create_msg_to_send(method, MethodDestination::Target(session_id.clone().into()), None)
    }

    pub fn navigate_to(&mut self, url: &str) -> MethodBeforSendResult {
        let c = Navigate { url };
        self.create_msg_to_send_with_session_id(c)
    }
}

impl Future for OnePage {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            match try_ready!(self.stream.poll()) {
                Some(value) => {
                    match &mut self.state {
                        OnePageState::WaitingPageCreate => {
                            if let Some(target_info) = MethodUtil::is_page_event_create(value) {
                                self.target_info = Some(target_info);
                                self.state = OnePageState::PageCreated;
                            }
                        },
                        OnePageState::PageCreated => {
                            let (_, method_str, _) = self.create_attach_method().unwrap();
                            self.stream.send_message(method_str);
                            self.state = OnePageState::WaitingPageAttach;
                        },
                        OnePageState::WaitingPageAttach => {
                            if let Some((session_id, target_info)) = MethodUtil::is_page_attach_event(value) {
                                self.session_id = Some(session_id);
                                self.target_info = Some(target_info);
                                self.state = OnePageState::PageAttached;
                            }
                        }
                        OnePageState::PageAttached => {
                            let (_, method_str, _) = self.create_msg_to_send_with_session_id(page::methods::Enable {}).unwrap();
                            self.stream.send_message(method_str);
                            self.state = OnePageState::Navigating;
                        }
                        OnePageState::Navigating => {
                            let (_, method_str, _) = self.navigate_to(self.entry_url).unwrap();
                            self.stream.send_message(method_str);
                            self.state = OnePageState::WaitingPageLoadEvent;
                        }
                        OnePageState::WaitingPageLoadEvent => {
                            if let Some(receive_message_from_target_params) = MethodUtil::is_page_load_event_fired(value) {
                                if (receive_message_from_target_params.target_id == self.target_info.as_mut().unwrap().target_id) && (receive_message_from_target_params.session_id == *self.session_id.as_mut().unwrap()) {
                                    self.state = OnePageState::Steady;
                                } else {
                                    info!("unequal session_id or target_id.");
                                }
                            }
                        }
                        _ => {
                            trace!("receive message: {:?}", value);
                        },
                    }
                },
                None => (),
            };
        }
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

    const ENTERY: &'static str = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    #[test]
    fn t_by_enum() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();

        let fib = ChromeBrowser::new();
        let display = OnePage::new(fib, ENTERY);

        tokio::run(display.map_err(|_| ()));

        // let mut rt = Runtime::new().unwrap();
        // rt.shutdown_on_idle().wait().unwrap();
    }
}
