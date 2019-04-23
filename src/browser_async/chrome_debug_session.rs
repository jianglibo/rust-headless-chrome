use super::task_describe::{self as tasks, TaskDescribe};
use super::id_type as ids;

use super::inner_event::{self, InnerEvent};
use super::page_message::ChangingFrame;
use crate::browser_async::chrome_browser::ChromeBrowser;

use crate::browser_async::dev_tools_method_util::{MethodUtil, MethodTuple};
use crate::browser::tab::element::{BoxModel, ElementQuad};
use crate::protocol::{self, dom, page, target};
use failure::Error;
use log::*;
use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize};
use websocket::futures::{Poll, Stream};
use std::convert::TryFrom;


#[derive(Debug)]
pub struct ChromeDebugSession {
    chrome_browser: ChromeBrowser,
    target_info: Option<protocol::target::TargetInfo>,
    session_id: Option<String>,
    method_id_2_task_id: HashMap<ids::Method, ids::Task>,
    task_id_2_task: HashMap<ids::Task, TaskDescribe>,
    // waiting_for_me: HashMap<ids::Task, Vec<ids::Task>>,
    unique_number: AtomicUsize,
    pending_tasks: VecDeque<ids::Task>,
}

impl ChromeDebugSession {
    pub fn new(chrome_browser: ChromeBrowser) -> Self {
        Self {
            chrome_browser,
            target_info: None,
            session_id: None,
            method_id_2_task_id: HashMap::new(),
            task_id_2_task: HashMap::new(),
            // waiting_for_me: HashMap::new(),
            unique_number: AtomicUsize::new(10000),
            pending_tasks: VecDeque::new(),
        }
    }

    pub fn method_id_2_task_id_remain(&self) -> usize {
        info!("{:?}", self.method_id_2_task_id);
        self.method_id_2_task_id.len()
    }

    pub fn task_id_2_task_remain(&self) -> usize {
        info!("{:?}", self.task_id_2_task);
        self.task_id_2_task.len()
    }

    pub fn pending_tasks_remain(&self) -> usize {
        info!("{:?}", self.pending_tasks);
        self.pending_tasks.len()
    }

    // pub fn waiting_for_me_remain(&self) -> usize {
    //     info!("{:?}", self.waiting_for_me);
    //     self.waiting_for_me.len()
    // }

