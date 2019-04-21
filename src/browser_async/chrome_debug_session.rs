use super::task_describe::{self as tasks, TaskDescribe};
use super::id_type as ids;

use super::inner_event::{self, InnerEvent};
use super::page_message::ChangingFrame;
use crate::browser_async::chrome_browser::ChromeBrowser;

use crate::browser_async::dev_tools_method_util::MethodUtil;
use crate::browser::tab::element::{BoxModel, ElementQuad};
use crate::protocol::{self, dom, page, target};
use failure::Error;
use log::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize};
use websocket::futures::{Poll, Stream};

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
        info!("add task is and task: {:?}, {:?}", task_id, task);
        self.task_id_2_task.insert(task_id, task);
    }

    pub fn add_method_task_map(&mut self, mid: usize, task_id: ids::Task) {
        if self.method_id_2_task_id.contains_key(&mid) {
            warn!("mid already exists in map. {:?}", mid);
        }
        info!("insert mid task_id pair: ({:?}, {:?})", mid, task_id);
        self.method_id_2_task_id.entry(mid).or_insert(task_id);
    }

    pub fn add_task_and_method_map(
        &mut self,
        mid: ids::Method,
        task_id: ids::Task,
        task: TaskDescribe,
    ) {
        self.add_method_task_map(mid, task_id);
        self.add_task(task_id, task);
    }

    pub fn add_waiting_task(
        &mut self,
        task_id_to_waiting_for: ids::Task,
        waiting_task_id: ids::Task,
    ) {
        self.waiting_for_me
            .entry(task_id_to_waiting_for)
            .or_insert_with(||vec![])
            .push(waiting_task_id);
    }


    fn get_waiting_tasks(&mut self, task_id: ids::Task) -> Vec<TaskDescribe> {
        // Take out all tasks waiting for me.
        let waiting_task_ids: Vec<_> = self
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

    pub fn dom_describe_node(&mut self, task: TaskDescribe) {
        if let TaskDescribe::DescribeNode(tasks::DescribeNode {
            task_id,
            node_id,
            backend_node_id,
            depth,
            session_id,
            ..
        }) = &task
        {
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(
                dom::methods::DescribeNode {
                    node_id: *node_id,
                    backend_node_id: *backend_node_id,
                    depth: *depth,
                },
                &session_id,
            )
            .unwrap();
            self.add_task_and_method_map(mid.unwrap(), task_id.clone(), task);
            self.chrome_browser.send_message(method_str);
        } else {
            error!("not a node_describe.")
        }
    }

    pub fn get_box_model(&mut self, task: TaskDescribe) {
        if let TaskDescribe::GetBoxModel(tasks::GetBoxModel {
            task_id,
            node_id,
            backend_node_id,
            session_id,
            ..
        }) = &task
        {
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(
                dom::methods::GetBoxModel {
                    node_id: *node_id,
                    backend_node_id: *backend_node_id,
                    object_id: None,
                },
                &session_id,
            )
            .unwrap();
            self.add_task_and_method_map(mid.unwrap(), *task_id, task);
            self.chrome_browser.send_message(method_str);
        } else {
            error!("not a get_box_model.")
        }
    }

    pub fn capture_screen_shot(&mut self, task: TaskDescribe) {
        if let TaskDescribe::ScreenShot(sh) = task {
            let cloned_sh = sh.clone();

            let (format, quality) = match sh.format {
                page::ScreenshotFormat::JPEG(quality) => {
                    (page::InternalScreenshotFormat::JPEG, quality)
                }
                page::ScreenshotFormat::PNG => (page::InternalScreenshotFormat::PNG, None),
            };
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(
                page::methods::CaptureScreenshot {
                    format,
                    quality,
                    clip: sh.clip,
                    from_surface: sh.from_surface,
                },
                &sh.session_id,
            )
            .unwrap();
            self.add_task_and_method_map(
                mid.unwrap(),
                sh.task_id,
                TaskDescribe::ScreenShot(cloned_sh),
            );
            self.chrome_browser.send_message(method_str);
        } else {
            error!("not a capture_screen_shot.")
        }
    }

    pub fn resolve_node(&mut self) -> (Option<ids::Task>, Option<ids::RemoteObject>) {
        (None, None)
    }

    pub fn feed_on_root_node_id(&mut self, task_id: ids::Task, node_id: dom::NodeId) {
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
                tasks::TaskDescribe::GetBoxModel(get_box_model) => {
                    get_box_model.node_id = Some(node_id);
                    self.get_box_model(task);
                }
                _ => (),
            }
        }
    }

    pub fn feed_on_box_model(&mut self, task_id: ids::Task, box_model: BoxModel) {
        let mut waiting_tasks = self.get_waiting_tasks(task_id);
        while let Some(mut task) = waiting_tasks.pop() {
            match &mut task {
                tasks::TaskDescribe::ScreenShot(screen_shot) => {
                    screen_shot.clip = Some(box_model.content_viewport());
                    self.capture_screen_shot(task);
                }
                _ => (),
            }
        }
    }
    // pub fn feed_on_node(&mut self, task_id: ids::Task, node: dom::Node) {
    //     let mut waiting_tasks = self.get_waiting_tasks(task_id);
    //     while let Some(mut task) = waiting_tasks.pop() {
    //         match &mut task {
    //             tasks::TaskDescribe::QuerySelector(query_selector) => {
    //                 query_selector.node_id = Some(node_id);
    //                 self.dom_query_selector(task);
    //             }
    //             tasks::TaskDescribe::DescribeNode(describe_node) => {
    //                 describe_node.node_id = Some(node_id);
    //                 self.dom_describe_node(task);
    //             }
    //             _ => (),
    //         }
    //     }
    // }

    #[allow(clippy::single_match_else)]
    pub fn handle_inner_target_events(
        &mut self,
        inner_event: InnerEvent,
        _raw_session_id: String,
        target_id: target::TargetId,
    ) -> Option<TaskDescribe> {
        match inner_event {
            InnerEvent::SetChildNodes(set_child_nodes_event) => {
                let params = set_child_nodes_event.params;
                return TaskDescribe::SetChildNodes(target_id, params.parent_id, params.nodes)
                    .into();
            }
            InnerEvent::LoadEventFired(load_event_fired_event) => {
                let params = load_event_fired_event.params;
                return TaskDescribe::LoadEventFired(target_id, params.timestamp).into();
            }
            _ => {
                info!("discard inner event: {:?}", inner_event);
            }
        }
        None
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
            .or_else(|| {
                error!(
                    "not matching task_id to call_id {}. resp {:?}",
                    call_id, resp
                );
                None
            });

        // message: "{\"id\":6,\"result\":{\"root\":{\"nodeId\":1,\"backendNodeId\":3,\"nodeType\":9,\"nodeName\":\"#document\",\"localName\":\"\",\"nodeValue\":\"\",\"childNodeCount\":2,\"documentURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"baseURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"xmlVersion\":\"\"}}}"
        // if json_assistor::response_result_field_has_properties(&resp, "root", vec!["nodeId", "backendNodeId"]) {
        //    let node: dom::methods::GetDocumentReturnObject = serde_json::from_value(resp.result.unwrap()).unwrap();
        // }

        if let Some(task) = maybe_matched_task {
            match task {
                TaskDescribe::GetDocument(task_id, t_id, _) => {
                    // it must be a GetDocumentReturnObject or else something must go wrong.
                    if let Some(result) = resp.result {
                        if let Ok(get_document_return_object) =
                            serde_json::from_value::<dom::methods::GetDocumentReturnObject>(result)
                        {
                            let node_id = get_document_return_object.root.node_id;
                            self.feed_on_root_node_id(task_id, node_id);
                            return Some(TaskDescribe::GetDocument(
                                task_id,
                                t_id,
                                Some(get_document_return_object.root),
                            ));
                        } else {
                            panic!("GetDocument failed.");
                        }
                    } else {
                        panic!("GetDocument respose has None result.");
                    }
                }
                TaskDescribe::PageEnable(task_id, target_id, session_id) => {
                    return Some(TaskDescribe::PageEnable(task_id, target_id, session_id));
                }
                TaskDescribe::QuerySelector(query_selector) => {
                    if let Some(result) = resp.result {
                        if let Ok(query_select_return_object) = serde_json::from_value::<
                            dom::methods::QuerySelectorReturnObject,
                        >(result)
                        {
                            self.feed_on_node_id(
                                query_selector.task_id,
                                query_select_return_object.node_id,
                            );
                            if query_selector.is_manual {
                                // query_selector.found_node_id.replace(query_select_return_object.node_id);
                                return Some(TaskDescribe::QuerySelector(tasks::QuerySelector {
                                    found_node_id: Some(query_select_return_object.node_id),
                                    ..query_selector
                                }));
                            }
                        } else {
                            panic!("QuerySelector failed.");
                        }
                    } else {
                        panic!("QuerySelector response has None result");
                    }

                }

                TaskDescribe::DescribeNode(mut describe_node) => {
                    if let Some(result) = resp.result {
                        if let Ok(describe_node_return_object) =
                            serde_json::from_value::<dom::methods::DescribeNodeReturnObject>(result)
                        {
                            if describe_node.is_manual {
                                describe_node.found_node = Some(describe_node_return_object.node);
                                return Some(TaskDescribe::DescribeNode(describe_node));
                            }
                        } else {
                            panic!("DescribeNode failed.");
                        }
                    } else {
                        panic!("DescribeNode response has None result {:?}", resp);
                    }

                }

                TaskDescribe::GetBoxModel(mut get_box_model) => {
                    if let Some(result) = resp.result {
                        if let Ok(get_box_model_return_object) =
                            serde_json::from_value::<dom::methods::GetBoxModelReturnObject>(result)
                        {
                            let raw_model = get_box_model_return_object.model;
                            let model_box = BoxModel {
                                content: ElementQuad::from_raw_points(&raw_model.content),
                                padding: ElementQuad::from_raw_points(&raw_model.padding),
                                border: ElementQuad::from_raw_points(&raw_model.border),
                                margin: ElementQuad::from_raw_points(&raw_model.margin),
                                width: raw_model.width,
                                height: raw_model.height,
                            };
                            self.feed_on_box_model(get_box_model.task_id, model_box.clone());
                            if get_box_model.is_manual {
                                get_box_model.found_box = Some(model_box);
                                return Some(TaskDescribe::GetBoxModel(get_box_model));
                            }
                        } else {
                            panic!("GetBoxModel failed.");
                        }
                    } else {
                        panic!("GetBoxModel response has None result");
                    }

                }

                TaskDescribe::ScreenShot(mut screen_shot) => {
                    if let Some(result) = resp.result {
                        if let Ok(get_box_model_return_object) = serde_json::from_value::<
                            page::methods::CaptureScreenshotReturnObject,
                        >(result)
                        {
                            screen_shot.base64 = Some(get_box_model_return_object.data);
                            return Some(TaskDescribe::ScreenShot(screen_shot));
                        } else {
                            panic!("ScreenShop failed.");
                        }
                    } else {
                        panic!("Screenshot response has None result");
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

pub fn parse_raw_message(raw_message: &str) -> Result<inner_event::InnerEventWrapper, Error> {
    Ok(serde_json::from_str::<inner_event::InnerEventWrapper>(
        raw_message,
    )?)
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
                                if let Ok(inner_event::InnerEventWrapper::InnerEvent(inner_event)) =
                                    parse_raw_message(&message_field)
                                {
                                    info!("got inner event: {:?}", inner_event);
                                    if let Some(page_message) = self.handle_inner_target_events(
                                        inner_event,
                                        session_id,
                                        target_id,
                                    ) {
                                        break Ok(Some(page_message).into());
                                    }
                                } else {
                                    error!("unprocessed ** {:?}", message_field);
                                }
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
