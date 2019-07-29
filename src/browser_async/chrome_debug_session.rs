use super::task_describe::{
    dom_events, network_events, page_events, runtime_events, target_events, BrowserCallMethodTask,
    TargetCallMethodTask, TaskDescribe,
};

use super::embedded_events::{self, EmbeddedEvent};
use crate::browser_async::{chrome_browser::ChromeBrowser, TaskId};

use super::task_manager;
use crate::browser::tab::element::{BoxModel, ElementQuad};
use crate::protocol::{self, dom, emulation, network, page, runtime, target};
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
    task_manager: task_manager::TaskManager,
}

impl ChromeDebugSession {
    pub fn new(chrome_browser: ChromeBrowser) -> Self {
        Self {
            chrome_browser,
            session_id: None,
            unique_number: AtomicUsize::new(10000),
            task_manager: task_manager::TaskManager::new(),
        }
    }

    pub fn tasks_waiting_for_response_count(&self) -> usize {
        self.task_manager.tasks_count()
    }

    /// execute_task causes sending a message to the chrome. This drive the program to run.
    /// invoking multiple times cause sending message in parallel.
    pub fn execute_task(&mut self, task_vec: Vec<TaskDescribe>) {
        self.execute_next_and_return_remains(task_manager::TaskGroup::new(task_vec));
    }

    pub fn check_stalled_tasks(&mut self) {
        if let Some(tg) = self.task_manager.get_stalled_task_group(45) {
            warn!("rerun stalled task group: {}", tg);
            self.execute_next_and_return_remains(tg);
        }
    }

