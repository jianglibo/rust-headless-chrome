use super::id_type as ids;
use super::task_describe::TaskDescribe;

use super::dev_tools_method_util::ChromePageError;
use super::inner_event::{self, InnerEvent};
use crate::browser_async::chrome_browser::ChromeBrowser;

use crate::browser::tab::element::{BoxModel, ElementQuad};
use crate::protocol::{self, dom, page, runtime, target};
use failure::Error;
use log::*;
use std::convert::TryFrom;
use std::sync::atomic::AtomicUsize;
use websocket::futures::{Poll, Stream};

#[derive(Debug)]
pub struct ChromeDebugSession {
    chrome_browser: ChromeBrowser,
    session_id: Option<String>,
    unique_number: AtomicUsize,
    tasks_waiting_for_response: Vec<Vec<TaskDescribe>>,
}

impl ChromeDebugSession {
    pub fn new(chrome_browser: ChromeBrowser) -> Self {
        Self {
            chrome_browser,
            session_id: None,
            unique_number: AtomicUsize::new(10000),
            tasks_waiting_for_response: Vec::new(),
        }
    }

    pub fn tasks_waiting_for_response_count(&self) -> usize {
        self.tasks_waiting_for_response.len()
    }

    pub fn execute_task(&mut self, tasks: Vec<TaskDescribe>) {
        if let Some(task_ref) = tasks.get(0) {
            match String::try_from(task_ref) {
                Ok(method_str) => {
                    self.tasks_waiting_for_response.push(tasks);
                    self.chrome_browser.send_message(method_str);
                }
                Err(err) => error!("first task deserialize fail: {:?}", err),
            }
        } else {
            error!("empty tasks list.")
        }
    }

    pub fn send_message_direct(&mut self, method_str: String) {
        self.chrome_browser.send_message(method_str);
    }

    pub fn resolve_node(&mut self) -> (Option<ids::Task>, Option<ids::RemoteObject>) {
        (None, None)
    }

