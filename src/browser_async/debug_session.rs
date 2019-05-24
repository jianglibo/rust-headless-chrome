use super::chrome_browser::ChromeBrowser;
use super::chrome_debug_session::ChromeDebugSession;
use super::interval_page_message::IntervalPageMessage;
use super::page_message::{PageResponse, PageResponseWrapper, response_object};
use super::tab::Tab;
use super::task_describe::{
    self as tasks, DomEvent, HasTaskId, PageEvent, RuntimeEnableTask, SetIgnoreCertificateErrorsTask,
    RuntimeEvent, SetDiscoverTargetsTask, TargetCallMethodTask, TargetEvent, TaskDescribe, SecurityEnableTask, BrowserCallMethodTask,
};

use crate::browser_async::{ChromePageError, TaskId};
use crate::protocol::target;
use failure;
use futures::{Async, Poll};
use log::*;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use websocket::futures::Stream;

struct Wrapper {
    pub chrome_debug_session: Arc<Mutex<ChromeDebugSession>>,
}

impl Stream for Wrapper {
    type Item = (
        Option<target::SessionID>,
        Option<target::TargetId>,
        TaskDescribe,
    );
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.chrome_debug_session.lock().unwrap().poll()
    }
}

fn handle_event_return(
    maybe_target_id: Option<target::TargetId>,
    page_response: PageResponse,
) -> Result<PageResponseWrapper, failure::Error> {
    Ok(PageResponseWrapper {
        target_id: maybe_target_id,
        task_id: None,
        page_response: page_response,
    })
}

/// An adapter for merging the output of two streams.
///
/// The merged stream produces items from either of the underlying streams as
/// they become available, and the streams are polled in a round-robin fashion.
/// Errors, however, are not merged: you get at most one error at a time.
// #[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct DebugSession {
    interval_page_message: IntervalPageMessage,
    pub chrome_debug_session: Arc<Mutex<ChromeDebugSession>>,
    seconds_from_start: usize,
    flag: bool,
    tabs: Vec<Tab>, // early created at front.
    wrapper: Wrapper,
}

impl Default for DebugSession {
    fn default() -> Self {
        let browser = ChromeBrowser::new();
        let chrome_debug_session = ChromeDebugSession::new(browser);
        Self::new(chrome_debug_session)
    }
}

impl DebugSession {
    pub fn new(chrome_debug_session: ChromeDebugSession) -> Self {
        let interval_page_message = IntervalPageMessage::new();
        let arc_cds = Arc::new(Mutex::new(chrome_debug_session));
        Self {
            interval_page_message,
            chrome_debug_session: arc_cds.clone(),
            seconds_from_start: 0,
            flag: false,
            tabs: Vec::new(),
            wrapper: Wrapper {
                chrome_debug_session: arc_cds,
            },
        }
    }

    pub fn get_tab_by_resp_mut(
        &mut self,
        page_response_wrapper: &PageResponseWrapper,
    ) -> Result<&mut Tab, failure::Error> {
        self.get_tab_by_id_mut(page_response_wrapper.target_id.as_ref())
    }
    pub fn get_tab_by_id_mut(
        &mut self,
        target_id: Option<&target::TargetId>,
    ) -> Result<&mut Tab, failure::Error> {
        if let Some(tab) = self
            .tabs
            .iter_mut()
            .find(|t| Some(&t.target_info.target_id) == target_id)
        {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn create_new_tab(&mut self, url: &str) {
        let task = tasks::CreateTargetTaskBuilder::default()
            .url(url.to_owned())
            .build()
            .expect("CreateTargetTaskBuilder should success.");
        let method_str: String = (&tasks::TaskDescribe::from(task))
            .try_into()
            .expect("should convert from CreateTargetTask");
        self.chrome_debug_session
            .lock()
            .expect("create_new_tab should obtain chrome_debug_session.")
            .send_message_direct(method_str);
    }

    pub fn get_browser_context_ids(&self) -> Vec<&target::BrowserContextID> {
        let mut ids: Vec<&target::BrowserContextID> = self
            .tabs
            .iter()
            .filter_map(|tab| tab.target_info.browser_context_id.as_ref())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    pub fn get_tab_by_id(
        &self,
        target_id: Option<&target::TargetId>,
    ) -> Result<&Tab, failure::Error> {
        if let Some(tab) = self
            .tabs
            .iter()
            .find(|t| Some(&t.target_info.target_id) == target_id)
        {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn first_page_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(0)
    }
    pub fn main_tab(&self) -> Option<&Tab> {
        self.tabs.get(0)
    }

    fn send_fail(
        &mut self,
        target_id: Option<target::TargetId>,
        task_id: Option<TaskId>,
    ) -> Poll<Option<PageResponseWrapper>, failure::Error> {
        let pr = PageResponseWrapper {
            target_id,
            task_id,
            page_response: PageResponse::Fail,
        };
        Ok(Some(pr).into())
    }

    pub fn runtime_enable(&mut self) {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .unwrap();
        let task = RuntimeEnableTask { common_fields };
        self.chrome_debug_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);
    }

    pub fn set_ignore_certificate_errors(&mut self, ignore: bool) {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .unwrap();
        let task = SetIgnoreCertificateErrorsTask { common_fields, ignore };
        self.chrome_debug_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);
    }

    pub fn set_discover_targets(&mut self, enable: bool) {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .unwrap();
        let task = SetDiscoverTargetsTask {
            common_fields,
            discover: enable,
        };
        self.chrome_debug_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);
    }

    pub fn security_enable(&mut self) {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .unwrap();
        let task = SecurityEnableTask {
            common_fields,
        };
        self.chrome_debug_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);        
    }

