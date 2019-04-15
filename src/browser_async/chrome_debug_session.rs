use super::task_describe::{self as tasks, TaskDescribe};
use super::element_async::{BoxModel, Element, ElementQuad};
use super::id_type as ids;
use super::page_message::ChangingFrame;
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
pub struct ChromeDebugSession {
    chrome_browser: ChromeBrowser,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    method_id_2_task_id: HashMap<ids::Method, ids::Task>,
    task_id_2_task: HashMap<ids::Task, TaskDescribe>,
    waiting_for_me: HashMap<ids::Task, Vec<ids::Task>>,
    unique_number: AtomicUsize,
}

impl ChromeDebugSession {
    pub fn new(chrome_browser: ChromeBrowser) -> Self {
        Self {
            chrome_browser,
            target_info: None,
            session_id: None,
            method_id_2_task_id: HashMap::new(),
            task_id_2_task: HashMap::new(),
            waiting_for_me: HashMap::new(),
            unique_number: AtomicUsize::new(10000),
        }
    }

    pub fn dom_query_selector(&mut self, task: TaskDescribe) {
        if let TaskDescribe::QuerySelector(tasks::QuerySelector {
            task_id,
            node_id: Some(node_id_value),
            session_id,
            selector,
            ..
        }) = &task
        {
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(
                dom::methods::QuerySelector {
                    node_id: *node_id_value,
                    selector,
                },
                &session_id,
            )
            .unwrap();
            self.add_task_and_method_map(mid.unwrap(), task_id.clone(), task);
            self.send_message(method_str);
        } else {
            error!("it's not a query selector task.");
            panic!("it's not a query selector task.");
        }
    }


    pub fn send_message(&mut self, method_str: String) {
        self.chrome_browser.send_message(method_str);
    }

    pub fn add_task(&mut self, task_id: ids::Task, task: TaskDescribe) {
        self.task_id_2_task.insert(task_id, task);
    }

    pub fn add_method_task_map(&mut self, mid: usize, task_id: ids::Task) {
        self.method_id_2_task_id.entry(mid).or_insert(task_id);
    }

    pub fn add_task_and_method_map(&mut self, mid: usize, task_id: usize, task: TaskDescribe) {
        self.add_method_task_map(mid, task_id);
        self.add_task(task_id, task);
    }

    pub fn add_waiting_task(&mut self, this_task_id: usize, waiting_task_id: usize) {
        self.waiting_for_me
            .entry(this_task_id)
            .or_insert(vec![])
            .push(waiting_task_id);
    }


    fn get_waiting_tasks(&mut self, task_id: ids::Task) -> Vec<TaskDescribe> {
        // Take out all tasks waiting for me.
        let mut waiting_task_ids: Vec<_> = self
            .waiting_for_me
            .get_mut(&task_id)
            .unwrap_or(&mut vec![])
            .drain(..)
            .collect();

        // Remove task_id task pair.
        waiting_task_ids
            .iter()
            .flat_map(|t_id| self.task_id_2_task.remove(&t_id))
            .collect()
    }

    pub fn dom_describe_node(&mut self, task: tasks::TaskDescribe) {
        if let tasks::TaskDescribe::DescribeNode(tasks::DescribeNode {
            task_id,
            node_id,
            is_manual,
            backend_node_id,
            selector,
            session_id,
        }) = &task
        {
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(dom::methods::DescribeNode {
                    node_id: *node_id,
                    backend_node_id: *backend_node_id,
                    depth: Some(100),
                }, &session_id)
                .unwrap();
            self.add_task_and_method_map(mid.unwrap(), task_id.clone(), task);
            self.chrome_browser.send_message(method_str);
        } else {
            error!("not a node_describe.")
        }
    }

