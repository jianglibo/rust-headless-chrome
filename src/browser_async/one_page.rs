use crate::browser::tab::keys;
use crate::browser_async::chrome_browser::ChromeBrowser;
use crate::browser_async::dev_tools_method_util::{
    ChromePageError, MethodBeforSendResult, MethodDestination, MethodUtil,
};
use super::element_async::{BoxModel, Element, ElementQuad};
use crate::browser_async::point_async::Point;
use crate::protocol::{self, dom, input, page, page::methods::Navigate, target};
use log::*;
use websocket::futures::{Async, Future, Poll, Stream};
// use tokio::timer::{Interval, Timeout};
// use std::time::{Duration, Instant};
use failure::{Error, Fail};
use super::page_message::{PageMessage};

#[derive(Debug, Fail)]
#[fail(display = "The event waited for never came")]
pub struct WaitTimeout;

impl std::convert::From<tokio_timer::timeout::Error<Error>> for WaitTimeout {
    fn from(_: tokio_timer::timeout::Error<Error>) -> Self {
        Self {}
    }
}


#[derive(Debug)]
pub enum OnePageState {
    WaitingPageCreate,
    WaitingPageAttach,
    WaitingPageEnable(usize),
    WaitingFrameTree(usize),
    AfterInvokeNavigate,
    // WaitingPageLoadEvent,
    WaitingGetDocument(usize, Option<&'static str>),
    WaitingNode(Option<&'static str>, usize),
    WaitingDescribeNode(Option<&'static str>, usize),
    WaitingRemoteObject(dom::NodeId, Option<&'static str>, usize),
    WaitingModelBox(Option<&'static str>, dom::NodeId, usize),
    WaitingScreenShot(usize),
    Consuming,
}

pub struct OnePage {
    chrome_browser: ChromeBrowser,
    pub state: OnePageState,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    // entry_url: &'static str,
    root_node: Option<dom::Node>,
    expect_page_message: PageMessage,
}

impl OnePage {
    pub fn new(chrome_browser: ChromeBrowser/*, entry_url: &'static str*/) -> Self {
        Self {
            chrome_browser,
            state: OnePageState::WaitingPageCreate,
            target_info: None,
            session_id: None,
            // entry_url: entry_url,
            root_node: None,
            expect_page_message: PageMessage::DocumentAvailable,
        }
    }

    pub fn force_next_stage(&mut self) {

    }

    pub fn attach_to_page(&mut self) {
        let (_, method_str, _) = MethodUtil::create_msg_to_send(
            target::methods::AttachToTarget {
                target_id: &(self.target_info.as_mut().unwrap().target_id),
                flatten: None,
            },
            MethodDestination::Browser,
            None,
        )
        .unwrap();
        self.state = OnePageState::WaitingPageAttach; // change state first.
        self.chrome_browser.send_message(method_str);
    }

    fn create_msg_to_send_with_session_id<C>(&self, method: C) -> MethodBeforSendResult
    where
        C: protocol::Method + serde::Serialize,
    {
        let session_id = self.session_id.as_ref().unwrap();
        MethodUtil::create_msg_to_send(
            method,
            MethodDestination::Target(session_id.clone().into()),
            None,
        )
    }

    pub fn page_enable(&mut self) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(page::methods::Enable {})
            .unwrap();
        self.state = OnePageState::WaitingPageEnable(mid.unwrap());
        self.chrome_browser.send_message(method_str);
    }

    pub fn navigate_to(&mut self, url: &str) {
        let (_, method_str, _) = self
            .create_msg_to_send_with_session_id(Navigate { url })
            .unwrap();
        self.state = OnePageState::AfterInvokeNavigate;
        self.chrome_browser.send_message(method_str);
    }

    pub fn get_document(&mut self, then_find_node: Option<&'static str>) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::GetDocument {
                depth: Some(0),
                pierce: Some(false),
            })
            .unwrap();
        self.state = OnePageState::WaitingGetDocument(mid.unwrap(), then_find_node);
        self.chrome_browser.send_message(method_str);
    }

    // fn wait_page_load_event_fired(&mut self, value: protocol::Message) {
    //     if let Some(receive_message_from_target_params) =
    //         MethodUtil::is_page_load_event_fired(value)
    //     {
    //         if (receive_message_from_target_params.target_id
    //             == self.target_info.as_mut().unwrap().target_id)
    //             && (receive_message_from_target_params.session_id
    //                 == *self.session_id.as_mut().unwrap())
    //         {
    //             self.get_document();
    //         } else {
    //             info!("unequal session_id or target_id.");
    //         }
    //     } else {
    //         info!("isn't is_page_load_event_fired.");
    //     }
    // }

    pub fn get_box_model(&mut self, selector: Option<&'static str>, element: &Element) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::GetBoxModel {
                node_id: None,
                backend_node_id: Some(element.backend_node_id),
                object_id: None,
            })
            .unwrap();

        self.state = OnePageState::WaitingModelBox(selector, element.backend_node_id, mid.unwrap());
        self.chrome_browser.send_message(method_str);
    }

