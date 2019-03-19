use crate::protocol;
use crate::protocol::page;
use crate::protocol::dom;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};
use crate::protocol::page::methods::{Navigate};
use crate::protocol::target;
use websocket::futures::{Async, Future, Poll, Stream};
use log::*;

#[derive(Debug)]
enum OnePageState {
    WaitingPageCreate,
    WaitingPageAttach,
    WaitingPageEnable(usize),
    WaitingPageLoadEvent,
    WaitingGetDocument(usize),
    WaitingNode(String, usize),
    Steady,
}

pub struct OnePage {
    stream: ChromeBrowser,
    state: OnePageState,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    entry_url: &'static str,
    root_node: Option<dom::Node>,
}

impl OnePage {
    fn new(stream: ChromeBrowser, entry_url: &'static str) -> Self {
        Self { stream, state: OnePageState::WaitingPageCreate, target_info: None, session_id: None, entry_url: entry_url, root_node: None }
    }

    pub fn attach_to_page(&mut self) {
        let (_, method_str, _) = MethodUtil::create_msg_to_send(target::methods::AttachToTarget {
                target_id: &(self.target_info.as_mut().unwrap().target_id),
                flatten: None,
            },
            MethodDestination::Browser, None).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingPageAttach;
    }

    fn create_msg_to_send_with_session_id<C>(&self, method: C) -> MethodBeforSendResult
    where
        C: protocol::Method + serde::Serialize,
    {   
        let session_id = self.session_id.as_ref().unwrap();
        MethodUtil::create_msg_to_send(method, MethodDestination::Target(session_id.clone().into()), None)
    }

    pub fn page_enable(&mut self) {
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(page::methods::Enable {}).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingPageEnable(mid.unwrap());
    }

    pub fn navigate_to(&mut self, url: &str) {
        let (_, method_str, _) = self.create_msg_to_send_with_session_id(Navigate { url }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingPageLoadEvent;
    }

    pub fn get_document(&mut self) {
        let (_, method_str, mid) =  self.create_msg_to_send_with_session_id(dom::methods::GetDocument {
            depth: Some(0),
            pierce: Some(false),
        }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingGetDocument(mid.unwrap());
    }

    fn wait_page_load_event_fired(&mut self, value: protocol::Message) {
        if let Some(receive_message_from_target_params) = MethodUtil::is_page_load_event_fired(value) {
            if (receive_message_from_target_params.target_id == self.target_info.as_mut().unwrap().target_id) && (receive_message_from_target_params.session_id == *self.session_id.as_mut().unwrap()) {
                self.get_document();
            } else {
                info!("unequal session_id or target_id.");
            }
        }
    }

    pub fn find_node<'a>(&'a mut self, selector: &'a str) {
        let rn = self.root_node.as_ref().unwrap();
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(dom::methods::QuerySelector {
            node_id: rn.node_id,
            selector,
        }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingNode(selector.to_string(), mid.unwrap());
    }
}

impl Future for OnePage {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.stream.poll()) {
                    match &mut self.state {
                        OnePageState::WaitingPageCreate => {
                            if let Some(target_info) = MethodUtil::is_page_event_create(value) {
                                self.target_info = Some(target_info);
                                self.attach_to_page();
                            }
                        },
                        OnePageState::WaitingPageAttach => {
                            if let Some((session_id, target_info)) = MethodUtil::is_page_attach_event(value) {
                                self.session_id = Some(session_id);
                                self.target_info = Some(target_info);
                                self.page_enable();
                            }
                        }
                        OnePageState::WaitingPageEnable(mid) => {
                            if MethodUtil::match_chrome_response(value, mid).is_some() {
                                self.navigate_to(self.entry_url);
                            }
                        }
                        OnePageState::WaitingPageLoadEvent => {
                            self.wait_page_load_event_fired(value);
                        }
                        OnePageState::WaitingGetDocument(mid) => {
                            if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                                if let Ok(c) =
                                    protocol::parse_response::<dom::methods::GetDocumentReturnObject>(resp)
                                {
                                    info!("got document Node: {:?}", c.root);
                                    self.root_node = Some(c.root);
                                    self.find_node("#ddlogin");
                                } else {
                                    return Err(ChromePageError::NoRootNode.into());
                                }
                            }
                        }
                        OnePageState::WaitingNode(selector, mid) => {
                            if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                                info!("----------got ddlogin Node: {:?}", resp);
                                self.state = OnePageState::Steady;
                            }
                            // match selector {
                            //     "#ddlogin" => {
                            //         info!("----------got ddlogin Node: {:?}", value);
                            //     }
                            // }
                        }
                        _ => {
                            trace!("receive message: {:?}", value);
                        },
                    }
                } else {
                    error!("got None, was stream ended?");
                }
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