    pub fn dom_query_selector_to_str(&mut self, task: &TaskDescribe) {
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
            self.send_message_direct(method_str);
        } else {
            error!("maybe node_id to select with is None.");
        }
    }

    pub fn send_message_and_save_task(&mut self, method_str_id: Option<(String, usize)>, task_id: ids::Task, task: TaskDescribe) {
        if let Some((method_str, mid)) = method_str_id {
            self.add_task_and_method_map(
                mid,
                task_id,
                task,
            );
            if self.pending_tasks.is_empty() {
                trace!("**sending** call_id: {:?}, call content: {:?}", mid, method_str);
                self.chrome_browser.send_message(method_str);
            }
        } else {
            self.add_task(task_id, task);
        }
        self.pending_tasks.push_back(task_id);
    }


    pub fn send_message_direct(&mut self, method_str: String) {
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
        trace!("add_method_task_map: {:?} -> {:?}", mid, task_id);
        self.add_task(task_id, task);
    }

    // pub fn add_waiting_task(
    //     &mut self,
    //     task_id_to_waiting_for: ids::Task,
    //     waiting_task_id: ids::Task,
    // ) {
    //     self.waiting_for_me
    //         .entry(task_id_to_waiting_for)
    //         .or_insert_with(||vec![])
    //         .push(waiting_task_id);
    //     info!("waiting_for_me: {:?}", self.waiting_for_me);
    // }


    // fn get_waiting_tasks(&mut self, task_id: ids::Task) -> Vec<TaskDescribe> {
    //     // Take out all tasks waiting for me.
    //     let waiting_task_ids: Vec<_> = self
    //         .waiting_for_me
    //         .get_mut(&task_id)
    //         .unwrap_or(&mut vec![])
    //         .drain(..)
    //         .collect();
    //     // Remove task_id task pair.
    //     waiting_task_ids
    //         .iter()
    //         .flat_map(|t_id| self.task_id_2_task.remove(&t_id))
    //         .collect()
    // }

    pub fn dom_describe_node(&mut self, task: &TaskDescribe) {
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
            self.send_message_direct(method_str);
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


    // pub fn feed_on_node_id(&mut self, task_id: ids::Task, node_id: Option<dom::NodeId>) -> Option<TaskDescribe> {
    //     let mut waiting_tasks = self.get_waiting_tasks(task_id);
    //     while let Some(mut task) = waiting_tasks.pop() {
    //         if node_id.is_none() {
    //             return Some(task);
    //         }
    //         match &mut task {
    //             TaskDescribe::QuerySelector(query_selector) => {
    //                 query_selector.node_id = node_id;
    //                 self.dom_query_selector(task);
    //             }
    //             TaskDescribe::DescribeNode(describe_node) => {
    //                 describe_node.node_id = node_id;
    //                 self.dom_describe_node(task);
    //             }
    //             TaskDescribe::GetBoxModel(get_box_model) => {
    //                 get_box_model.node_id = node_id;
    //                 self.get_box_model(task);
    //             }
    //             _ => (),
    //         }
    //     }
    //     None
    // }

    // pub fn feed_on_box_model(&mut self, task_id: ids::Task, box_model: BoxModel) {
    //     let mut waiting_tasks = self.get_waiting_tasks(task_id);
    //     while let Some(mut task) = waiting_tasks.pop() {
    //         match &mut task {
    //             tasks::TaskDescribe::ScreenShot(screen_shot) => {
    //                 screen_shot.clip = Some(box_model.content_viewport());
    //                 self.capture_screen_shot(task);
    //             }
    //             _ => (),
    //         }
    //     }
    // }
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

    pub fn parse_response_error(response: protocol::Response) -> Result<protocol::Response, Error> {
        if let Some(error) = response.error {
            Err(error.into())
        } else {
            Ok(response)
        }
    }

    pub fn after_get_document(&mut self,next_task_id: ids::Task, node_id: dom::NodeId) {
        if let Some(mut next_task) = self.task_id_2_task.get_mut(&next_task_id){
            match &mut next_task {
                tasks::TaskDescribe::QuerySelector(query_selector) => {
                    query_selector.node_id = Some(node_id);
                    let (_, method_str, _) = MethodTuple::try_from(&*next_task).unwrap();
                    self.send_message_direct(method_str);
                }
                tasks::TaskDescribe::DescribeNode(describe_node) => {
                    describe_node.node_id = Some(node_id);
                    let (_, method_str, _) = MethodTuple::try_from(&*next_task).unwrap();
                    self.send_message_direct(method_str);
                }
                _ => (),
            }
        } else {
            error!("cannot find task in task_id_2_task: {:?}", next_task_id);
        }       
    }

    #[allow(clippy::single_match_else)]
    fn process_pending_tasks(&mut self,task_id: ids::Task, current_task: &TaskDescribe) {
        if let Some(pending_task_id) = self.pending_tasks.pop_front() {
            if pending_task_id == task_id {
                if let Some(next_id) = self.pending_tasks.front() {
                    // it doesn't matter taking task out of task_id_2_task, I can put it back again.
                        match current_task {
                            TaskDescribe::GetDocument(get_document) => {
                                let nd = get_document.root_node.as_ref().unwrap().node_id;
                                self.after_get_document(*next_id, nd);
                            }
                            _ => {
                                warn!("unprocessed after task: {:?}", current_task);
                            }
                        }

                } else {
                    trace!("no pending tasks.");
                }
            } else {
                error!("unmatched task ids, pending_task_id: {:?}, this task id: {:?}", pending_task_id, task_id);
            }
        } else {
            error!("missing pending task_id: {:?}", task_id);
        }
    }


    pub fn handle_response(
        &mut self,
        resp: protocol::Response,
        session_id: Option<String>,
        target_id: Option<String>,
    ) -> Option<TaskDescribe> {
        trace!("got **response**. {:?}", resp);
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
        // already remove from method_id_2_task_id and task_id_2_task!!!
        
        // if let Some(error) = resp.error {
        //     return Err(error.into());
        // }

        // message: "{\"id\":6,\"result\":{\"root\":{\"nodeId\":1,\"backendNodeId\":3,\"nodeType\":9,\"nodeName\":\"#document\",\"localName\":\"\",\"nodeValue\":\"\",\"childNodeCount\":2,\"documentURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"baseURL\":\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\",\"xmlVersion\":\"\"}}}"
        // if json_assistor::response_result_field_has_properties(&resp, "root", vec!["nodeId", "backendNodeId"]) {
        //    let node: dom::methods::GetDocumentReturnObject = serde_json::from_value(resp.result.unwrap()).unwrap();
        // }

        if let Some(task) = maybe_matched_task {
            match task {
                TaskDescribe::GetDocument(get_document) => {
                    // it must be a GetDocumentReturnObject or else something must go wrong.
                    match protocol::parse_response::<dom::methods::GetDocumentReturnObject>(resp) {
                        Ok(get_document_return_object) => {
                            // let node_id = get_document_return_object.root.node_id;
                            // self.feed_on_root_node_id(task_id, node_id);
                            let t = TaskDescribe::GetDocument(tasks::GetDocument {
                                root_node: Some(get_document_return_object.root),
                                ..get_document
                            });
                            self.process_pending_tasks(get_document.task_id, &t);
                            return Some(t);
                        }
                        Err(remote_error) => panic!("{:?}", remote_error)
                    }
                }
                TaskDescribe::PageEnable(task_id, target_id, session_id) => {
                    let t = TaskDescribe::PageEnable(task_id, target_id, session_id);
                    self.process_pending_tasks(task_id, &t);
                    return Some(t);
                }
                TaskDescribe::QuerySelector(query_selector) => {
                    match protocol::parse_response::<dom::methods::QuerySelectorReturnObject>(resp) {
                        Ok(query_select_return_object) => {
                            let t = TaskDescribe::QuerySelector(tasks::QuerySelector {
                                    found_node_id: Some(query_select_return_object.node_id),
                                    ..query_selector
                                });
                            // self.feed_on_node_id(
                            //     query_selector.task_id,
                            //     Some(query_select_return_object.node_id),
                            // );
                            self.process_pending_tasks(query_selector.task_id, &t);
                            if query_selector.is_manual {
                                return Some(t);
                            }
                        }
                        Err(remote_error) => {
                            error!("{:?}, {:?}", query_selector, remote_error);
                            // if let Some(tk) = self.feed_on_node_id(
                            //     query_selector.task_id,
                            //     None,
                            // ) {
                            //     return Some(tk);
                            // }
                            return Some(TaskDescribe::QuerySelector(query_selector));
                        }
                    }
                }

                TaskDescribe::DescribeNode(mut describe_node) => {
                    match protocol::parse_response::<dom::methods::DescribeNodeReturnObject>(resp) {
                        Ok(describe_node_return_object) => {
                            if describe_node.is_manual {
                                describe_node.found_node = Some(describe_node_return_object.node);
                                let task_id = describe_node.task_id;
                                let t = TaskDescribe::DescribeNode(describe_node);
                                self.process_pending_tasks(task_id, &t);
                                return Some(t);
                            }
                        }
                        Err(remote_error) => {
                            error!("{:?}, {:?}",describe_node, remote_error);
                            return Some(TaskDescribe::DescribeNode(describe_node));
                        }
                    }
                }

                TaskDescribe::GetBoxModel(mut get_box_model) => {
                    match protocol::parse_response::<dom::methods::GetBoxModelReturnObject>(resp) {
                        Ok(get_box_model_return_object) => {
                            let raw_model = get_box_model_return_object.model;
                            let model_box = BoxModel {
                                content: ElementQuad::from_raw_points(&raw_model.content),
                                padding: ElementQuad::from_raw_points(&raw_model.padding),
                                border: ElementQuad::from_raw_points(&raw_model.border),
                                margin: ElementQuad::from_raw_points(&raw_model.margin),
                                width: raw_model.width,
                                height: raw_model.height,
                            };
                            let is_manual = get_box_model.is_manual;
                            get_box_model.found_box = Some(model_box);
                            let task_id = get_box_model.task_id;
                            let t = TaskDescribe::GetBoxModel(get_box_model);
                            self.process_pending_tasks(task_id, &t);
                            // self.feed_on_box_model(get_box_model.task_id, model_box.clone());
                            if is_manual {
                                return Some(t);
                            }
                        }
                        Err(remote_error) => {
                            error!("{:?}", remote_error);
                            return Some(TaskDescribe::GetBoxModel(get_box_model));
                        }
                    }
                }
                TaskDescribe::ScreenShot(mut screen_shot) => {
                    match protocol::parse_response::<page::methods::CaptureScreenshotReturnObject>(resp) {
                        Ok(capture_screenshot_return_object) => {
                            screen_shot.base64 = Some(capture_screenshot_return_object.data);
                            let task_id = screen_shot.task_id;
                            let t = TaskDescribe::ScreenShot(screen_shot);
                            self.process_pending_tasks(task_id, &t);
                            return Some(t);
                        }
                        Err(remote_error) => {
                            error!("{:?}", remote_error);
                            return Some(TaskDescribe::ScreenShot(screen_shot));
                        }
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