    pub fn feed_on_node_id(&mut self, task_id: ids::Task, node_id: dom::NodeId) {
        let mut waiting_tasks = self.get_waiting_tasks(task_id);
        while let Some(mut task) = waiting_tasks.pop() {
            match &mut task {
                tasks::TaskDescribe::QuerySelector(query_selector) => {
                    query_selector.node_id = Some(node_id);
                    self.dom_query_selector(task);
                }
                tasks::TaskDescribe::DescribeNode(describe_node) => {
                    describe_node.node_id = Some(node_id);
                    self.dom_describe_node(task);
                }
                _ => (),
            }
        }
    }

    pub fn handle_response(
        &mut self,
        resp: protocol::Response,
        session_id: Option<String>,
        target_id: Option<String>,
    ) -> Option<TaskDescribe> {
        trace!("got message from target response. {:?}", resp);
        let call_id = resp.call_id;
        // remove method id and task from page scope hashmap.
        let maybe_matched_task = self
            .method_id_2_task_id
            .remove(&call_id)
            .and_then(|task_id| self.task_id_2_task.remove(&task_id))
            .or({
                error!("not matching task_id to call_id {}.", call_id);
                None
            });

        // message: "{\"id\":6,\"result\":{\"root\":{\"nodeId\":1,\"backendNodeId\":3,\"nodeType\":9,\"nodeName\":\"#document\",\"localName\":\"\",\"nodeValue\":\"\",\"childNodeCount\":2,\"documentURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"baseURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"xmlVersion\":\"\"}}}"
        // if json_assistor::response_result_field_has_properties(&resp, "root", vec!["nodeId", "backendNodeId"]) {
        //    let node: dom::methods::GetDocumentReturnObject = serde_json::from_value(resp.result.unwrap()).unwrap();
        // }

        if let Some(mut task) = maybe_matched_task {
            match &mut task {
                TaskDescribe::GetDocument(task_id, _, _) => {
                    // it must be a GetDocumentReturnObject or else something must go wrong.
                    if let Ok(get_document_return_object) =
                        serde_json::from_value::<dom::methods::GetDocumentReturnObject>(
                            resp.result.unwrap(),
                        )
                    {
                        let node_id = get_document_return_object.root.node_id;
                        self.feed_on_node_id(*task_id, node_id);
                        return Some(TaskDescribe::GetDocument(
                            *task_id,
                            target_id,
                            Some(get_document_return_object.root),
                        ));
                    } else {
                        panic!("GetDocument failed.");
                    }
                }
                TaskDescribe::PageEnable(task_id, _) => {
                    return Some(TaskDescribe::PageEnable(*task_id, target_id));
                }
                TaskDescribe::QuerySelector(query_selector) => {
                    if let Ok(query_select_return_object) =
                        serde_json::from_value::<dom::methods::QuerySelectorReturnObject>(
                            resp.result.unwrap(),
                        )
                    {
                        self.feed_on_node_id(query_selector.task_id, query_select_return_object.node_id);
                        if query_selector.is_manual {
                            query_selector.found_node_id.replace(query_select_return_object.node_id);
                            return Some(task);
                        }
                    } else {
                        panic!("QuerySelector failed.");
                    }
                }
                TaskDescribe::DescribeNode(describe_node) => {
                    if let Ok(node) = serde_json::from_value::<dom::methods::DescribeNodeReturnObject>(
                        resp.result.unwrap(),
                    ) {
                        // self.feed_on_node(describe_node.task_id, node.node);
                        if describe_node.is_manual {
                            return Some(task);
                            // return Some(TaskDescribe::DescribeNode(*describe_node));
                        }
                    } else {
                        panic!("QuerySelector failed.");
                    }
                }
                task_describe => {
                    info!("got task_describe: {:?}", task_describe);
                }
            }
        } else {
            error!("method id {:?} has no task matched. {:?}", call_id, resp);
        }
        None
    }

