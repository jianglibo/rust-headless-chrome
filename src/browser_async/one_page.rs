use super::element_async::{BoxModel, Element, ElementQuad};
use super::id_type as ids;
use super::json_assistor;
use super::page_message::{
    ChangingFrame, ChangingFrameTree, PageEventName, PageMessage, QuerySelector, SelectorString,
    TaskDescribe, TaskExpect, TaskId,
};
use crate::browser::tab::keys;
use crate::browser_async::chrome_browser::ChromeBrowser;
use crate::browser_async::dev_tools_method_util::{
    ChromePageError, MethodBeforSendResult, MethodDestination, MethodUtil,
};
use crate::browser_async::point_async::Point;
use crate::protocol::{self, dom, input, page, page::methods::Navigate, target};
use failure::{Error, Fail};
use log::*;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use websocket::futures::{Async, Future, Poll, Stream};

#[derive(Hash, Eq, PartialEq, Debug)]
enum PageEvent {
    GetDocument,
}

#[derive(Debug)]
pub enum OnePageState {
    WaitingPageCreate,
    WaitingPageAttach,
    WaitingPageEnable(usize),
    WaitingFrameTree(usize),
    // AfterInvokeNavigate,
    WaitingGetDocument(usize, Option<&'static str>),
    WaitingDomQuerySelector(Option<&'static str>, usize, bool),
    WaitingDescribeNode(Option<&'static str>, usize, dom::NodeId, bool),
    WaitingRemoteObject(dom::NodeId, Option<&'static str>, usize),
    WaitingModelBox(Option<&'static str>, dom::NodeId, usize),
    WaitingScreenShot(usize),
    Consuming,
}

/// why need a task_id? If invoked find_node('#a'), find_node('#b'), find_node('#c') at a time how can I distinguish which is which in the following loop? That's task_id for.
/// Who drive the App to loop? They are tasks. For some reasons tasks execution must be delayed must waiting some events to happen, Store these tasks and execute lately.
/// It's a must that we know how to play from a task_describe.

pub struct OnePage {
    chrome_browser: ChromeBrowser,
    pub state: OnePageState,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    root_node: Option<dom::Node>,
    method_id_2_task_id: HashMap<ids::Method, ids::Task>,
    task_id_2_task: HashMap<ids::Task, TaskDescribe>,
    tasks_waiting_event: HashMap<PageEvent, Vec<ids::Method>>,
    pub changing_frame_tree: ChangingFrameTree,
    unique_number: AtomicUsize,
}

impl OnePage {
    pub fn is_main_frame_navigated(&self) -> bool {
        if let Some(ChangingFrame::Navigated(_)) = &self.changing_frame_tree.changing_frame {
            true
        } else {
            false
        }
    }

    pub fn is_frame_navigated(&self, frame_name: &'static str) -> Option<&page::Frame> {
        let op = self
            .changing_frame_tree
            .child_changing_frames
            .values()
            .find(|cv| {
                if let ChangingFrame::Navigated(frame) = cv {
                    frame.name == Some(frame_name.into())
                } else {
                    false
                }
            });
        if let Some(ChangingFrame::Navigated(frame)) = op {
            Some(frame)
        } else {
            None
        }
    }
    pub fn new(chrome_browser: ChromeBrowser) -> Self {
        Self {
            chrome_browser,
            state: OnePageState::WaitingPageCreate,
            target_info: None,
            session_id: None,
            root_node: None,
            method_id_2_task_id: HashMap::new(),
            task_id_2_task: HashMap::new(),
            tasks_waiting_event: HashMap::new(),
            changing_frame_tree: ChangingFrameTree::new(),
            unique_number: AtomicUsize::new(10000),
        }
    }

    pub fn force_next_stage(&mut self) {}

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
        self.state = OnePageState::Consuming;
        self.chrome_browser.send_message(method_str);
    }

    pub fn get_document(&mut self) {
        let t_id = self.unique_number.fetch_add(1, Ordering::SeqCst);
        if self.root_node.is_none() {
            let (_, method_str, mid) = self
                .create_msg_to_send_with_session_id(dom::methods::GetDocument {
                    depth: Some(0),
                    pierce: Some(false),
                })
                .unwrap();
            self.task_id_2_task
                .insert(t_id, TaskDescribe::GetDocument(t_id));
            self.method_id_2_task_id.entry(mid.unwrap()).or_insert(t_id);
            self.chrome_browser.send_message(method_str);
        }
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

    // pass in an usize under 10000 or None.
    pub fn dom_query_selector_by_selector(
        &mut self,
        selector: &'static str,
        task_id: Option<usize>,
    ) {
        let t_id = task_id.unwrap_or(self.unique_number.fetch_add(1, Ordering::SeqCst));
        let td = QuerySelector {
            selector,
            task_expect: TaskExpect::NodeId,
            task_id: t_id,
        };
        self.task_id_2_task
            .insert(td.task_id, TaskDescribe::QuerySelector(td));
        self.dom_query_selector_by_selector_1(t_id);
    }

    pub fn dom_query_selector_by_selector_1(&mut self, task_id: ids::Task) {
        // When get a task_describe, we must enable to image out what to do.
        if let Some(root_node) = &self.root_node {
            self.dom_query_selector_extra(root_node.node_id, task_id);
        } else {
            self.tasks_waiting_event
                .entry(PageEvent::GetDocument)
                .or_insert(vec![])
                .push(task_id);
            self.get_document();
        }
    }

    pub fn dom_query_selector_by_selector_2(&mut self, query_selector: QuerySelector) {
        let t_id = query_selector.task_id;
        self.task_id_2_task
            .insert(t_id, TaskDescribe::QuerySelector(query_selector));
        if let Some(root_node) = &self.root_node {
            self.dom_query_selector_extra(root_node.node_id, t_id);
        } else {
            error!("root node does't exists.");
        }
    }

    pub fn dom_query_selector(&mut self, ancestor: dom::NodeId, selector: &'static str) {
        let t_id = self.unique_number.fetch_add(1, Ordering::SeqCst);
        let td = QuerySelector {
            selector,
            task_expect: TaskExpect::NodeId,
            task_id: t_id,
        };
        self.task_id_2_task
            .insert(td.task_id, TaskDescribe::QuerySelector(td));
        self.dom_query_selector_extra(ancestor, t_id);
    }

    fn dom_query_selector_extra(&mut self, ancestor: dom::NodeId, task_id: ids::Task) {
        let task_describe = self.task_id_2_task.get(&task_id).unwrap();
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::QuerySelector {
                node_id: ancestor,
                selector: task_describe.get_selector().unwrap(),
            })
            .unwrap();
        self.chrome_browser.send_message(method_str);
        self.method_id_2_task_id
            .entry(mid.unwrap())
            .or_insert(task_describe.get_task_id());
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

    pub fn dom_describe_node_by_selector(
        &mut self,
        selector: &'static str,
        task_id: Option<usize>,
    ) {
        let td = QuerySelector {
            selector,
            task_expect: TaskExpect::Node,
            task_id: task_id.unwrap_or(self.unique_number.fetch_add(1, Ordering::SeqCst)),
        };
        let task_id = td.task_id;
        self.task_id_2_task
            .insert(td.task_id, TaskDescribe::QuerySelector(td));
        self.dom_query_selector_by_selector_1(task_id);
    }

    pub fn dom_describe_node_extra(&mut self, node_id: dom::NodeId, task_describe: TaskDescribe) {
        let (_, method_str, mid) = self
            .create_msg_to_send_with_session_id(dom::methods::DescribeNode {
                node_id: Some(node_id),
                backend_node_id: None,
                depth: Some(100),
            })
            .unwrap();
        self.method_id_2_task_id
            .entry(mid.unwrap())
            .or_insert(task_describe.get_task_id());
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

    // pub fn capture_screenshot_by_selector(
    //     &mut self,
    //     selector: &'static str,
    //     format: page::ScreenshotFormat,
    //     from_surface: bool,
    // ) -> &TaskDescribe {

    //     let td = TaskDescribe::QuerySelector {
    //         selector,
    //         task_expect: TaskExpect::ScreenShot,
    //         task_id: self.unique_number.fetch_add(1, Ordering::SeqCst),
    //     };
    //     // self.expect_page_message =
    //     //     PageMessage::Screenshot(Some(selector), format, from_surface, None);
    //     self.dom_query_selector_by_selector(selector)
    // }

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

    /// pick out some special tasks like getDocument.
    pub fn handle_response(&mut self, resp: protocol::Response) -> Option<PageMessage> {
        trace!("got message from target response. {:?}", resp);
        let call_id = resp.call_id;
        let matched_task = self
            .method_id_2_task_id
            .get(&call_id)
            .and_then(|task_id| self.task_id_2_task.get(task_id))
            .or({
                error!("not matching task_id to call_id {}.", call_id);
                None
            });

        // message: "{\"id\":6,\"result\":{\"root\":{\"nodeId\":1,\"backendNodeId\":3,\"nodeType\":9,\"nodeName\":\"#document\",\"localName\":\"\",\"nodeValue\":\"\",\"childNodeCount\":2,\"documentURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"baseURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"xmlVersion\":\"\"}}}"
        // if json_assistor::response_result_field_has_properties(&resp, "root", vec!["nodeId", "backendNodeId"]) {
        //    let node: dom::methods::GetDocumentReturnObject = serde_json::from_value(resp.result.unwrap()).unwrap();
        // }

        // info!("got call_id {}, task_id {}", call_id, task_id);
        // info!("got response: {:?}", resp);
        if let Some(TaskDescribe::GetDocument(_)) = matched_task {
            let mut waiting_task_ids: Vec<_> = self
                .tasks_waiting_event
                .get_mut(&PageEvent::GetDocument)
                .unwrap_or(&mut vec![])
                .drain(..)
                .collect();

            let mut waiting_tasks: Vec<_> = waiting_task_ids
                .iter()
                .flat_map(|t_id| self.task_id_2_task.remove(&t_id))
                .collect();

            while let Some(task) = waiting_tasks.pop() {
                match task {
                    TaskDescribe::QuerySelector(qs) => {
                        self.dom_query_selector_by_selector_2(qs);
                    }
                    _ => (),
                }
            }
        // break Ok(Some())
        } else {
            trace!("skipping....................0");
        }
        None
    }
}

// The main loop should stop at some point, by invoking the methods on the page to drive the loop to run.
impl Stream for OnePage {
    type Item = PageMessage;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.chrome_browser.poll()) {
                // info!("raw value: {:?}", value);
                match &mut self.state {
                    OnePageState::WaitingPageCreate => {
                        trace!("*** WaitingPageCreate ***");
                        if let Some(target_info) = MethodUtil::is_page_event_create(value) {
                            self.target_info = Some(target_info);
                            self.attach_to_page();
                        }
                    }
                    OnePageState::WaitingPageAttach => {
                        trace!("*** WaitingPageAttach ***");
                        if let Some((session_id, target_info)) =
                            MethodUtil::is_page_attach_event(value)
                        {
                            self.session_id = Some(session_id);
                            self.target_info = Some(target_info);
                            self.page_enable();
                        }
                    }
                    OnePageState::WaitingPageEnable(mid) => {
                        // The page enableing has no return value, so must use mid.
                        trace!("*** WaitingPageEnable ***");
                        if MethodUtil::match_chrome_response(value, mid).is_some() {
                            return Ok(Some(PageMessage::EnablePageDone).into());
                        }
                    }
                    OnePageState::WaitingFrameTree(mid) => {
                        trace!("*** WaitingFrameTree {:?} ***", mid);
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(v) = protocol::parse_response::<
                                page::methods::GetFrameTreeReturnObject,
                            >(resp)
                            {
                                trace!("----------------- got frames: {:?}", v);
                                return Ok(Some(PageMessage::GetFrameTree(v.frame_tree)).into());
                            }
                        }
                    }
                    // OnePageState::WaitingGetDocument(mid, ref next_find_node) => {
                    //     info!("*** WaitingGetDocument ***");
                    //     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                    //         if let Ok(c) = protocol::parse_response::<
                    //             dom::methods::GetDocumentReturnObject,
                    //         >(resp)
                    //         {
                    //             info!("got document Node: {:?}", c.root);
                    //             self.root_node = Some(c.root);
                    //             if let Some(selector) = next_find_node.as_ref() {
                    //                 let s = *selector;
                    //                 self.dom_query_selector(None, s, true);
                    //             }
                    //             return Ok(Some(PageMessage::DocumentAvailable).into());
                    //         } else {
                    //             return Err(ChromePageError::NoRootNode.into());
                    //         }
                    //     }
                    // }
                    // OnePageState::WaitingDomQuerySelector(selector, mid, invoke_next) => {
                    //     info!("*** WaitingNode {:?} ***", mid);
                    //     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                    //         let selector_cloned = selector.clone();
                    //         if let Ok(v) = protocol::parse_response::<
                    //             dom::methods::QuerySelectorReturnObject,
                    //         >(resp)
                    //         {   let inv = invoke_next.clone();
                    //             if inv {
                    //                 self.dom_describe_node(selector_cloned, Some(v.node_id), inv);
                    //             }
                    //             if v.node_id > 0 {
                    //                 break Ok(Some(PageMessage::DomQuerySelector(selector_cloned, Some(v.node_id))).into());
                    //             } else {
                    //                 break Ok(Some(PageMessage::DomQuerySelector(selector_cloned, None)).into());
                    //             }
                    //         }
                    //     }
                    // }
                    OnePageState::WaitingDescribeNode(
                        maybe_selector,
                        mid,
                        node_id,
                        invoke_next,
                    ) => {
                        trace!("*** WaitingDescribeNode ***");
                        if node_id == &0 {
                            break Ok(
                                Some(PageMessage::DomDescribeNode(*maybe_selector, None)).into()
                            );
                        }
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            trace!("----------got describe Node resp: {:?}", resp);
                            if let Ok(v) = protocol::parse_response::<
                                dom::methods::DescribeNodeReturnObject,
                            >(resp)
                            {
                                let selector_cloned = maybe_selector.clone();
                                let selector_cloned_1 = maybe_selector.clone();
                                if *invoke_next {
                                    self.find_element(selector_cloned, v.node.backend_node_id);
                                }
                                break Ok(Some(PageMessage::DomDescribeNode(
                                    selector_cloned_1,
                                    Some(v.node),
                                ))
                                .into());
                            }
                        }
                    }
                    OnePageState::WaitingRemoteObject(backend_node_id, selector, mid) => {
                        trace!("*** WaitingRemoteObject ***");
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
                            // if let PageMessage::FindElement(_, _) = self.expect_page_message {
                            //     return Ok(Some(PageMessage::FindElement(
                            //         selector_cloned,
                            //         Some(element),
                            //     )).into());
                            // } else {
                            //     self.get_box_model(selector_cloned, &element);
                            // }
                            } else {
                                self.state = OnePageState::Consuming;
                            }
                        }
                    }
                    OnePageState::WaitingModelBox(selector, backend_node_id, mid) => {
                        trace!("*** WaitingModelBox ***");
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
                            // match &self.expect_page_message {
                            //     PageMessage::GetBoxModel(_, _, _) => {
                            //         return Ok(Some(PageMessage::GetBoxModel(
                            //             *selector,
                            //             *backend_node_id,
                            //             model_box,
                            //         )).into());
                            //     }
                            //     PageMessage::Screenshot(a, fmt, from_surface, c) => {
                            //         self.capture_screenshot(
                            //             fmt.clone(),
                            //             Some(model_box.content_viewport()),
                            //             from_surface.clone(),
                            //         );
                            //     }
                            //     _ => (),
                            // }
                            } else {
                                trace!("waiting for WaitingModelBox...1");
                                self.state = OnePageState::Consuming;
                            }
                        } else {
                            trace!("waiting for WaitingModelBox...2");
                        }
                    }
                    OnePageState::WaitingScreenShot(mid) => {
                        trace!("*** WaitingScreenShot ***");
                        if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
                            if let Ok(v) = protocol::parse_response::<
                                page::methods::CaptureScreenshotReturnObject,
                            >(resp)
                            {
                                self.state = OnePageState::Consuming;
                                let data_v8 = base64::decode(&v.data).unwrap();
                                // if let PageMessage::Screenshot(_, format, from_surface, _) =
                                //     &self.expect_page_message
                                // {
                                //     return Ok(Some(PageMessage::Screenshot(
                                //         None,
                                //         format.clone(),
                                //         from_surface.clone(),
                                //         Some(data_v8),
                                //     )).into());
                                // }
                            }
                            self.state = OnePageState::Consuming;
                        }
                    }
                    _ => {
                        // #[derive(Deserialize, Debug, PartialEq, Clone)]
                        // pub struct Response {
                        //     #[serde(rename(deserialize = "id"))]
                        //     pub call_id: CallId,
                        //     pub result: Option<Value>,
                        //     pub error: Option<RemoteError>,
                        // }
                        match value {
                            protocol::Message::Response(resp) => {
                                if let Some(page_message) = self.handle_response(resp) {
                                    break Ok(Some(page_message).into());
                                }
                            }
                            protocol::Message::Event(
                                protocol::Event::ReceivedMessageFromTarget(target_message_event),
                            ) => {
                                let event_params = &target_message_event.params;
                                let message_field = &event_params.message;
                                match protocol::parse_raw_message(&message_field) {
                                    Ok(protocol::Message::Response(resp)) => {
                                        if let Some(page_message) = self.handle_response(resp) {
                                            break Ok(Some(page_message).into());
                                        }
                                    }
                                    Ok(protocol::Message::Event(inner_event)) => {
                                        match inner_event {
                                            protocol::Event::FrameNavigated(
                                                frame_navigated_event,
                                            ) => {
                                                let frame_id =
                                                    frame_navigated_event.params.frame.id.clone();
                                                let parent_id = frame_navigated_event
                                                    .params
                                                    .frame
                                                    .parent_id
                                                    .clone();
                                                let changing_frame = ChangingFrame::Navigated(
                                                    frame_navigated_event.params.frame,
                                                );
                                                if parent_id.is_none() {
                                                    self.changing_frame_tree
                                                        .changing_frame
                                                        .replace(changing_frame);
                                                } else {
                                                    self.changing_frame_tree
                                                        .child_changing_frames
                                                        .insert(frame_id, changing_frame);
                                                    // entry(frame_id).and_modify(|v| *v = changing_frame).or_insert(changing_frame);
                                                }
                                                break Ok(Some(PageMessage::PageEvent(
                                                    PageEventName::frameNavigated,
                                                ))
                                                .into());
                                            }
                                            protocol::Event::TargetInfoChanged(
                                                target_info_changed,
                                            ) => {
                                                break Ok(Some(PageMessage::TargetInfoChanged(
                                                    target_info_changed.params.target_info,
                                                ))
                                                .into());
                                            }
                                            _ => {
                                                error!(
                                                    "unprocessed inner event: {:?}",
                                                    inner_event
                                                );
                                            }
                                        }
                                    }
                                    _ => {
                                        error!("unprocessed {:?}", target_message_event);
                                    }
                                }
                            }
                            other => {
                                error!("got unknown message1: {:?}", other);
                            }
                        }
                        // trace!("receive message: {:?}", value);
                        // return Ok(Some(PageMessage::MessageAvailable(value)).into());
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

// pub type OnePageWithTimeout = TimeoutStream<OnePage>;
// Page.frameAttached -> Page.frameStartedLoading(44) -> Page.frameNavigated(48) -> Page.domContentEventFired(64) -> Page.loadEventFired(131) -> Page.frameStoppedLoading(132)

// target_id and browser_context_id keep unchanged.
// Event(TargetInfoChanged(TargetInfoChangedEvent { params: TargetInfoChangedParams {
// target_info: TargetInfo { target_id: "7AF7B8E3FC73BFB961EF5F16A814EECC", target_type: Page, title: "about:blank", url: "about:blank", attached: true, opener_id: None, browser_context_id: Some("1771E7BCAE49411BB7D7C9C152191641") } } }))
// target_info: TargetInfo { target_id: "7AF7B8E3FC73BFB961EF5F16A814EECC", target_type: Page, title: "https://pc", url: "https://pc", attached: true, opener_id: None, browser_context_id: Some("1771E7BCAE49411BB7D7C9C152191641") } } }))
