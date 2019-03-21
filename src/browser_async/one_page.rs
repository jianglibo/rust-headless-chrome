use crate::protocol;
use crate::protocol::page;
use crate::protocol::dom;
use crate::protocol::input;
use std::fmt;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};
use crate::protocol::page::methods::{Navigate};
use crate::protocol::target;
use crate::browser_async::element_async::{Element};
use websocket::futures::{Async, Future, Poll, Stream};
use crate::browser_async::point_async::{Point};
use crate::browser_async::keys_async;
use log::*;

#[derive(Debug)]
enum OnePageState<'a> {
    WaitingPageCreate,
    WaitingPageAttach,
    WaitingPageEnable(usize),
    WaitingPageLoadEvent,
    WaitingGetDocument(usize),
    WaitingNode(&'a str, usize),
    WaitingDescribeNode(Option<&'a str>, usize),
    WaitingRemoteObject(dom::NodeId, &'a str, usize),
    Consuming,
}

pub struct OnePage<'a> {
    stream: &'a mut ChromeBrowser,
    state: OnePageState<'a>,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    entry_url: &'a str,
    root_node: Option<dom::Node>,
}

impl<'a> OnePage<'a> {
    pub fn new(stream: &'a mut ChromeBrowser, entry_url: &'a str) -> Self {
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

    pub fn find_node(&mut self, selector: &'a str) {
        let rn = self.root_node.as_ref().unwrap();
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(dom::methods::QuerySelector {
            node_id: rn.node_id,
            selector,
        }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingNode(selector, mid.unwrap());
    }

    pub fn find_element(&mut self, selector: &'a str, backend_node_id: dom::NodeId) {
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(dom::methods::ResolveNode {
            backend_node_id: Some(backend_node_id),
        }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingRemoteObject(backend_node_id, selector, mid.unwrap());
    }

    pub fn describe_node(&mut self, selector: Option<&'a str>, node_id: dom::NodeId) {
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(dom::methods::DescribeNode {
                node_id: Some(node_id),
                backend_node_id: None,
                depth: Some(100),
            }).unwrap();
        self.stream.send_message(method_str);
        self.state = OnePageState::WaitingDescribeNode(selector, mid.unwrap());
    }

    pub fn type_str(&mut self, string_to_type: &str) {
        for c in string_to_type.split("") {
            // split call above will have empty string at start and end which we won't type
            if c == "" {
                continue;
            }
            self.press_key(c);
        }
    }

    pub fn press_key(&mut self, key: &str) {
        let definition = keys_async::get_key_definition(key).unwrap();
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(input::methods::DispatchKeyEvent {
            event_type: "keyDown",
            key: definition.key,
            text: definition.text,
        }).unwrap();
        self.stream.send_message(method_str);

        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(input::methods::DispatchKeyEvent {
            event_type: "keyUp",
            key: definition.key,
            text: definition.text,
        }).unwrap();
        self.stream.send_message(method_str);
    }

    pub fn click_point(&mut self, point: Point) {
        trace!("Clicking point: {:?}", point);
        if point.x == 0.0 && point.y == 0.0 {
            warn!("Midpoint of element shouldn't be 0,0. Something is probably wrong.")
        }

        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
            event_type: "mouseMoved",
            x: point.x,
            y: point.y,
            ..Default::default()
        }).unwrap();
        self.stream.send_message(method_str);


        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
            event_type: "mousePressed",
            x: point.x,
            y: point.y,
            button: Some("left"),
            click_count: Some(1),
        }).unwrap();
        self.stream.send_message(method_str);

        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
            event_type: "mouseReleased",
            x: point.x,
            y: point.y,
            button: Some("left"),
            click_count: Some(1),
        }).unwrap();
        self.stream.send_message(method_str);
    }

    pub fn request_midpoint(&mut self, backend_node_id: dom::NodeId) {
        let (_, method_str, mid) = self.create_msg_to_send_with_session_id(dom::methods::GetContentQuads {
                node_id: None,
                backend_node_id: Some(backend_node_id),
                object_id: None,
            }).unwrap();
            self.stream.send_message(method_str);

            // let return_object = self.parent.call_method()?;
            // let raw_quad = return_object.quads.first().unwrap();
            // let input_quad = ElementQuad::from_raw_points(&raw_quad);

            // Ok((input_quad.bottom_right + input_quad.top_left) / 2.0)
        }
}

pub enum PageMessage<'a> {
    DocumentAvailable,
    FindNode(Option<&'a str>, dom::Node),
    FindElement(Element<'a>),
    MessageAvailable(protocol::Message),
}

impl<'a> fmt::Debug for PageMessage<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PageMessage::FindElement(ele) => {
                write!(f, "remote_object_id: {}, backend_node_id: {},found_via_selector: {}", ele.remote_object_id, ele.backend_node_id, ele.found_via_selector)
            }
            _ => write!(f, "{:?}", self)
        }
        
    }
}

impl<'a> Stream for OnePage<'a> {
    type Item = PageMessage<'a>;
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
                                    // self.state = OnePageState::Consuming;
                                    self.find_element(maybe_selector_cloned.clone().unwrap(), v.node.backend_node_id);
                                    return Ok(Async::Ready(Some(PageMessage::FindNode(maybe_selector_cloned, v.node))));
                                }
                            }
                        }
                        OnePageState::WaitingRemoteObject(backend_node_id, selector, mid) => {
                            if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                                if let Ok(v)  = protocol::parse_response::<dom::methods::ResolveNodeReturnObject>(resp) {
                                    self.state = OnePageState::Consuming;
                                    
                                        let element = Element {
                                            remote_object_id: v.object.object_id.unwrap().clone(),
                                            backend_node_id: *backend_node_id,
                                            parent: self,
                                            found_via_selector: selector,
                                            };
                                    return Ok(Async::Ready(Some(PageMessage::FindElement(element))));
                                    
                                }
                                // info!("got remote object: {:?}", resp);
                                self.state = OnePageState::Consuming;
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