    fn handle_protocol_event(
        &mut self,
        protocol_event: protocol::Event,
        session_id: Option<String>,
        target_id: Option<String>,
    ) -> Option<TaskDescribe> {
        match protocol_event {
            protocol::Event::FrameNavigated(frame_navigated_event) => {
                let changing_frame = ChangingFrame::Navigated(frame_navigated_event.params.frame);
                return Some(TaskDescribe::FrameNavigated(
                    target_id.unwrap(),
                    changing_frame,
                ));
            }
            protocol::Event::TargetInfoChanged(target_info_changed) => {
                return Some(TaskDescribe::TargetInfoChanged(
                    target_info_changed.params.target_info,
                ));
            }
            protocol::Event::TargetCreated(target_created_event) => {
                let target_type = &(target_created_event.params.target_info.target_type);
                match target_type {
                    protocol::target::TargetType::Page => {
                        trace!(
                            "receive page create event. {:?}",
                            target_created_event.params.target_info
                        );
                        return Some(TaskDescribe::PageCreated(
                            target_created_event.params.target_info,
                            None,
                        ));
                    }
                    _ => (),
                }
            }
            protocol::Event::AttachedToTarget(event) => {
                let attach_to_target_params: protocol::target::events::AttachedToTargetParams =
                    event.params;
                let target_info: protocol::target::TargetInfo = attach_to_target_params.target_info;

                match target_info.target_type {
                    protocol::target::TargetType::Page => {
                        info!(
                            "got attach to page event and sessionId: {}",
                            attach_to_target_params.session_id
                        );
                        return Some(TaskDescribe::PageAttached(
                            target_info,
                            attach_to_target_params.session_id.into(),
                        ));
                        // return Some((attach_to_target_params.session_id, target_info));
                    }
                    _ => (),
                }
            }
            _ => {
                error!("unprocessed inner event: {:?}", protocol_event);
            }
        }
        None
    }
}

// The main loop should stop at some point, by invoking the methods on the page to drive the loop to run.
impl Stream for ChromeDebugSession {
    type Item = TaskDescribe;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.chrome_browser.poll()) {
                match value {
                    protocol::Message::Response(resp) => {
                        if let Some(page_message) = self.handle_response(resp, None, None) {
                            break Ok(Some(page_message).into());
                        }
                    }
                    protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(
                        target_message_event,
                    )) => {
                        let event_params = &target_message_event.params;
                        let session_id = event_params.session_id.clone();
                        let target_id = event_params.target_id.clone();
                        let message_field = &event_params.message;
                        match protocol::parse_raw_message(&message_field) {
                            Ok(protocol::Message::Response(resp)) => {
                                if let Some(page_message) =
                                    self.handle_response(resp, Some(session_id), Some(target_id))
                                {
                                    break Ok(Some(page_message).into());
                                }
                            }
                            Ok(protocol::Message::Event(protocol_event)) => {
                                if let Some(page_message) = self.handle_protocol_event(
                                    protocol_event,
                                    Some(session_id),
                                    Some(target_id),
                                ) {
                                    break Ok(Some(page_message).into());
                                }
                            }
                            _ => {
                                error!("unprocessed ** {:?}", message_field);
                            }
                        }
                    }
                    protocol::Message::Event(protocol_event) => {
                        if let Some(page_message) =
                            self.handle_protocol_event(protocol_event, None, None)
                        {
                            break Ok(Some(page_message).into());
                        }

                    }
                    //                             pub enum Event {
                    //     #[serde(rename = "Target.attachedToTarget")]
                    //     AttachedToTarget(target::events::AttachedToTargetEvent),
                    //     #[serde(rename = "Target.receivedMessageFromTarget")]
                    //     ReceivedMessageFromTarget(target::events::ReceivedMessageFromTargetEvent),
                    //     #[serde(rename = "Target.targetInfoChanged")]
                    //     TargetInfoChanged(target::events::TargetInfoChangedEvent),
                    //     #[serde(rename = "Target.targetCreated")]
                    //     TargetCreated(target::events::TargetCreatedEvent),
                    //     #[serde(rename = "Target.targetDestroyed")]
                    //     TargetDestroyed(target::events::TargetDestroyedEvent),
                    //     #[serde(rename = "Page.frameStartedLoading")]
                    //     FrameStartedLoading(page::events::FrameStartedLoadingEvent),
                    //     #[serde(rename = "Page.frameNavigated")]
                    //     FrameNavigated(page::events::FrameNavigatedEvent),
                    //     #[serde(rename = "Page.frameAttached")]
                    //     FrameAttached(page::events::FrameAttachedEvent),
                    //     #[serde(rename = "Page.frameStoppedLoading")]
                    //     FrameStoppedLoading(page::events::FrameStoppedLoadingEvent),
                    //     #[serde(rename = "Page.lifecycleEvent")]
                    //     Lifecycle(page::events::LifecycleEvent),
                    //     #[serde(rename = "Network.requestIntercepted")]
                    //     RequestIntercepted(network::events::RequestInterceptedEvent),
                    // }
                    other => {
                        error!("got unknown message1: {:?}", other);
                    }
                }
            // trace!("receive message: {:?}", value);
            // return Ok(Some(PageMessage::MessageAvailable(value)).into());
            // }
            // }
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

