use crate::protocol;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult,
};
use crate::protocol::target;
use websocket::futures::{Async, Future, Poll, Stream};
use log::*;

#[derive(Debug)]
enum OnePageState {
    WaitingPageCreate,
    PageCreated,
    WaitingPageAttach,
    PageAttached,
}

pub struct OnePage {
    stream: ChromeBrowser,
    state: OnePageState,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
}

impl OnePage {
    fn new(stream: ChromeBrowser) -> Self {
        Self { stream, state: OnePageState::WaitingPageCreate, target_info: None, session_id: None }
    }

    pub fn create_attach_method(&mut self) -> MethodBeforSendResult {
        MethodUtil::create_msg_to_send(target::methods::AttachToTarget {
                target_id: &(self.target_info.as_mut().unwrap().target_id),
                flatten: None,
            },
            MethodDestination::Browser, None)
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
                        _ => {
                            trace!("receive message: {:?}", value);
                        },
                    }
                },
                None => (),
            };
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

        let fib = ChromeBrowser::new();
        let display = OnePage::new(fib);

        tokio::run(display.map_err(|_| ()));

        // let mut rt = Runtime::new().unwrap();
        // rt.shutdown_on_idle().wait().unwrap();
    }
}