    #[allow(clippy::single_match_else)]
    #[allow(unreachable_patterns)]
    pub fn handle_inner_target_events(
        &mut self,
        inner_event: InnerEvent,
        session_id: Option<String>,
        target_id: Option<target::TargetId>,
    ) -> Option<TaskDescribe> {
        match inner_event {
            InnerEvent::SetChildNodes(set_child_nodes_event) => {
                let params = set_child_nodes_event.params;
                return TaskDescribe::SetChildNodes(
                    target_id.unwrap(),
                    params.parent_id,
                    params.nodes,
                )
                .into();
            }
            InnerEvent::LoadEventFired(load_event_fired_event) => {
                let params = load_event_fired_event.params;
                return TaskDescribe::LoadEventFired(target_id.unwrap(), params.timestamp).into();
            }
            InnerEvent::ExecutionContextCreated(execution_context_created) => {
                return TaskDescribe::RuntimeExecutionContextCreated(
                    execution_context_created.params.context,
                    (session_id, target_id).into(),
                )
                .into();
            }
            InnerEvent::ExecutionContextDestroyed(execution_context_destroyed) => {
                return TaskDescribe::RuntimeExecutionContextDestroyed(
                    execution_context_destroyed.params.execution_context_id,
                    (session_id, target_id).into(),
                )
                .into();
            }
            InnerEvent::ConsoleAPICalled(console_api_called) => {
                return TaskDescribe::RuntimeConsoleAPICalled(
                    console_api_called.params,
                    (session_id, target_id).into(),
                )
                .into();
            }
            InnerEvent::DomContentEventFired(dom_content_event_fired) => {
                trace!("{:?}", dom_content_event_fired.params);
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

    pub fn handle_response(
        &mut self,
        resp: protocol::Response,
        _session_id: Option<String>,
        _target_id: Option<String>,
    ) -> Option<TaskDescribe> {
        trace!("got **response**. {:?}", resp);
        let call_id = resp.call_id;
        trace!(
            "tasks_waiting_for_response: {:?}",
            self.tasks_waiting_for_response
        );
        if let Some(idx) = self.tasks_waiting_for_response.iter().position(|v| {
            if let Some(it) = v.get(0) {
                if let Some(cf) = it.get_common_fields() {
                    cf.call_id == call_id
                } else {
                    false
                }
            } else {
                false
            }
        }) {
            let mut tasks = self.tasks_waiting_for_response.remove(idx);
            let mut current_task = tasks.remove(0);

            // if has remote error.
            if resp.error.is_some() {
                error!("got remote error: {:?}", resp);
                let t = if let Some(tk) = tasks.pop() {
                    tk
                } else {
                    current_task
                };
                error!("the task: {:?}", t);
                return Some(t);
            }

            if let Err(err) = self.full_fill_task(resp, &mut current_task) {
                error!("fulfill task failed. {:?}", err);
                // return last task (not fulfilled) in vec.
                if let Some(tk) = tasks.pop() {
                    return Some(tk);
                } else {
                    return Some(current_task);
                }
            }
            if tasks.is_empty() {
                return Some(current_task);
            }
            if let Err(mut err) = self.handle_next_task(current_task, tasks) {
                if let Some(ChromePageError::NextTaskExecution { tasks, error }) =
                    err.downcast_mut::<ChromePageError>()
                {
                    error!("handle next task fail. {:?}, {:?}", tasks, error);
                    return (*tasks).pop();
                } else {
                    panic!("handle next task fail. {:?}", err);
                }
            }
        } else {
            trace!("no matching task for call_id: {:?}", call_id);
        }
        None
    }

    fn execute_next_and_return_remains(
        &mut self,
        tasks: Vec<TaskDescribe>,
    ) -> Result<(), failure::Error> {
        let next_task = tasks.get(0).unwrap();
        match String::try_from(next_task) {
            Ok(method_str) => {
                self.tasks_waiting_for_response.push(tasks);
                self.send_message_direct(method_str);
                Ok(())
            }
            Err(error) => Err(ChromePageError::NextTaskExecution { tasks, error }.into()),
        }
    }

    fn handle_next_task(
        &mut self,
        current_task: TaskDescribe,
        mut tasks: Vec<TaskDescribe>,
    ) -> Result<(), failure::Error> {
        let mut next_task = tasks.get_mut(0).unwrap();
        match (&current_task, &mut next_task) {
            (
                TaskDescribe::GetDocument(get_document),
                TaskDescribe::QuerySelector(query_selector),
            ) => {
                query_selector.node_id = get_document
                    .task_result
                    .as_ref()
                    .and_then(|nd| Some(nd.node_id));
                return self.execute_next_and_return_remains(tasks);
            }
            (
                TaskDescribe::QuerySelector(query_selector),
                TaskDescribe::DescribeNode(describe_node),
            ) => {
                describe_node.node_id = query_selector.task_result;
                return self.execute_next_and_return_remains(tasks);
            }
            (
                TaskDescribe::QuerySelector(query_selector),
                TaskDescribe::GetBoxModel(get_box_model),
            ) => {
                get_box_model.node_id = query_selector.task_result;
                return self.execute_next_and_return_remains(tasks);
            }
            (TaskDescribe::GetBoxModel(get_box_model), TaskDescribe::CaptureScreenshot(screen_shot)) => {
                if let Some(mb) = &get_box_model.task_result {
                    let viewport = mb.content_viewport();
                    screen_shot.clip = Some(viewport);
                    return self.execute_next_and_return_remains(tasks);
                } else {
                    failure::bail!("found_box is None!");
                }
            }
            _ => {
                error!("unknown pair: {:?}, {:?}", current_task, next_task);
            }
        }
        Ok(())
    }

    fn full_fill_task(
        &self,
        resp: protocol::Response,
        mut task: &mut TaskDescribe,
    ) -> Result<(), failure::Error> {
        match &mut task {
            TaskDescribe::GetDocument(get_document) => {
                let get_document_return_object =
                    protocol::parse_response::<dom::methods::GetDocumentReturnObject>(resp)?;
                get_document.task_result = Some(get_document_return_object.root);
            }
            TaskDescribe::PageEnable(_common_fields) => {}
            TaskDescribe::QuerySelector(query_selector) => {
                let query_select_return_object =
                    protocol::parse_response::<dom::methods::QuerySelectorReturnObject>(resp)?;
                query_selector.task_result= Some(query_select_return_object.node_id);
            }
            TaskDescribe::DescribeNode(describe_node) => {
                let describe_node_return_object =
                    protocol::parse_response::<dom::methods::DescribeNodeReturnObject>(resp)?;
                describe_node.task_result= Some(describe_node_return_object.node);
            }
            TaskDescribe::GetBoxModel(get_box_model) => {
                let get_box_model_return_object =
                    protocol::parse_response::<dom::methods::GetBoxModelReturnObject>(resp)?;
                let raw_model = get_box_model_return_object.model;
                let model_box = BoxModel {
                    content: ElementQuad::from_raw_points(&raw_model.content),
                    padding: ElementQuad::from_raw_points(&raw_model.padding),
                    border: ElementQuad::from_raw_points(&raw_model.border),
                    margin: ElementQuad::from_raw_points(&raw_model.margin),
                    width: raw_model.width,
                    height: raw_model.height,
                };
                get_box_model.task_result = Some(model_box);
            }
            TaskDescribe::CaptureScreenshot(screen_shot) => {
                let capture_screenshot_return_object =
                    protocol::parse_response::<page::methods::CaptureScreenshotReturnObject>(resp)?;
                screen_shot.task_result= Some(capture_screenshot_return_object.data);
            }
            TaskDescribe::RuntimeEvaluate(runtime_evaluate) => {
                let evaluate_return_object =
                    protocol::parse_response::<runtime::methods::EvaluateReturnObject>(resp)?;
                runtime_evaluate.task_result = Some(evaluate_return_object.result);
                runtime_evaluate.exception_details = evaluate_return_object.exception_details;
            }
            TaskDescribe::NavigateTo(navigate_to) => {
                let navigate_to_return_object = protocol::parse_response::<page::methods::NavigateReturnObject>(resp)?;
                navigate_to.task_result = Some(navigate_to_return_object);
            }
            TaskDescribe::RuntimeEnable(common_fields) => {
                trace!("runtime enabled: {:?}", common_fields);
            }
            TaskDescribe::RuntimeGetProperties(get_properties) => {
                let get_properties_return_object = protocol::parse_response::<runtime::methods::GetPropertiesReturnObject>(resp)?;
                get_properties.task_result = Some(get_properties_return_object);
            }
            TaskDescribe::RuntimeCallFunctionOn(task) => {
                let task_return_object = protocol::parse_response::<runtime::methods::CallFunctionOnReturnObject>(resp)?;
                task.task_result = Some(task_return_object);
            }
            TaskDescribe::PrintToPDF(task) => {
                let task_return_object = protocol::parse_response::<page::methods::PrintToPdfReturnObject>(resp)?;
                task.task_result = Some(task_return_object.data);
            }
            task_describe => {
                warn!("got unprocessed task_describe: {:?}", task_describe);
            }
        }
        Ok(())
    }

    #[allow(clippy::single_match)]
    fn handle_protocol_event(
        &mut self,
        protocol_event: protocol::Event,
        session_id: Option<String>,
        target_id: Option<String>,
    ) -> Option<TaskDescribe> {
        match protocol_event {
            protocol::Event::FrameNavigated(frame_navigated_event) => {
                // let changing_frame = ChangingFrame::Navigated(frame_navigated_event.params.frame);
                return Some(TaskDescribe::FrameNavigated(
                    Box::new(frame_navigated_event.params.frame),
                    (session_id, target_id).into(),
                    // Box::new(changing_frame),
                ));
            }
            protocol::Event::TargetInfoChanged(target_info_changed) => {
                return Some(TaskDescribe::TargetInfoChanged(
                    target_info_changed.params.target_info,
                    (session_id, target_id).into(),
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
                        trace!(
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
            protocol::Event::FrameAttached(evt) => {
                return Some(TaskDescribe::FrameAttached(
                    evt.params,
                    (session_id, target_id).into(),
                ));
            }
            protocol::Event::FrameStoppedLoading(evt) => {
                let frame_id = evt.params.frame_id;
                return Some(TaskDescribe::FrameStoppedLoading(
                    frame_id,
                    (session_id, target_id).into(),
                ));
            }
            protocol::Event::FrameStartedLoading(evt) => {
                let frame_id = evt.params.frame_id;
                return Some(TaskDescribe::FrameStartedLoading(
                    frame_id,
                    (session_id, target_id).into(),
                ));
            }
            protocol::Event::FrameDetached(evt) => {
                let frame_id = evt.params.frame_id;
                return Some(TaskDescribe::FrameDetached(
                    frame_id,
                    (session_id, target_id).into(),
                ));
            }
            _ => {
                warn!("unprocessed inner event: {:?}", protocol_event);
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
                            _ => match parse_raw_message(&message_field) {
                                Ok(inner_event::InnerEventWrapper::InnerEvent(inner_event)) => {
                                    trace!("got inner event: {:?}", inner_event);
                                    if let Some(page_message) = self.handle_inner_target_events(
                                        inner_event,
                                        Some(session_id),
                                        Some(target_id),
                                    ) {
                                        break Ok(Some(page_message).into());
                                    }
                                }
                                Err(_error) => {
                                    error!(
                                        "parse_raw_message failed ** {:?}",
                                        message_field
                                    );
                                }
                            },
                        }
                    }
                    protocol::Message::Event(protocol_event) => {
                        if let Some(page_message) =
                            self.handle_protocol_event(protocol_event, None, None)
                        {
                            break Ok(Some(page_message).into());
                        }
                    }
                    protocol::Message::Connected => {
                        break Ok(Some(TaskDescribe::ChromeConnected).into());
                    }
                    //   pub enum Event {
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
                        warn!("got unknown message1: {:?}", other);
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