// ChromeSessionState::WaitingPageCreate => {
//     trace!("*** WaitingPageCreate ***");
//     if let Some(target_info) = MethodUtil::is_page_event_create(value) {
//         // self.target_info = Some(target_info);
//         // self.attach_to_page();
//         info!("page created: {:?}", target_info);
//         return Ok(Some(PageMessage::PageCreated(target_info, None)).into());
//     }
// }
// ChromeSessionState::WaitingPageAttach => {
//     trace!("*** WaitingPageAttach ***");
//     if let Some((session_id, target_info)) =
//         MethodUtil::is_page_attach_event(value)
//     {
//         self.session_id = Some(session_id.clone());
//         self.target_info = Some(target_info.clone());
//         self.page_enable(target_info.target_id.clone());
//         return Ok(Some(PageMessage::PageAttached(target_info, session_id.into())).into());
//     }
// }
// ChromeSessionState::WaitingPageEnable(mid, target_id) => {
//     // The page enableing has no return value, so must use mid.
//     trace!("*** WaitingPageEnable ***");
//     if MethodUtil::match_chrome_response(value, mid).is_some() {
//         return Ok(Some(PageMessage::EnablePageDone(target_id.to_string())).into());
//     }
// }
// ChromeSessionState::WaitingFrameTree(mid) => {
//     trace!("*** WaitingFrameTree {:?} ***", mid);
//     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
//         if let Ok(v) = protocol::parse_response::<
//             page::methods::GetFrameTreeReturnObject,
//         >(resp)
//         {
//             trace!("----------------- got frames: {:?}", v);
//             return Ok(Some(PageMessage::GetFrameTree(v.frame_tree)).into());
//         }
//     }
// }
// ChromeSessionState::WaitingGetDocument(mid, ref next_find_node) => {
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
// ChromeSessionState::WaitingDomQuerySelector(selector, mid, invoke_next) => {
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
// ChromeSessionState::WaitingDescribeNode(
//     maybe_selector,
//     mid,
//     node_id,
//     invoke_next,
// ) => {
//     trace!("*** WaitingDescribeNode ***");
//     if node_id == &0 {
//         break Ok(
//             Some(PageMessage::DomDescribeNode(*maybe_selector, None)).into()
//         );
//     }
//     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
//         trace!("----------got describe Node resp: {:?}", resp);
//         if let Ok(v) = protocol::parse_response::<
//             dom::methods::DescribeNodeReturnObject,
//         >(resp)
//         {
//             let selector_cloned = maybe_selector.clone();
//             let selector_cloned_1 = maybe_selector.clone();
//             if *invoke_next {
//                 self.find_element(selector_cloned, v.node.backend_node_id);
//             }
//             break Ok(Some(PageMessage::DomDescribeNode(
//                 selector_cloned_1,
//                 Some(v.node),
//             ))
//             .into());
//         }
//     }
// }
// ChromeSessionState::WaitingRemoteObject(backend_node_id, selector, mid) => {
//     trace!("*** WaitingRemoteObject ***");
//     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
//         if let Ok(v) = protocol::parse_response::<
//             dom::methods::ResolveNodeReturnObject,
//         >(resp)
//         {
//             let selector_cloned = selector.clone();
//             let element = Element {
//                 remote_object_id: v.object.object_id.unwrap().clone(),
//                 backend_node_id: *backend_node_id,
//             };
//         } else {
//             self.state = ChromeSessionState::Consuming;
//         }
//     }
// }
// ChromeSessionState::WaitingModelBox(selector, backend_node_id, mid) => {
//     trace!("*** WaitingModelBox ***");
//     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
//         if let Ok(v) = protocol::parse_response::<
//             dom::methods::GetBoxModelReturnObject,
//         >(resp)
//         {
//             let raw_model = v.model;
//             let model_box = BoxModel {
//                 content: ElementQuad::from_raw_points(&raw_model.content),
//                 padding: ElementQuad::from_raw_points(&raw_model.padding),
//                 border: ElementQuad::from_raw_points(&raw_model.border),
//                 margin: ElementQuad::from_raw_points(&raw_model.margin),
//                 width: raw_model.width,
//                 height: raw_model.height,
//             };
//         // match &self.expect_page_message {
//         //     PageMessage::GetBoxModel(_, _, _) => {
//         //         return Ok(Some(PageMessage::GetBoxModel(
//         //             *selector,
//         //             *backend_node_id,
//         //             model_box,
//         //         )).into());
//         //     }
//         //     PageMessage::Screenshot(a, fmt, from_surface, c) => {
//         //         self.capture_screenshot(
//         //             fmt.clone(),
//         //             Some(model_box.content_viewport()),
//         //             from_surface.clone(),
//         //         );
//         //     }
//         //     _ => (),
//         // }
//         } else {
//             trace!("waiting for WaitingModelBox...1");
//             self.state = ChromeSessionState::Consuming;
//         }
//     } else {
//         trace!("waiting for WaitingModelBox...2");
//     }
// }
// ChromeSessionState::WaitingScreenShot(mid) => {
//     trace!("*** WaitingScreenShot ***");
//     if let Some(resp) = MethodUtil::match_chrome_response(value, mid) {
//         if let Ok(v) = protocol::parse_response::<
//             page::methods::CaptureScreenshotReturnObject,
//         >(resp)
//         {
//             self.state = ChromeSessionState::Consuming;
//             let data_v8 = base64::decode(&v.data).unwrap();
//             // if let PageMessage::Screenshot(_, format, from_surface, _) =
//             //     &self.expect_page_message
//             // {
//             //     return Ok(Some(PageMessage::Screenshot(
//             //         None,
//             //         format.clone(),
//             //         from_surface.clone(),
//             //         Some(data_v8),
//             //     )).into());
//             // }
//         }
//         self.state = ChromeSessionState::Consuming;
//     }
// }

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