    fn execute_next_and_return_remains(&mut self, mut task_group: task_manager::TaskGroup) {
        task_group.renew_first_task_call_id();
        let next_task = task_group.get_first_task_ref();
        if let Ok(method_str) = String::try_from(next_task) {
            self.task_manager.push_task_group(task_group);
            self.send_message_direct(method_str);
        } else {
            error!(
                "execute_next_and_return_remains to_method_str failed. {:?}",
                next_task
            );
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
        _session_id: Option<String>,
        _target_id: Option<target::TargetId>,
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
            }
            EmbeddedEvent::LoadingFinished(embedded_event) => {
                let event = network_events::LoadingFinished::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::RequestWillBeSent(embedded_event) => {
                let event = network_events::RequestWillBeSent::new(embedded_event);
                return TaskDescribe::from(event).into();
            }
            EmbeddedEvent::ChildNodeCountUpdated(embedded_event) => {
                let event = dom_events::ChildNodeCountUpdated::new(embedded_event);
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
        if let Some(idx) = self.task_manager.find_task_vec_by_call_id(call_id) {
            let mut task_group = self.task_manager.take_task_group(idx);
            let mut current_task = task_group.get_first_task();

            // if has remote error.
            if resp.error.is_some() {
                error!("got remote error: {:?}", resp);
                let last_task = task_group.get_last_task_or_current(current_task);
                error!(
                    "return current or last task with unfulfilled result: {:?}",
                    last_task
                );
                return Some(last_task);
            }

            if let Err(err) = self.full_fill_current_task(resp, &mut current_task) {
                error!("fulfill task failed. {:?}", err);
                // return last task (not fulfilled) in vec.
                return Some(task_group.get_last_task_or_current(current_task));
            }
            if task_group.is_empty() {
                return Some(current_task);
            }

            let cloned_task: TaskDescribe = match &current_task {
                TaskDescribe::TargetCallMethod(target_task) => target_task.clone().into(),
                TaskDescribe::BrowserCallMethod(browser_task) => browser_task.clone().into(),
                _ => {
                    error!("got impossible task type: {:?}", &current_task);
                    panic!("got impossible response.");
                }
            };

            task_group.push_completed_task(current_task);
            task_group.full_fill_next_task();
            self.execute_next_and_return_remains(task_group);
            return Some(cloned_task);
        } else {
            info!("no matching task for call_id: {:?}", resp);
        }
        None
    }

    fn full_fill_current_task(
        &self,
        resp: protocol::Response,
        mut task_describe: &mut TaskDescribe,
    ) -> Result<(), failure::Error> {
        match &mut task_describe {
            TaskDescribe::TargetCallMethod(target_call) => match target_call {
                TargetCallMethodTask::GetDocument(task) => {
                    let return_object =
                        protocol::parse_response::<dom::methods::GetDocumentReturnObject>(resp)?;
                    task.task_result.replace(return_object.root);
                }
                TargetCallMethodTask::PageEnable(_common_fields) => {}
                TargetCallMethodTask::PageReload(_page_reload) => {}
                TargetCallMethodTask::QuerySelector(task) => {
                    let return_object =
                        protocol::parse_response::<dom::methods::QuerySelectorReturnObject>(resp)?;
                    task.task_result.replace(return_object.node_id);
                }
                TargetCallMethodTask::DescribeNode(task) => {
                    let return_object =
                        protocol::parse_response::<dom::methods::DescribeNodeReturnObject>(resp)?;
                    task.task_result.replace(return_object.node);
                }
                TargetCallMethodTask::GetBoxModel(task) => {
                    let return_object =
                        protocol::parse_response::<dom::methods::GetBoxModelReturnObject>(resp)?;
                    let raw_model = return_object.model;
                    let model_box = BoxModel {
                        content: ElementQuad::from_raw_points(&raw_model.content),
                        padding: ElementQuad::from_raw_points(&raw_model.padding),
                        border: ElementQuad::from_raw_points(&raw_model.border),
                        margin: ElementQuad::from_raw_points(&raw_model.margin),
                        width: raw_model.width,
                        height: raw_model.height,
                    };
                    task.task_result.replace(model_box);
                }
                TargetCallMethodTask::GetContentQuads(task) => {
                    let return_object = protocol::parse_response::<
                        dom::methods::GetContentQuadsReturnObject,
                    >(resp)?;
                    task.task_result.replace(return_object.quads);
                }
                TargetCallMethodTask::CaptureScreenshot(task) => {
                    let capture_screenshot_return_object = protocol::parse_response::<
                        page::methods::CaptureScreenshotReturnObject,
                    >(resp)?;
                    task.task_result
                        .replace(capture_screenshot_return_object.data);
                }
                TargetCallMethodTask::Evaluate(task) => {
                    let evaluate_return_object =
                        protocol::parse_response::<runtime::methods::EvaluateReturnObject>(resp)?;
                    task.task_result.replace(evaluate_return_object);
                }
                TargetCallMethodTask::NavigateTo(task) => {
                    let return_object =
                        protocol::parse_response::<page::methods::NavigateReturnObject>(resp)?;
                    task.task_result.replace(return_object);
                }
                TargetCallMethodTask::RuntimeEnable(task) => {
                    trace!("runtime enabled: {:?}", task);
                }
                TargetCallMethodTask::GetProperties(task) => {
                    let return_object = protocol::parse_response::<
                        runtime::methods::GetPropertiesReturnObject,
                    >(resp)?;
                    task.task_result.replace(return_object);
                }
                TargetCallMethodTask::GetResponseBodyForInterception(task) => {
                    let return_object = protocol::parse_response::<
                        network::methods::GetResponseBodyForInterceptionReturnObject,
                    >(resp)?;
                    task.task_result.replace(return_object);
                }
                TargetCallMethodTask::RuntimeCallFunctionOn(task) => {
                    let task_return_object = protocol::parse_response::<
                        runtime::methods::CallFunctionOnReturnObject,
                    >(resp)?;
                    task.task_result = Some(task_return_object);
                }
                TargetCallMethodTask::PrintToPDF(task) => {
                    let task_return_object =
                        protocol::parse_response::<page::methods::PrintToPdfReturnObject>(resp)?;
                    task.task_result.replace(task_return_object.data);
                }
                TargetCallMethodTask::NetworkEnable(_task) => {
                    info!("network enabled.");
                }
                TargetCallMethodTask::PageClose(_task) => {
                    info!("page closed.");
                }
                TargetCallMethodTask::LogEnable(_task) => {
                    info!("log enabled.");
                }
                TargetCallMethodTask::SetLifecycleEventsEnabled(_task) => {
                    info!("set lifecycle event enabled.");
                }
                TargetCallMethodTask::SetRequestInterception(task) => {
                    info!("set_request_interception enabled. {:?}", task);
                }
                TargetCallMethodTask::ContinueInterceptedRequest(_task) => {
                    info!("continue_intercepted_request done.");
                }
                TargetCallMethodTask::GetLayoutMetrics(task) => {
                    let task_return_object = protocol::parse_response::<
                        page::methods::GetLayoutMetricsReturnObject,
                    >(resp)?;
                    task.task_result.replace(task_return_object);
                }
                TargetCallMethodTask::BringToFront(_task) => {
                    info!("bring_to_front done.");
                }
                TargetCallMethodTask::DispatchMouseEvent(_task) => {
                    info!("dispatch_mouse_event done.");
                }
                TargetCallMethodTask::CanEmulate(task) => {
                    let task_return_object = protocol::parse_response::<
                        emulation::methods::CanEmulateReturnObject,
                    >(resp)?;
                    task.task_result.replace(task_return_object.result);
                }
                TargetCallMethodTask::SetDeviceMetricsOverride(task) => {
                    task.task_result.replace(true);
                }
            },
            TaskDescribe::BrowserCallMethod(browser_call) => match browser_call {
                BrowserCallMethodTask::CreateTarget(task) => {
                    info!("nothing to full fill CreateTarget:: {:?}", task);
                }
                BrowserCallMethodTask::SetDiscoverTargets(task) => {
                    info!("nothing to full fill SetDiscoverTargets:: {:?}", task);
                }
                BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => {
                    info!(
                        "nothing to full fill SetIgnoreCertificateErrors:: {:?}",
                        task
                    );
                }
                BrowserCallMethodTask::SecurityEnable(task) => {
                    info!("nothing to full fill SecurityEnable:: {:?}", task);
                }
                BrowserCallMethodTask::AttachedToTarget(task) => {
                    info!("nothing to full fill AttachedToTarget:: {:?}", task);
                }
                BrowserCallMethodTask::ActivateTarget(task) => {
                    info!("nothing to full fill ActivateTarget:: {:?}", task);
                }
                BrowserCallMethodTask::CloseTarget(task) => {
                    let task_return_object =
                        protocol::parse_response::<target::methods::CloseTargetReturnObject>(resp)?;
                    task.task_result = Some(task_return_object.success);
                    // if let Some(r) = task.task_result {
                    //     if r {
                    //         info!("tab close method call returned. close successfully.");
                    //     } else {
                    //         error!("tab close method call returned. close failed.");
                    //     }
                    // } else {
                    //     error!("tab close method call returned. close failed. {:?}", task);
                    // }
                    // debug_session.tab_closed(maybe_target_id.as_ref(), task.task_result);
                }
            },
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
            protocol::Event::TargetDestroyed(raw_event) => {
                let event = target_events::TargetDestroyed::new(raw_event);
                return Some(event.into());
            }
            protocol::Event::Lifecycle(raw_event) => {
                let event = page_events::LifeCycle::new(raw_event);
                return Some(event.into());
            }
            _ => {
                warn!("unprocessed inner event: {:?}", protocol_event);
            }
        }
        None
    }

    fn process_message(
        &mut self,
        value: protocol::Message,
    ) -> Option<(
        Option<target::SessionID>,
        Option<target::TargetId>,
        TaskDescribe,
    )> {
        match value {
            protocol::Message::Response(resp) => {
                if let Some(page_message) = self.handle_response(resp, None, None) {
                    (None, None, page_message).into()
                } else {
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
                        if let Some(page_message) = self.handle_response(
                            resp,
                            Some(session_id.clone()),
                            Some(target_id.clone()),
                        ) {
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
                        Ok(embedded_events::EmbeddedEventWrapper::EmbeddedEvent(
                            target_message_event,
                        )) => {
                            // trace!("got inner event: {:?}", target_message_event);
                            if let Some(page_message) = self.handle_inner_target_events(
                                target_message_event,
                                Some(session_id.clone()),
                                Some(target_id.clone()),
                            ) {
                                (Some(session_id), Some(target_id), page_message).into()
                            } else {
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
                if let Some(page_message) = self.handle_protocol_event(protocol_event, None, None) {
                    (None, None, page_message).into()
                } else {
                    None
                }
            }
            protocol::Message::Connected => Some((None, None, TaskDescribe::ChromeConnected)),
            other => {
                warn!("got unknown message1: {:?}", other);
                None
            }
        }
    }
}

pub fn parse_raw_message(
    raw_message: &str,
) -> Result<embedded_events::EmbeddedEventWrapper, Error> {
    Ok(serde_json::from_str::<embedded_events::EmbeddedEventWrapper>(raw_message)?)
}

// The main loop should stop at some point, by invoking the methods on the page to drive the loop to run.
impl Stream for ChromeDebugSession {
    type Item = (
        Option<target::SessionID>,
        Option<target::TargetId>,
        TaskDescribe,
    );
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.chrome_browser.poll()) {
                if let Some(task_describe) = self.process_message(value) {
                    break Ok(Some(task_describe).into());
                } else {
                    info!("discard intermediate tasks.");
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}