    fn handle_dom_event(
        &mut self,
        dom_event: DomEvent,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match dom_event {
            DomEvent::AttributeModified(event) => {}
            DomEvent::AttributeRemoved(event) => {}
            DomEvent::CharacterDataModified(event) => {}
            DomEvent::ChildNodeCountUpdated(event) => {}
            DomEvent::ChildNodeInserted(event) => {}
            DomEvent::ChildNodeRemoved(event) => {}
            DomEvent::DocumentUpdated(event) => {}
            DomEvent::SetChildNodes(event) => {
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let (parent_id, nodes) = event.into_parent_children();
                tab.node_arrived(parent_id, nodes);
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: None,
                    page_response: PageResponse::SetChildNodesOccured(parent_id),
                });
            }
        }
        warn!("unhandled branch handle_dom_event");
        Ok(PageResponseWrapper::default())
    }

    fn handle_target_event(
        &mut self,
        target_event: TargetEvent,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match target_event {
            TargetEvent::ReceivedMessageFromTarget(event) => {}
            TargetEvent::TargetCreated(event) => {
                if let target::TargetType::Page = event.get_target_type() {
                    // info!("receive page created event: {:?}", event);
                    let target_info = event.to_target_info();
                    let target_id = target_info.target_id.clone();
                    let tab = Tab::new(target_info, Arc::clone(&self.chrome_debug_session));
                    self.tabs.push(tab);
                    let idx = self.tabs.len();
                    return Ok(PageResponseWrapper {
                        target_id: Some(target_id),
                        task_id: None,
                        page_response: PageResponse::PageCreated(idx),
                    });
                } else {
                    info!("got other target_event: {:?}", event);
                }
            }
            TargetEvent::TargetCrashed(event) => {}
            TargetEvent::AttachedToTarget(event) => {
                if event.is_page_attached() {
                    let target_id = event.get_target_id();
                    let tab = self
                        .get_tab_by_id_mut(Some(&target_id))
                        .expect("when the page attached, tab should have been exists.");
                    tab.session_id.replace(event.get_session_id());
                    return Ok(event
                        .try_into_page_attached()
                        .expect("should be a page attached."));
                } else {
                    info!("got AttachedToTarget event it's target_type was other than page.");
                }
            }
            TargetEvent::TargetInfoChanged(event) => {
                let target_info = event.into_target_info();
            if let Ok(tab) = self.get_tab_by_id_mut(Some(&target_info.target_id)) {
                    tab.target_info = target_info;
                    trace!(
                        "target info changed: {:?}",
                        tab.target_info
                    );
                } else {
                    warn!(
                        "target changed, no correspond tab. {:?}",
                        target_info
                    );
                }
            }
        }
        warn!("unhandled branch handle_target_event");
        Ok(PageResponseWrapper::default())
    }

    fn handle_runtime_event(
        &mut self,
        runtime_event: RuntimeEvent,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match runtime_event {
            RuntimeEvent::ConsoleAPICalled(event) => {
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let console_call_parameters = event.into_raw_parameters();
                tab.verify_execution_context_id(&console_call_parameters);
            }
            RuntimeEvent::ExceptionRevoked(event) => {}
            RuntimeEvent::ExceptionThrown(event) => {}
            RuntimeEvent::ExecutionContextCreated(event) => {
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let frame_id = tab
                    .runtime_execution_context_created(event.into_exection_context_description());
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::RuntimeExecutionContextCreated(frame_id),
                );
            }
            RuntimeEvent::ExecutionContextDestroyed(event) => {
                let execution_context_id = event.into_exection_context_id();
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.runtime_execution_context_destroyed(execution_context_id);
            }
            RuntimeEvent::ExecutionContextsCleared(event) => {}
            RuntimeEvent::InspectRequested(event) => {}
        }
        warn!("unhandled branch handle_runtime_event");
        Ok(PageResponseWrapper::default())
    }

    fn handle_page_event(
        &mut self,
        page_event: PageEvent,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match page_event {
            PageEvent::DomContentEventFired(event) => {}
            // attached may not invoke, if invoked it's the first. then started, navigated, stopped.
            PageEvent::FrameAttached(event) => {
                let raw_parameters = event.into_raw_parameters();
                let frame_id = raw_parameters.frame_id.clone();
                info!(
                    "-----------------frame_attached-----------------{:?}", frame_id
                );
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab._frame_attached(raw_parameters);
                return handle_event_return(maybe_target_id, PageResponse::FrameAttached(frame_id));
            }
            PageEvent::FrameDetached(event) => {
                let frame_id = event.into_frame_id();
                info!(
                    "-----------------frame_detached-----------------{:?}",
                    frame_id.clone()
                );
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab._frame_detached(&frame_id);
            }
            PageEvent::FrameStartedLoading(event) => {
                let frame_id = event.into_frame_id();
            // started loading is first, then attached.
                info!(
                    "-----------------frame_started_loading-----------------{:?}",
                    frame_id
                );
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab._frame_started_loading(frame_id.clone());
                return handle_event_return(maybe_target_id, PageResponse::FrameStartedLoading(frame_id));
            }
            PageEvent::FrameNavigated(event) => {
                info!(
                    "-----------------frame_navigated-----------------{:?}",
                    event
                );
                let frame = event.into_frame();
                let frame_id = frame.id.clone();
                self.get_tab_by_id_mut(maybe_target_id.as_ref())
                    .expect("FrameNavigated event should have target_id.")
                    ._frame_navigated(frame);
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::FrameNavigated(frame_id),
                );
            }
            PageEvent::FrameStoppedLoading(event) => {
                // TaskDescribe::FrameStoppedLoading(frame_id, common_fields) => {
                info!(
                    "-----------------frame_stopped_loading-----------------{:?}",
                    event
                );
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let frame_id = event.into_frame_id();
                tab._frame_stopped_loading(frame_id.clone());
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::FrameStoppedLoading(frame_id),
                );
            }
            PageEvent::LoadEventFired(event) => {
                return handle_event_return(maybe_target_id, event.into_page_response());
            }
        }
        warn!("unhandled branch handle_page_event");
        Ok(PageResponseWrapper::default())
    }
    fn handle_target_method_call(
        &mut self,
        target_call_method_task: TargetCallMethodTask,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match target_call_method_task {
            TargetCallMethodTask::GetDocument(task) => {
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let v = Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::GetDocumentDone,
                });
                tab.root_node = task.task_result;
                return v;
            }
            TargetCallMethodTask::NavigateTo(task) => {
                trace!("navigate_to task returned: {:?}", task);
            }
            TargetCallMethodTask::QuerySelector(task) => {
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: task.into_page_response(),
                });
            }
            TargetCallMethodTask::DescribeNode(task) => {
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let node_id = task.task_result.as_ref().and_then(|n| Some(n.node_id));

                let v = Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::DescribeNodeDone(task.selector, node_id),
                });

                tab.node_returned(task.task_result);
                return v;
            }
            TargetCallMethodTask::PrintToPDF(task) => {
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::PrintToPdfDone(task.task_result),
                });
            }
            TargetCallMethodTask::GetBoxModel(task) => {
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::GetBoxModelDone(task.selector, task.task_result.map(Box::new)),
                });
            }
            TargetCallMethodTask::PageEnable(task) => {
                info!("page_enabled: {:?}", task);
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::PageEnabled,
                });
            }
            TargetCallMethodTask::RuntimeEnable(task) => {
                return Ok(PageResponseWrapper{
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::RuntimeEnabled,
                });
            }
            TargetCallMethodTask::CaptureScreenshot(task) => {
                let task_id = task.get_task_id();
                let ro = response_object::CaptureScreenshot {
                    selector: task.selector,
                    base64: task.task_result,
                };
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task_id),
                    page_response: PageResponse::CaptureScreenshotDone(ro),
                });
            }
            TargetCallMethodTask::RuntimeEvaluate(task) => {
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::EvaluateDone(task.task_result),
                });
            }
            TargetCallMethodTask::RuntimeGetProperties(task) => {
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::GetPropertiesDone(task.task_result),
                });
            }
            TargetCallMethodTask::RuntimeCallFunctionOn(task) => {
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::CallFunctionOnDone(task.task_result),
                });
            }
            _ => {
                info!("ignored method return. {:?}", target_call_method_task);
            }
        }
        warn!("unhandled branch handle_target_method_call");
        Ok(PageResponseWrapper::default())
    }



    fn handle_browser_method_call(
        &mut self,
        browser_call_method_task: BrowserCallMethodTask,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match browser_call_method_task {
            BrowserCallMethodTask::SetDiscoverTargets(task) => {
                trace!("TargetSetDiscoverTargets returned. {:?}", task);
            }
            BrowserCallMethodTask::CreateTarget(task) => {
                trace!("TargetSetDiscoverTargets returned. {:?}", task);
            }
        }
        Ok(PageResponseWrapper::default())
    }

    pub fn send_page_message(
        &mut self,
        item_tuple: (
            Option<target::SessionID>,
            Option<target::TargetId>,
            TaskDescribe,
        ),
    ) -> Poll<Option<PageResponseWrapper>, failure::Error> {
        let (session_id, target_id, item) = item_tuple;
        match item {
            TaskDescribe::Interval => {
                self.seconds_from_start += 1;
                Ok(Some(PageResponseWrapper::new(PageResponse::SecondsElapsed(
                    self.seconds_from_start,
                )))
                .into())
            }
            TaskDescribe::TargetCallMethod(task) => Ok(self
                .handle_target_method_call(task, session_id, target_id)
                .ok()
                .into()),
            TaskDescribe::PageEvent(page_event) => Ok(self
                .handle_page_event(page_event, session_id, target_id)
                .ok()
                .into()),
            TaskDescribe::RuntimeEvent(runtime_event) => Ok(self
                .handle_runtime_event(runtime_event, session_id, target_id)
                .ok()
                .into()),
            TaskDescribe::TargetEvent(target_event) => Ok(self
                .handle_target_event(target_event, session_id, target_id)
                .ok()
                .into()),
            TaskDescribe::DomEvent(dom_event) => Ok(self
                .handle_dom_event(dom_event, session_id, target_id)
                .ok()
                .into()),
            TaskDescribe::ChromeConnected => {
                let resp = Some(PageResponseWrapper::new(PageResponse::ChromeConnected));
                Ok(resp.into())
            }
            TaskDescribe::BrowserCallMethod(task) => 
                Ok(self
                .handle_browser_method_call(task, session_id, target_id)
                .ok()
                .into()),
            _ => {
                warn!("debug_session got unknown task. {:?}", item);
                self.send_fail(None, None)
            }
        }
    }
}

impl Stream for DebugSession {
    type Item = PageResponseWrapper;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let (a, b) = if self.flag {
            (
                &mut self.wrapper as &mut Stream<Item = _, Error = _>,
                &mut self.interval_page_message as &mut Stream<Item = _, Error = _>,
            )
        } else {
            (
                &mut self.interval_page_message as &mut Stream<Item = _, Error = _>,
                &mut self.wrapper as &mut Stream<Item = _, Error = _>,
            )
        };
        self.flag = !self.flag;
        let a_done = match a.poll()? {
            Async::Ready(Some(item)) => return self.send_page_message(item),
            Async::Ready(None) => true,
            Async::NotReady => false,
        };

        match b.poll()? {
            Async::Ready(Some(item)) => {
                // If the other stream isn't finished yet, give them a chance to
                // go first next time as we pulled something off `b`.
                if !a_done {
                    self.flag = !self.flag;
                }
                self.send_page_message(item)
            }
            Async::Ready(None) if a_done => Ok(None.into()),
            Async::Ready(None) | Async::NotReady => Ok(Async::NotReady),
        }
    }
}