// pub fn get_box_model(&mut self, selector: Option<&'static str>, element: &Element) {
//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(dom::methods::GetBoxModel {
//             node_id: None,
//             backend_node_id: Some(element.backend_node_id),
//             object_id: None,
//         })
//         .unwrap();

//     // self.state = ChromeSessionState::WaitingModelBox(selector, element.backend_node_id, mid.unwrap());
//     self.chrome_browser.send_message(method_str);
// }

// pass in an usize under 10000 or None.
// pub fn dom_query_selector_by_selector(
//     &mut self,
//     selector: &'static str,
//     manual_task_id: Option<usize>,
// ) -> (Option<ids::Task>, Option<dom::NodeId>) {
//     let (this_task_id, is_manual) = manual_task_id.map_or_else(
//         || (self.unique_number.fetch_add(1, Ordering::SeqCst), false),
//         |mid| (mid, true),
//     );
//     let mut qs = tasks::QuerySelector {
//         selector,
//         is_manual,
//         node_id: None,
//         task_id: this_task_id,
//     };
//     match self.get_document(None) {
//         (Some(task_id), _) => {
//             // if root node is not ready, will return a task_id.
//             self.task_id_2_task
//                 .insert(qs.task_id, tasks::TaskDescribe::QuerySelector(qs));
//             self.waiting_for_me
//                 .entry(this_task_id)
//                 .or_insert(vec![])
//                 .push(task_id);
//         }
//         (_, Some(node_id)) => {
//             // self.dom_query_selector_extra(node_id, t_id);
//             qs.node_id = Some(node_id);
//             self.dom_query_selector(tasks::TaskDescribe::QuerySelector(qs));
//         }
//         _ => {
//             error!("get_document return impossible value combination.");
//         }
//     }
//     (Some(this_task_id), None)
// }

