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
    WaitingDescribeNode(Option<String>, usize),
    Consuming,
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
    pub fn new(stream: ChromeBrowser, entry_url: &'static str) -> Self {
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

    pub fn describe_node(&mut self, selector: Option<String>, node_id: dom::NodeId) {
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(dom::methods::DescribeNode {
                node_id: Some(node_id),
                backend_node_id: None,
                depth: Some(100),
            }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingDescribeNode(selector, mid.unwrap());
    }
}


#[derive(Debug)]
pub enum PageMessage {
    DocumentAvailable,
    FindNode(Option<String>, dom::Node),
    MessageAvailable(protocol::Message),
} 

impl Stream for OnePage {
    type Item = PageMessage;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
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
                                    return Ok(Async::Ready(Some(PageMessage::DocumentAvailable)));
                                } else {
                                    return Err(ChromePageError::NoRootNode.into());
                                }
                            }
                        }
                        OnePageState::WaitingNode(selector, mid) => {
                            if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                                let selector_cloned = Some(selector.clone());
                                // let backend_node_id = self.describe_node(node_id)?.backend_node_id;
                                if let Ok(v)  = protocol::parse_response::<dom::methods::QuerySelectorReturnObject>(resp) {
                                    self.describe_node(selector_cloned, v.node_id);
                                }
                            }
                        }
                        OnePageState::WaitingDescribeNode(maybe_selector, mid) => {
                            if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                                 trace!("----------got describe Node resp: {:?}", resp);
                                // let backend_node_id = self.describe_node(node_id)?.backend_node_id;
                                if let Ok(v)  = protocol::parse_response::<dom::methods::DescribeNodeReturnObject>(resp) {
                                    trace!("----------got describe Node: {:?}", v.node);
                                    let maybe_selector_cloned = maybe_selector.clone();
                                    self.state = OnePageState::Consuming;
                                    return Ok(Async::Ready(Some(PageMessage::FindNode(maybe_selector_cloned, v.node))));
                                }
                            }
                        }
                        _ => {
                            trace!("receive message: {:?}", value);
                            return Ok(Async::Ready(Some(PageMessage::MessageAvailable(value))));
                        },
                    }
                } else {
                    error!("got None, was stream ended?");
                }
            }
    }
}