    pub fn find_node(&mut self, selector: &'static str) {
        if self.root_node.is_some() {
            let rn = self.root_node.as_ref().unwrap();
            let (_, method_str, mid) = self
                .create_msg_to_send_with_session_id(dom::methods::QuerySelector {
                    node_id: rn.node_id,
                    selector,
                })
                .unwrap();
            self.state = OnePageState::WaitingNode(Some(selector), mid.unwrap());
            self.chrome_browser.send_message(method_str);
        } else {
            self.get_document(Some(selector));
        }
    }

    pub fn get_frame_tree(&mut self) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(page::methods::GetFrameTree {})
            .unwrap();
        self.state = OnePageState::WaitingFrameTree(mid.unwrap());
        self.chrome_browser.send_message(method_str);
    }

    pub fn find_element(&mut self, selector: Option<&'static str>, backend_node_id: dom::NodeId) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::ResolveNode {
                backend_node_id: Some(backend_node_id),
            })
            .unwrap();
        self.state = OnePageState::WaitingRemoteObject(backend_node_id, selector, mid.unwrap());
        self.chrome_browser.send_message(method_str);
    }

    pub fn describe_node(&mut self, selector: Option<&'static str>, node_id: dom::NodeId) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::DescribeNode {
                node_id: Some(node_id),
                backend_node_id: None,
                depth: Some(100),
            })
            .unwrap();
        self.state = OnePageState::WaitingDescribeNode(selector, mid.unwrap());
        self.chrome_browser.send_message(method_str);
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
        let definition = keys::get_key_definition(key).unwrap();

        // See https://github.com/GoogleChrome/puppeteer/blob/62da2366c65b335751896afbb0206f23c61436f1/lib/Input.js#L114-L115
        let text = definition.text.or_else(|| {
            if definition.key.len() == 1 {
                Some(definition.key)
            } else {
                None
            }
        });

        // See https://github.com/GoogleChrome/puppeteer/blob/62da2366c65b335751896afbb0206f23c61436f1/lib/Input.js#L52
        let key_down_event_type = if text.is_some() {
            "keyDown"
        } else {
            "rawKeyDown"
        };

        let key = Some(definition.key);
        let code = Some(definition.code);

        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(input::methods::DispatchKeyEvent {
                event_type: key_down_event_type,
                key,
                text,
                code: Some(definition.code),
                windows_virtual_key_code: definition.key_code,
                native_virtual_key_code: definition.key_code,
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);

        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(input::methods::DispatchKeyEvent {
                event_type: "keyUp",
                key,
                text,
                code,
                windows_virtual_key_code: definition.key_code,
                native_virtual_key_code: definition.key_code,
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);
    }

    pub fn click_point(&mut self, point: Point) {
        trace!("Clicking point: {:?}", point);
        if point.x == 0.0 && point.y == 0.0 {
            warn!("Midpoint of element shouldn't be 0,0. Something is probably wrong.")
        }

        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
                event_type: "mouseMoved",
                x: point.x,
                y: point.y,
                ..Default::default()
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);

        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
                event_type: "mousePressed",
                x: point.x,
                y: point.y,
                button: Some("left"),
                click_count: Some(1),
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);

        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
                event_type: "mouseReleased",
                x: point.x,
                y: point.y,
                button: Some("left"),
                click_count: Some(1),
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);
    }

    pub fn request_midpoint(&mut self, backend_node_id: dom::NodeId) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::GetContentQuads {
                node_id: None,
                backend_node_id: Some(backend_node_id),
                object_id: None,
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);

        // let return_object = self.parent.call_method()?;
        // let raw_quad = return_object.quads.first().unwrap();
        // let input_quad = ElementQuad::from_raw_points(&raw_quad);
        // Ok((input_quad.bottom_right + input_quad.top_left) / 2.0)
    }

    pub fn capture_screenshot_by_selector(
        &mut self,
        selector: &'static str,
        format: page::ScreenshotFormat,
        from_surface: bool,
    ) {
        self.expect_page_message =
            PageMessage::Screenshot(Some(selector), format, from_surface, None);
        self.find_node(selector);
    }

    pub fn capture_screenshot(
        &mut self,
        format: page::ScreenshotFormat,
        clip: Option<page::Viewport>,
        from_surface: bool,
    ) /*-> Result<Vec<u8>, Error>*/
    {
        let (format, quality) = match format {
            page::ScreenshotFormat::JPEG(quality) => {
                (page::InternalScreenshotFormat::JPEG, quality)
            }
            page::ScreenshotFormat::PNG => (page::InternalScreenshotFormat::PNG, None),
        };

        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(page::methods::CaptureScreenshot {
                format,
                clip,
                quality,
                from_surface,
            })
            .unwrap();
        self.state = OnePageState::WaitingScreenShot(mid.unwrap());
        self.chrome_browser.send_message(method_str);
    }
}