// fn dom_query_selector(&mut self, task: tasks::TaskDescribe) {
//     if let tasks::TaskDescribe::QuerySelector(tasks::QuerySelector {
//         task_id,
//         is_manual,
//         node_id: Some(node_id_value),
//         selector,
//     }) = task
//     {
//         let (_, method_str, mid) = self
//             .create_msg_to_send_with_session_id(dom::methods::QuerySelector {
//                 node_id: node_id_value,
//                 selector: selector,
//             })
//             .unwrap();
//         self.method_id_2_task_id
//             .entry(mid.unwrap())
//             .or_insert(task_id);
//         self.task_id_2_task.insert(task_id, task);
//         self.chrome_browser.send_message(method_str);
//     } else {
//         error!("it's not a query selector task.");
//         panic!("it's not a query selector task.");
//     }
// }

// pub fn dom_describe_node_by_selector(
//     &mut self,
//     selector: &'static str,
//     manual_task_id: Option<usize>,
// ) {
//     let (this_task_id, is_manual) = manual_task_id.map_or_else(
//         || (self.unique_number.fetch_add(1, Ordering::SeqCst), false),
//         |mid| (mid, true),
//     );
//     // let this_task_id =
//     //     manual_task_id.unwrap_or(self.unique_number.fetch_add(1, Ordering::SeqCst));
//     let mut describe_node = tasks::DescribeNode {
//         selector,
//         is_manual,
//         node_id: None,
//         backend_node_id: None,
//         task_id: this_task_id,
//     };
//     // because node_id is unknown, get it by selector provided.
//     match self.dom_query_selector_by_selector(selector, None) {
//         (Some(task_id), None) => {
//             self.task_id_2_task.insert(
//                 this_task_id,
//                 tasks::TaskDescribe::DescribeNode(describe_node),
//             );

//             self.waiting_for_me
//                 .entry(task_id)
//                 .or_insert(vec![])
//                 .push(this_task_id);
//         }
//         (_, Some(node_id)) => {
//             describe_node.node_id = Some(node_id);
//             self.dom_describe_node(tasks::TaskDescribe::DescribeNode(describe_node));
//         }
//         _ => {
//             error!("dom_query_selector_by_selector return impossible value.");
//         }
//     }
// }

// pub fn get_frame_tree(&mut self) {
//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(page::methods::GetFrameTree {})
//         .unwrap();
//     // self.state = ChromeSessionState::WaitingFrameTree(mid.unwrap());
//     self.chrome_browser.send_message(method_str);
// }

// pub fn find_element(&mut self, selector: Option<&'static str>, backend_node_id: dom::NodeId) {
//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(dom::methods::ResolveNode {
//             backend_node_id: Some(backend_node_id),
//         })
//         .unwrap();
//     // self.state = ChromeSessionState::WaitingRemoteObject(backend_node_id, selector, mid.unwrap());
//     self.chrome_browser.send_message(method_str);
// }

