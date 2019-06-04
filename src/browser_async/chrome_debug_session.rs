use super::task_describe::{TaskDescribe, HasCallId, TargetCallMethodTask, BrowserCallMethodTask, 
page_events, target_events, dom_events, runtime_events, network_events};

use super::embedded_events::{self, EmbeddedEvent};
use crate::browser_async::{chrome_browser::ChromeBrowser, TaskId, ChromePageError};

use crate::browser::tab::element::{BoxModel, ElementQuad};
use crate::protocol::{self, dom, page, runtime, target, network};
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

    pub fn resolve_node(&mut self) -> (Option<TaskId>, Option<runtime::RemoteObjectId>) {
        (None, None)
    }

    #[allow(clippy::single_match_else)]
    #[allow(unreachable_patterns)]
    pub fn handle_inner_target_events(
        &mut self,
        target_message_event: EmbeddedEvent,
        session_id: Option<String>,
        target_id: Option<target::TargetId>,
    ) -> Option<TaskDescribe> {
        match target_message_event {
            EmbeddedEvent::SetChildNodes(embedded_event) => {
                let event = dom_events::SetChildNodes::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::LoadEventFired(embedded_event) => {
                let event = page_events::LoadEventFired::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::ExecutionContextCreated(embedded_event) => {
                let event = runtime_events::ExecutionContextCreated::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::ExecutionContextDestroyed(embedded_event) => {
                let event = runtime_events::ExecutionContextDestroyed::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::ConsoleAPICalled(embedded_event) => {
                let event = runtime_events::ConsoleAPICalled::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::DomContentEventFired(embedded_event) => {
                let event = page_events::DomContentEventFired::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::ResponseReceived(embedded_event) => {
                let event = network_events::ResponseReceived::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::DataReceived(embedded_event) => {
                let event = network_events::DataReceived::new(embedded_event);
                return TaskDescribe::from(event).into();
                // warn!("ignore DataReceived inner event.");
            }
            EmbeddedEvent::LoadingFinished(embedded_event) => {
                let event = network_events::LoadingFinished::new(embedded_event);
                return TaskDescribe::from(event).into();
                // warn!("ignore LoadingFinished inner event.");
            }
            EmbeddedEvent::RequestWillBeSent(embedded_event) => {
                // warn!("ignore RequestWillBeSent inner event. {:?}", embedded_event);
                let event = network_events::RequestWillBeSent::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            _ => {
                warn!("discard inner event: {:?}", target_message_event);
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
        let s = format!("got **response**. {:?}", resp);
        if s.len() > 400 {
            info!("{:?} ......................", s.split_at(400).0);
        } else {
            info!("{:?}", s);
        }
        let call_id = resp.call_id;
        if let Some(idx) = self.tasks_waiting_for_response.iter().position(|tasks| {
            if let Some(task) = tasks.get(0) {
                match task {
                    TaskDescribe::TargetCallMethod(target_call) => target_call.get_call_id() == call_id,
                    TaskDescribe::BrowserCallMethod(browser_call) => browser_call.get_call_id() == call_id,
                    _ => false,
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
                let last_task = if let Some(tk) = tasks.pop() {
                    tk
                } else {
                    current_task
                };
                error!("return current or last task with unfullfilled result: {:?}", last_task);
                return Some(last_task);
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
            if let Err(mut err) = self.handle_next_task(&current_task, tasks) {
                if let Some(ChromePageError::NextTaskExecution { tasks, error }) =
                    err.downcast_mut::<ChromePageError>()
                {
                    error!("handle next task fail. {:?}, {:?}", tasks, error);
                    return (*tasks).pop();
                } else {
                    panic!("handle next task fail. {:?}", err);
                }
            }
            return Some(current_task); // always popup task.
        } else {
            trace!("no matching task for call_id: {:?}", call_id);
        }
        None
    }

    fn execute_next_and_return_remains(
        &mut self,
        tasks: Vec<TaskDescribe>,
    ) {
        let next_task = tasks.get(0).expect("execute_next_and_return_remains got empty tasks.");
        if let Ok(method_str) =  String::try_from(next_task) {
            self.tasks_waiting_for_response.push(tasks);
            self.send_message_direct(method_str);
        } else {
            error!("execute_next_and_return_remains to_methd_str failed. {:?}", next_task);
        }
     }

    fn handle_next_task(
        &mut self,
        current_task: &TaskDescribe,
        mut tasks: Vec<TaskDescribe>,
    ) -> Result<(), failure::Error> {
        let mut next_task = tasks.get_mut(0).expect("handle_next_task received empty tasks.");
        match (current_task, &mut next_task) {
            (
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetDocument(get_document)),
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::QuerySelector(query_selector)),
            ) => {
                query_selector.node_id = get_document
                    .task_result
                    .as_ref()
                    .and_then(|nd| Some(nd.node_id));
                self.execute_next_and_return_remains(tasks);
            }
            (
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::QuerySelector(query_selector)),
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::DescribeNode(describe_node)),
            ) => {
                describe_node.node_id = query_selector.task_result;
                self.execute_next_and_return_remains(tasks);
            }
            (
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::QuerySelector(query_selector)),
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetBoxModel(get_box_model)),
            ) => {
                get_box_model.node_id = query_selector.task_result;
                self.execute_next_and_return_remains(tasks);
            }
            (
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetBoxModel(get_box_model)), 
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::CaptureScreenshot(screen_shot)),
            ) => {
                if let Some(mb) = &get_box_model.task_result {
                    let viewport = mb.content_viewport();
                    screen_shot.clip = Some(viewport);
                    self.execute_next_and_return_remains(tasks);
                } else {
                    failure::bail!("found_box is None!");
                }
            }
            _ => {
                // warn!("unknown pair: {:?}, {:?}", current_task, next_task);
                self.execute_next_and_return_remains(tasks);
            }
        }
        Ok(())
    }

    fn full_fill_task(
        &self,
        resp: protocol::Response,
        mut task_describe: &mut TaskDescribe,
    ) -> Result<(), failure::Error> {
        match &mut task_describe {
            TaskDescribe::TargetCallMethod(target_call) => match target_call {
                TargetCallMethodTask::GetDocument(get_document) => {
                    let get_document_return_object =
                        protocol::parse_response::<dom::methods::GetDocumentReturnObject>(resp)?;
                    get_document.task_result = Some(get_document_return_object.root);
                }
                TargetCallMethodTask::PageEnable(_common_fields) => {}
                TargetCallMethodTask::PageReload(_page_reload) => {}
                TargetCallMethodTask::QuerySelector(query_selector) => {
                    let return_object =
                        protocol::parse_response::<dom::methods::QuerySelectorReturnObject>(resp)?;
                    query_selector.task_result= Some(return_object.node_id);
                }
                TargetCallMethodTask::DescribeNode(describe_node) => {
                    let describe_node_return_object =
                        protocol::parse_response::<dom::methods::DescribeNodeReturnObject>(resp)?;
                    describe_node.task_result= Some(describe_node_return_object.node);
                }
                TargetCallMethodTask::GetBoxModel(get_box_model) => {
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
                TargetCallMethodTask::CaptureScreenshot(screen_shot) => {
                    let capture_screenshot_return_object =
                        protocol::parse_response::<page::methods::CaptureScreenshotReturnObject>(resp)?;
                    screen_shot.task_result= Some(capture_screenshot_return_object.data);
                }
                TargetCallMethodTask::RuntimeEvaluate(runtime_evaluate) => {
                    let evaluate_return_object =
                        protocol::parse_response::<runtime::methods::EvaluateReturnObject>(resp)?;
                    runtime_evaluate.task_result = Some(evaluate_return_object);
                }
                TargetCallMethodTask::NavigateTo(navigate_to) => {
                    let navigate_to_return_object = protocol::parse_response::<page::methods::NavigateReturnObject>(resp)?;
                    navigate_to.task_result = Some(navigate_to_return_object);
                }
                TargetCallMethodTask::RuntimeEnable(common_fields) => {
                    trace!("runtime enabled: {:?}", common_fields);
                }
                TargetCallMethodTask::RuntimeGetProperties(get_properties) => {
                    let get_properties_return_object = protocol::parse_response::<runtime::methods::GetPropertiesReturnObject>(resp)?;
                    get_properties.task_result = Some(get_properties_return_object);
                }
                TargetCallMethodTask::GetResponseBodyForInterception(task) => {
                    let return_object = protocol::parse_response::<network::methods::GetResponseBodyForInterceptionReturnObject>(resp)?;
                    task.task_result = Some(return_object);
                }
                TargetCallMethodTask::RuntimeCallFunctionOn(task) => {
                    let task_return_object = protocol::parse_response::<runtime::methods::CallFunctionOnReturnObject>(resp)?;
                    task.task_result = Some(task_return_object);
                }
                TargetCallMethodTask::PrintToPDF(task) => {
                    let task_return_object = protocol::parse_response::<page::methods::PrintToPdfReturnObject>(resp)?;
                    task.task_result = Some(task_return_object.data);
                }
                TargetCallMethodTask::NetworkEnable(_task) => {
                    info!("network enabled.");
                }
                TargetCallMethodTask::SetRequestInterception(task) => {
                    info!("set_request_interception enabled. {:?}", task);
                }
                TargetCallMethodTask::ContinueInterceptedRequest(_task) => {
                    info!("continue_intercepted_request done.");
                }
            }
            TaskDescribe::BrowserCallMethod(browser_call) => match browser_call {
                BrowserCallMethodTask::CreateTarget(task) => {
                    info!("nothing to full fill: {:?}", task);
                }
                BrowserCallMethodTask::SetDiscoverTargets(task) => {
                    info!("nothing to full fill:: {:?}", task);
                }
                BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => {
                    info!("nothing to full fill:: {:?}", task);
                }
                BrowserCallMethodTask::SecurityEnable(task) => {
                    info!("nothing to full fill:: {:?}", task);
                }
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
        _maybe_session_id: Option<String>,
        _maybe_target_id: Option<String>,
    ) -> Option<TaskDescribe> {
        match protocol_event {
            protocol::Event::FrameNavigated(raw_event) => {
                let event = page_events::FrameNavigated::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::TargetInfoChanged(raw_event) => {
                let event = target_events::TargetInfoChanged::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::TargetCreated(raw_event) => {
                let event = target_events::TargetCreated::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::AttachedToTarget(raw_event) => {
                let event = target_events::AttachedToTarget::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::FrameAttached(raw_event) => {
                let event = page_events::FrameAttached::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::FrameStoppedLoading(raw_event) => {
                let event = page_events::FrameStoppedLoading::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::FrameStartedLoading(raw_event) => {
                let event = page_events::FrameStartedLoading::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::FrameDetached(raw_event) => {
                let event = page_events::FrameDetached::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::RequestIntercepted(raw_event) => {
                let event = network_events::RequestIntercepted::new(raw_event);
                return Some(event.into());
            }
            _ => {
                warn!("unprocessed inner event: {:?}", protocol_event);
            }
        }
        None
    }

fn process_message(&mut self, value: protocol::Message) -> Option<(Option<target::SessionID>, Option<target::TargetId>, TaskDescribe)> {
                match value {
                    protocol::Message::Response(resp) => {
                        if let Some(page_message) = self.handle_response(resp, None, None) {
                            (None, None, page_message).into()
                        } else{
                            None
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
                                    self.handle_response(resp, Some(session_id.clone()), Some(target_id.clone()))
                                {
                                    (Some(session_id), Some(target_id), page_message).into()
                                } else {
                                    None
                                }
                            }
                            Ok(protocol::Message::Event(protocol_event)) => {
                                if let Some(page_message) = self.handle_protocol_event(
                                    protocol_event,
                                    Some(session_id.clone()),
                                    Some(target_id.clone()),
                                ) {
                                    (Some(session_id), Some(target_id), page_message).into()
                                } else {
                                    None
                                }
                            }
                            _ => match parse_raw_message(&message_field) {
                                Ok(embedded_events::EmbeddedEventWrapper::EmbeddedEvent(target_message_event)) => {
                                    // trace!("got inner event: {:?}", target_message_event);
                                    if let Some(page_message) = self.handle_inner_target_events(
                                        target_message_event,
                                        Some(session_id.clone()),
                                        Some(target_id.clone()),
                                    ) {
                                        (Some(session_id), Some(target_id), page_message).into()
                                    } else{
                                        None
                                    }
                                }
                                Err(_error) => {
                                    error!(
                                        "parse_raw_message failed ** this is message_field aka inner event: {:?}",
                                        message_field
                                    );
                                    None
                                }
                            },
                        }
                    }
                    protocol::Message::Event(protocol_event) => {
                        if let Some(page_message) =
                            self.handle_protocol_event(protocol_event, None, None)
                        {
                            (None, None, page_message).into()
                        } else {
                            None
                        }
                    }
                    protocol::Message::Connected => {
                        Some((None, None, TaskDescribe::ChromeConnected))
                    }
                    other => {
                        warn!("got unknown message1: {:?}", other);
                        None
                    }
                }
}
}

pub fn parse_raw_message(raw_message: &str) -> Result<embedded_events::EmbeddedEventWrapper, Error> {
    Ok(serde_json::from_str::<embedded_events::EmbeddedEventWrapper>(
        raw_message,
    )?)
}


// The main loop should stop at some point, by invoking the methods on the page to drive the loop to run.
impl Stream for ChromeDebugSession {
    type Item = (Option<target::SessionID>, Option<target::TargetId>, TaskDescribe);
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.chrome_browser.poll()) {

                if let Some(task_describe) = self.process_message(value) {
                    break Ok(Some(task_describe).into());
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