// The main loop should stop at some point, by invoking the methods on the page to drive the loop to run.
impl Stream for OnePage {
    type Item = PageMessage;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.chrome_browser.poll()) {
                match &mut self.state {
                    OnePageState::WaitingPageCreate => {
                        info!("*** WaitingPageCreate ***");
                        if let Some(target_info) = MethodUtil::is_page_event_create(value) {
                            self.target_info = Some(target_info);
                            self.attach_to_page();
                        }
                    }
                    OnePageState::WaitingPageAttach => {
                        info!("*** WaitingPageAttach ***");
                        if let Some((session_id, target_info)) =
                            MethodUtil::is_page_attach_event(value)
                        {
                            self.session_id = Some(session_id);
                            self.target_info = Some(target_info);
                            self.page_enable();
                        }
                    }
                    OnePageState::WaitingPageEnable(mid) => {
                        info!("*** WaitingPageEnable ***");
                        if MethodUtil::match_chrome_response(value, mid).is_some() {
                            return Ok(Some(PageMessage::EnablePageDone).into());
                        }
                    }
                    OnePageState::AfterInvokeNavigate => {
                        info!("*** AfterInvokeNavigate ***");
                        if let Some(mg) = MethodUtil::is_received_message_from_target_event(&value) {
                            if let Ok(inner_message) = protocol::parse_raw_message(&mg.message) {
                                if let protocol::Message::Event(inner_event) = inner_message {
                                    match inner_event {
                                        protocol::Event::FrameNavigated(frame_navigated_event) => return Ok(Some(PageMessage::FrameNavigatedEvent((&mg.session_id).clone(), (&mg.target_id).clone(), frame_navigated_event)).into()),
                                        protocol::Event::TargetInfoChanged(target_info_changed) => return Ok(Some(PageMessage::TargetInfoChanged(target_info_changed)).into()),
                                        _ => {
                                            info!("inner event: {:?}", inner_event);
                                        }
                                    }
                                }
                            } else {
                                info!("parse inner event failure: {:?}", value);
                            }

                            // if let Ok(inner_msg) = MethodUtil::parse_target_message_event(&mg.message) {
                            //     // match inner_msg {
                            //         info!("json value: {:?}", inner_msg);
                            //         // _ => ()
                            //     // }
                            // } else {
                            //     info!("{:?}", value);
                            // }
                        } else {
                            info!("{:?}", value);
                        }
                    }
                    OnePageState::WaitingFrameTree(mid) => {
                        info!("*** WaitingFrameTree {:?} ***", mid);
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(v) = protocol::parse_response::<
                                page::methods::GetFrameTreeReturnObject,
                            >(resp)
                            {
                                info!("----------------- got frames: {:?}", v);
                            }
                        }
                    }
                    OnePageState::WaitingGetDocument(mid, ref next_find_node) => {
                        info!("*** WaitingGetDocument ***");
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(c) = protocol::parse_response::<
                                dom::methods::GetDocumentReturnObject,
                            >(resp)
                            {
                                info!("got document Node: {:?}", c.root);
                                self.root_node = Some(c.root);
                                if let Some(selector) = next_find_node.as_ref() {
                                    let s = *selector;
                                    self.find_node(s);
                                }
                                return Ok(Async::Ready(Some(PageMessage::DocumentAvailable)));
                            } else {
                                return Err(ChromePageError::NoRootNode.into());
                            }
                        }
                    }
                    OnePageState::WaitingNode(selector, mid) => {
                        info!("*** WaitingNode {:?} ***", mid);
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            let selector_cloned = selector.clone();
                            if let Ok(v) = protocol::parse_response::<
                                dom::methods::QuerySelectorReturnObject,
                            >(resp)
                            {
                                self.describe_node(selector_cloned, v.node_id);
                            }
                        }
                    }
                    OnePageState::WaitingDescribeNode(maybe_selector, mid) => {
                        info!("*** WaitingDescribeNode ***");
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            trace!("----------got describe Node resp: {:?}", resp);
                            if let Ok(v) = protocol::parse_response::<
                                dom::methods::DescribeNodeReturnObject,
                            >(resp)
                            {
                                if let PageMessage::FindNode(_, _) = &self.expect_page_message {
                                    return Ok(Async::Ready(Some(PageMessage::FindNode(
                                        *maybe_selector,
                                        Some(v.node),
                                    ))));
                                } else {
                                    let selector_cloned = maybe_selector.clone();
                                    self.find_element(selector_cloned, v.node.backend_node_id);
                                }
                            }
                        }
                    }
                    OnePageState::WaitingRemoteObject(backend_node_id, selector, mid) => {
                        info!("*** WaitingRemoteObject ***");
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(v) = protocol::parse_response::<
                                dom::methods::ResolveNodeReturnObject,
                            >(resp)
                            {
                                let selector_cloned = selector.clone();
                                let element = Element {
                                    remote_object_id: v.object.object_id.unwrap().clone(),
                                    backend_node_id: *backend_node_id,
                                };
                                if let PageMessage::FindElement(_, _) = self.expect_page_message {
                                    return Ok(Async::Ready(Some(PageMessage::FindElement(
                                        selector_cloned,
                                        Some(element),
                                    ))));
                                } else {
                                    self.get_box_model(selector_cloned, &element);
                                }
                            } else {
                                self.state = OnePageState::Consuming;
                            }
                        }
                    }
                    OnePageState::WaitingModelBox(selector, backend_node_id, mid) => {
                        info!("*** WaitingModelBox ***");
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(v) = protocol::parse_response::<
                                dom::methods::GetBoxModelReturnObject,
                            >(resp)
                            {
                                let raw_model = v.model;
                                let model_box = BoxModel {
                                    content: ElementQuad::from_raw_points(&raw_model.content),
                                    padding: ElementQuad::from_raw_points(&raw_model.padding),
                                    border: ElementQuad::from_raw_points(&raw_model.border),
                                    margin: ElementQuad::from_raw_points(&raw_model.margin),
                                    width: raw_model.width,
                                    height: raw_model.height,
                                };
                                match &self.expect_page_message {
                                    PageMessage::GetBoxModel(_, _, _) => {
                                        return Ok(Async::Ready(Some(PageMessage::GetBoxModel(
                                            *selector,
                                            *backend_node_id,
                                            model_box,
                                        ))));
                                    }
                                    PageMessage::Screenshot(a, fmt, from_surface, c) => {
                                        self.capture_screenshot(
                                            fmt.clone(),
                                            Some(model_box.content_viewport()),
                                            from_surface.clone(),
                                        );
                                    }
                                    _ => (),
                                }
                            } else {
                                info!("waiting for WaitingModelBox...1");
                                self.state = OnePageState::Consuming;
                            }
                        } else {
                            info!("waiting for WaitingModelBox...2");
                        }
                    }
                    OnePageState::WaitingScreenShot(mid) => {
                        info!("*** WaitingScreenShot ***");
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(v) = protocol::parse_response::<
                                page::methods::CaptureScreenshotReturnObject,
                            >(resp)
                            {
                                self.state = OnePageState::Consuming;
                                let data_v8 = base64::decode(&v.data).unwrap();
                                if let PageMessage::Screenshot(_, format, from_surface, _) =
                                    &self.expect_page_message
                                {
                                    return Ok(Async::Ready(Some(PageMessage::Screenshot(
                                        None,
                                        format.clone(),
                                        from_surface.clone(),
                                        Some(data_v8),
                                    ))));
                                }
                            }
                            self.state = OnePageState::Consuming;
                        }
                    }
                    _ => {
                        trace!("receive message: {:?}", value);
                        return Ok(Async::Ready(Some(PageMessage::MessageAvailable(value))));
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

// pub type OnePageWithTimeout = TimeoutStream<OnePage>;