// pub fn type_str(&mut self, string_to_type: &str) {
//     for c in string_to_type.split("") {
//         // split call above will have empty string at start and end which we won't type
//         if c == "" {
//             continue;
//         }
//         self.press_key(c);
//     }
// }

// pub fn press_key(&mut self, key: &str) {
//     let definition = keys::get_key_definition(key).unwrap();

//     // See https://github.com/GoogleChrome/puppeteer/blob/62da2366c65b335751896afbb0206f23c61436f1/lib/Input.js#L114-L115
//     let text = definition.text.or_else(|| {
//         if definition.key.len() == 1 {
//             Some(definition.key)
//         } else {
//             None
//         }
//     });

//     // See https://github.com/GoogleChrome/puppeteer/blob/62da2366c65b335751896afbb0206f23c61436f1/lib/Input.js#L52
//     let key_down_event_type = if text.is_some() {
//         "keyDown"
//     } else {
//         "rawKeyDown"
//     };

//     let key = Some(definition.key);
//     let code = Some(definition.code);

//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(input::methods::DispatchKeyEvent {
//             event_type: key_down_event_type,
//             key,
//             text,
//             code: Some(definition.code),
//             windows_virtual_key_code: definition.key_code,
//             native_virtual_key_code: definition.key_code,
//         })
//         .unwrap();
//     self.chrome_browser.send_message(method_str);

//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(input::methods::DispatchKeyEvent {
//             event_type: "keyUp",
//             key,
//             text,
//             code,
//             windows_virtual_key_code: definition.key_code,
//             native_virtual_key_code: definition.key_code,
//         })
//         .unwrap();
//     self.chrome_browser.send_message(method_str);
// }

// pub fn click_point(&mut self, point: Point) {
//     trace!("Clicking point: {:?}", point);
//     if point.x == 0.0 && point.y == 0.0 {
//         warn!("Midpoint of element shouldn't be 0,0. Something is probably wrong.")
//     }

//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
//             event_type: "mouseMoved",
//             x: point.x,
//             y: point.y,
//             ..Default::default()
//         })
//         .unwrap();
//     self.chrome_browser.send_message(method_str);

//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
//             event_type: "mousePressed",
//             x: point.x,
//             y: point.y,
//             button: Some("left"),
//             click_count: Some(1),
//         })
//         .unwrap();
//     self.chrome_browser.send_message(method_str);

//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(input::methods::DispatchMouseEvent {
//             event_type: "mouseReleased",
//             x: point.x,
//             y: point.y,
//             button: Some("left"),
//             click_count: Some(1),
//         })
//         .unwrap();
//     self.chrome_browser.send_message(method_str);
// }

// pub fn request_midpoint(&mut self, backend_node_id: dom::NodeId) {
//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(dom::methods::GetContentQuads {
//             node_id: None,
//             backend_node_id: Some(backend_node_id),
//             object_id: None,
//         })
//         .unwrap();
//     self.chrome_browser.send_message(method_str);

//     // let return_object = self.parent.call_method()?;
//     // let raw_quad = return_object.quads.first().unwrap();
//     // let input_quad = ElementQuad::from_raw_points(&raw_quad);
//     // Ok((input_quad.bottom_right + input_quad.top_left) / 2.0)
// }

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

// pub fn capture_screenshot(
//     &mut self,
//     format: page::ScreenshotFormat,
//     clip: Option<page::Viewport>,
//     from_surface: bool,
// ) /*-> Result<Vec<u8>, Error>*/
// {
//     let (format, quality) = match format {
//         page::ScreenshotFormat::JPEG(quality) => {
//             (page::InternalScreenshotFormat::JPEG, quality)
//         }
//         page::ScreenshotFormat::PNG => (page::InternalScreenshotFormat::PNG, None),
//     };

//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(page::methods::CaptureScreenshot {
//             format,
//             clip,
//             quality,
//             from_surface,
//         })
//         .unwrap();
//     // self.state = ChromeSessionState::WaitingScreenShot(mid.unwrap());
//     self.chrome_browser.send_message(method_str);
// }
// #[derive(Debug)]
// pub enum ChromeSessionState {
//     WaitingPageCreate,
//     WaitingPageAttach,
//     WaitingPageEnable(usize, String),
//     WaitingFrameTree(usize),
//     // AfterInvokeNavigate,
//     WaitingGetDocument(usize, Option<&'static str>),
//     WaitingDomQuerySelector(Option<&'static str>, usize, bool),
//     WaitingDescribeNode(Option<&'static str>, usize, dom::NodeId, bool),
//     WaitingRemoteObject(dom::NodeId, Option<&'static str>, usize),
//     WaitingModelBox(Option<&'static str>, dom::NodeId, usize),
//     WaitingScreenShot(usize),
//     Consuming,
// }

// why need a task_id? If invoked find_node('#a'), find_node('#b'), find_node('#c') at a time how can I distinguish which is which in the following loop? That's task_id for.
// Who drive the App to loop? They are tasks. For some reasons tasks execution must be delayed must waiting some events to happen, Store these tasks and execute lately.
// It's a must that we know how to play from a task_describe.

// pub fn force_next_stage(&mut self) {}

// pub fn attach_to_page(&mut self) {
//     let (_, method_str, _) = MethodUtil::create_msg_to_send(
//         target::methods::AttachToTarget {
//             target_id: &(self.target_info.as_mut().unwrap().target_id),
//             flatten: None,
//         },
//         MethodDestination::Browser,
//         None,
//     )
//     .unwrap();
//     self.state = ChromeSessionState::WaitingPageAttach; // change state first.
//     self.chrome_browser.send_message(method_str);
// }

// fn create_msg_to_send_with_session_id<C>(&self, method: C) -> MethodBeforSendResult
// where
//     C: protocol::Method + serde::Serialize,
// {
//     let session_id = self.session_id.as_ref().unwrap();
//     MethodUtil::create_msg_to_send(
//         method,
//         MethodDestination::Target(session_id.clone().into()),
//         None,
//     )
// }

// pub fn page_enable(&mut self, target_id: String) {
//     let (_, method_str, mid) = self
//         .create_msg_to_send_with_session_id(page::methods::Enable {})
//         .unwrap();
//     // self.state = ChromeSessionState::WaitingPageEnable(mid.unwrap(), target_id);
//     self.chrome_browser.send_message(method_str);
// }

// pub fn navigate_to(&mut self, url: &str) {
//     let (_, method_str, _) = self
//         .create_msg_to_send_with_session_id(Navigate { url })
//         .unwrap();
//     self.state = ChromeSessionState::Consuming;
//     self.send_message(method_str);
// }
// pub fn get_document(
//     &mut self,
//     manual_task_id: Option<ids::Task>,
// ) -> (Option<ids::Task>, Option<dom::NodeId>) {
//     if let Some(root_node) = &self.root_node {
//         (None, Some(root_node.node_id))
//     } else {
//         let this_id =
//             manual_task_id.unwrap_or(self.unique_number.fetch_add(1, Ordering::SeqCst));
//         let (_, method_str, mid) = self
//             .create_msg_to_send_with_session_id(dom::methods::GetDocument {
//                 depth: Some(0),
//                 pierce: Some(false),
//             })
//             .unwrap();
//         self.task_id_2_task
//             .insert(this_id, tasks::TaskDescribe::GetDocument(this_id));
//         self.method_id_2_task_id
//             .entry(mid.unwrap())
//             .or_insert(this_id);
//         self.chrome_browser.send_message(method_str);
//         (Some(this_id), None)
//     }
// }