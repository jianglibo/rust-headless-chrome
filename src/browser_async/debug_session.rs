use super::chrome_browser::ChromeBrowser;
use super::chrome_debug_session::ChromeDebugSession;
use super::interval_page_message::IntervalPageMessage;
use super::page_message::{response_object, PageResponse, PageResponseWrapper};
use super::tab::Tab;
use super::task_describe::{
    self as tasks, BrowserCallMethodTask, CommonDescribeFields, DomEvent, PageEvent,
    RuntimeEnableTask, RuntimeEvent, SetDiscoverTargetsTask, TargetCallMethodTask, TargetEvent,
    TaskDescribe,
};

use crate::browser_async::{ChromePageError, TaskId};
use crate::protocol::target;
use failure;
use futures::{Async, Poll};
use log::*;
use std::convert::TryInto;
use std::sync::{Arc, Mutex};
use websocket::futures::Stream;

// const DEFAULT_TAB_NAME: &str = "_default_tab_";

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

    fn send_fail_1(
        &mut self,
        common_fields: Option<&CommonDescribeFields>,
    ) -> Poll<Option<PageResponseWrapper>, failure::Error> {
        if let Some(cf) = common_fields {
            let pr = PageResponseWrapper {
                target_id: cf.target_id.clone(),
                task_id: Some(cf.task_id),
                page_response: PageResponse::Fail
            };
            Ok(Some(pr).into())
        } else {
            Ok(Some(PageResponseWrapper::default()).into())
        }
    }

    fn handle_dom_event(&self, dom_event: DomEvent) -> PageResponseWrapper {
        match dom_event {
            DomEvent::AttributeModified(event) => {}
            DomEvent::AttributeRemoved(event) => {}
            DomEvent::CharacterDataModified(event) => {}
            DomEvent::ChildNodeCountUpdated(event) => {}
            DomEvent::ChildNodeInserted(event) => {}
            DomEvent::ChildNodeRemoved(event) => {}
            DomEvent::DocumentUpdated(event) => {}
            DomEvent::SetChildNodes(event) => {}
        }
        PageResponseWrapper::default()
    }

    fn handle_target_event(
        &mut self,
        target_event: TargetEvent,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> PageResponseWrapper {
        match target_event {
            TargetEvent::ReceivedMessageFromTarget(event) => {}
            TargetEvent::TargetCreated(event) => {
                if let target::TargetType::Page = event.get_target_type() {
                    info!("receive page created event: {:?}", event);
                    let target_info = event.to_target_info();
                    let target_id = target_info.target_id.clone();
                    let tab = Tab::new(target_info, Arc::clone(&self.chrome_debug_session));
                    self.tabs.push(tab);
                    let idx = self.tabs.len();
                    return PageResponseWrapper{
                        target_id: Some(target_id),
                        task_id: None,
                        page_response: PageResponse::PageCreated(idx),
                    };
                } else {
                    info!("got other target_event: {:?}", event);
                }
            }
            TargetEvent::TargetCrashed(event) => {}
            TargetEvent::AttachedToTarget(event) => {
                if event.is_page_attached() {
                    let target_id = event.get_target_id();
                    let tab = self.get_tab_by_id_mut(Some(&target_id)).expect("when the page attached, tab should have been exists.");
                    tab.session_id.replace(event.get_session_id());
                    return event.try_into_page_attached().expect("should be a page attached.");
                } else {
                    info!("got AttachedToTarget event it's target_type was other than page.");
                }
            }
            TargetEvent::TargetInfoChanged(event) => {}
        }
        PageResponseWrapper::default()
    }

    fn handle_runtime_event(&self, runtime_event: RuntimeEvent) -> PageResponseWrapper {
        match runtime_event {
            RuntimeEvent::ConsoleAPICalled(event) => {}
            RuntimeEvent::ExceptionRevoked(event) => {}
            RuntimeEvent::ExceptionThrown(event) => {}
            RuntimeEvent::ExecutionContextCreated(event) => {}
            RuntimeEvent::ExecutionContextDestroyed(event) => {}
            RuntimeEvent::ExecutionContextsCleared(event) => {}
            RuntimeEvent::InspectRequested(event) => {}
        }
        PageResponseWrapper::default()
    }

    fn handle_page_event(&self, page_event: PageEvent) -> PageResponseWrapper {
        match page_event {
            // PageEvent::PageCreated(event) => {}
            PageEvent::DomContentEventFired(event) => {}
            PageEvent::FrameAttached(event) => {}
            PageEvent::FrameDetached(event) => {}
            PageEvent::FrameStartedLoading(event) => {}
            PageEvent::FrameNavigated(event) => {}
            PageEvent::FrameStoppedLoading(event) => {}
            PageEvent::LoadEventFired(event) => {}
        }
        PageResponseWrapper::default()
    }
    fn handle_target_method_call(
        &self,
        target_call_method_task: TargetCallMethodTask,
    ) -> PageResponseWrapper {
        match target_call_method_task {
            TargetCallMethodTask::GetDocument(task) => {}
            TargetCallMethodTask::NavigateTo(task) => {}
            TargetCallMethodTask::QuerySelector(task) => {}
            TargetCallMethodTask::DescribeNode(task) => {}
            TargetCallMethodTask::PrintToPDF(task) => {}
            TargetCallMethodTask::GetBoxModel(task) => {}
            TargetCallMethodTask::PageEnable(task) => {}
            TargetCallMethodTask::RuntimeEnable(task) => {}
            TargetCallMethodTask::CaptureScreenshot(task) => {}
            TargetCallMethodTask::TargetSetDiscoverTargets(task) => {}
            TargetCallMethodTask::RuntimeEvaluate(task) => {}
            TargetCallMethodTask::RuntimeGetProperties(task) => {}
            TargetCallMethodTask::RuntimeCallFunctionOn(task) => {}
        }
        PageResponseWrapper::default()
    }

    fn convert_to_page_response(
        &self,
        common_fields: Option<&CommonDescribeFields>,
        page_response: PageResponse,
    ) -> Option<PageResponseWrapper> {
        trace!("got page response: {:?}", page_response);
        if let Some(cf) = common_fields {
            Some(PageResponseWrapper{
                target_id: cf.target_id.clone(),
                task_id: Some(cf.task_id),
                page_response,
            })
        } else {
            Some(PageResponseWrapper::new(page_response))
        }
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
                Ok(Some(PageResponseWrapper::new(PageResponse::SecondsElapsed(self.seconds_from_start))).into())
            }
            TaskDescribe::TargetCallMethod(target_call_method_task) => {
                Ok(Some(self.handle_target_method_call(target_call_method_task)).into())
            }
            TaskDescribe::PageEvent(page_event) => {
                Ok(Some(self.handle_page_event(page_event)).into())
            }
            TaskDescribe::RuntimeEvent(runtime_event) => {
                Ok(Some(self.handle_runtime_event(runtime_event)).into())
            }
            TaskDescribe::TargetEvent(target_event) => {
                Ok(Some(self.handle_target_event(target_event, session_id, target_id)).into())
            }
            TaskDescribe::DomEvent(dom_event) => Ok(Some(self.handle_dom_event(dom_event)).into()),
            TaskDescribe::ChromeConnected => {
                let resp = Some(PageResponseWrapper::new(PageResponse::ChromeConnected));
                Ok(resp.into())
            }

            // TaskDescribe::PageAttached(target_info, session_id) => {
            //     // each attach return different session_id.
            //     // when the chrome process started, it default creates a target and attach it. the url is: about:blank
            //     info!(
            //         "receive page attached event: {:?}, session_id: {:?}",
            //         target_info,
            //         session_id.clone()
            //     );
            //     match self.get_tab_by_id_mut(Some(&target_info.target_id)) {
            //         Ok(tab) => {
            //             tab.session_id.replace(session_id.clone());
            //             // tab.page_enable();
            //             let pr = (
            //                 Some(target_info.target_id.clone()),
            //                 None,
            //                 PageResponse::PageAttached(target_info, session_id),
            //             );
            //             Ok(Some(pr).into())
            //         }
            //         Err(error) => {
            //             error!("page attached event has caught, but cannot find corresponding tab. {:?}", error);
            //             self.send_fail(None, None)
            //         }
            //     }
            // }
            // TaskDescribe::PageEnable(page_enable) => {
            //     info!("page_enabled: {:?}", page_enable);
            //     let resp = self.convert_to_page_response(
            //         Some(&page_enable.common_fields),
            //         PageResponse::PageEnable,
            //     );
            //     Ok(resp.into())
            // }
            // // attached may not invoke, if invoked it's the first. then started, navigated, stopped.
            // TaskDescribe::FrameNavigated(frame, common_fields) => {
            //     info!(
            //         "-----------------frame_navigated-----------------{:?}",
            //         frame
            //     );
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     let frame_id = frame.id.clone();
            //     tab._frame_navigated(*frame);
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::FrameNavigated(frame_id),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::FrameStartedLoading(frame_id, common_fields) => {
            //     // started loading is first, then attached.
            //     info!(
            //         "-----------------frame_started_loading-----------------{:?}",
            //         frame_id
            //     );
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     tab._frame_started_loading(frame_id.clone());
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::FrameStartedLoading(frame_id),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::FrameStoppedLoading(frame_id, common_fields) => {
            //     info!(
            //         "-----------------frame_stopped_loading-----------------{:?}",
            //         frame_id
            //     );
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     tab._frame_stopped_loading(frame_id.clone());
            //     let pr = (
            //         common_fields.target_id,
            //         None,
            //         PageResponse::FrameStoppedLoading(frame_id),
            //     );
            //     Ok(Some(pr).into())
            // }
            // TaskDescribe::FrameAttached(frame_attached_params, common_fields) => {
            //     info!(
            //         "-----------------frame_attached-----------------{:?}",
            //         frame_attached_params.frame_id
            //     );
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     let frame_id = frame_attached_params.frame_id.clone();
            //     tab._frame_attached(frame_attached_params);
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::FrameAttached(frame_id),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::FrameDetached(frame_id, common_fields) => {
            //     info!(
            //         "-----------------frame_detached-----------------{:?}",
            //         frame_id.clone()
            //     );
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     tab._frame_detached(&frame_id);
            //     self.send_fail(None, None)
            // }
            // TaskDescribe::GetDocument(get_document) => {
            //     let common_fields = &get_document.common_fields;
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;

            //     tab.root_node = get_document.task_result;
            //     let resp =
            //         self.convert_to_page_response(Some(common_fields), PageResponse::GetDocument);
            //     Ok(resp.into())
            // }
            // TaskDescribe::SetChildNodes(target_id, parent_node_id, nodes) => {
            //     let tab = self.get_tab_by_id_mut(Some(&target_id))?;
            //     tab.node_arrived(parent_node_id, nodes);
            //     let pr = (
            //         Some(target_id),
            //         None,
            //         PageResponse::SetChildNodes(parent_node_id, vec![]),
            //     );
            //     Ok(Some(pr).into())
            // }
            // TaskDescribe::QuerySelector(query_selector) => {
            //     let pr = PageResponse::QuerySelector(
            //         query_selector.selector,
            //         query_selector.task_result,
            //     );
            //     let resp = self.convert_to_page_response(Some(&query_selector.common_fields), pr);
            //     Ok(resp.into())
            // }
            // TaskDescribe::DescribeNode(describe_node) => {
            //     let common_fields = &describe_node.common_fields;
            //     let node_id = describe_node
            //         .task_result
            //         .as_ref()
            //         .and_then(|n| Some(n.node_id));
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;

            //     tab.node_returned(describe_node.task_result);
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::DescribeNode(describe_node.selector, node_id),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::GetBoxModel(get_box_model) => {
            //     let common_fields = &get_box_model.common_fields;
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::GetBoxModel(
            //             get_box_model.selector,
            //             get_box_model.task_result.map(Box::new),
            //         ),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::LoadEventFired(target_id, timestamp) => {
            //     let pr = (
            //         Some(target_id),
            //         None,
            //         PageResponse::LoadEventFired(timestamp),
            //     );
            //     Ok(Some(pr).into())
            // }
            // TaskDescribe::CaptureScreenshot(screen_shot) => {
            //     let common_fields = &screen_shot.common_fields;
            //     let ro = response_object::CaptureScreenshot {
            //         selector: screen_shot.selector,
            //         base64: screen_shot.task_result,
            //     };
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::CaptureScreenshot(ro),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::RuntimeEvaluate(runtime_evaluate) => {
            //     let common_fields = &runtime_evaluate.common_fields;
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::RuntimeEvaluate(
            //             runtime_evaluate.task_result.map(Box::new),
            //             runtime_evaluate.exception_details.map(Box::new),
            //         ),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::ChromeConnected => {
            //     let resp = Some((None, None, PageResponse::ChromeConnected));
            //     Ok(resp.into())
            // }
            // TaskDescribe::RuntimeEnable(common_fields) => {
            //     let resp = self
            //         .convert_to_page_response(Some(&common_fields), PageResponse::RuntimeEnable);
            //     Ok(resp.into())
            // }
            // TaskDescribe::RuntimeExecutionContextCreated(
            //     runtime_execution_context_created,
            //     common_fields,
            // ) => {
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     let frame_id =
            //         tab.runtime_execution_context_created(runtime_execution_context_created);
            //     let resp = self.convert_to_page_response(
            //         Some(&common_fields),
            //         PageResponse::RuntimeExecutionContextCreated(frame_id),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::RuntimeExecutionContextDestroyed(
            //     runtime_execution_context_destroyed,
            //     common_fields,
            // ) => {
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     tab.runtime_execution_context_destroyed(runtime_execution_context_destroyed);
            //     self.send_fail_1(Some(&common_fields))
            // }
            // TaskDescribe::RuntimeConsoleAPICalled(console_api_called, common_fields) => {
            //     let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
            //     // let execution_context_id = console_api_called.execution_context_id.clone();
            //     tab.verify_execution_context_id(&console_api_called);
            //     self.send_fail(None, None)
            // }
            // TaskDescribe::TargetSetDiscoverTargets(value, _common_fields) => {
            //     assert!(value);
            //     self.send_fail(None, None)
            // }
            // TaskDescribe::TargetInfoChanged(target_info, common_fields) => {
            //     if let Ok(tab) = self.get_tab_by_id_mut(Some(&target_info.target_id)) {
            //         tab.target_info = target_info;
            //         trace!(
            //             "target info changed: {:?}, {:?}",
            //             tab.target_info,
            //             common_fields
            //         );
            //     } else {
            //         warn!(
            //             "target changed, no correspond tab. {:?}, {:?}",
            //             target_info, common_fields
            //         );
            //     }
            //     self.send_fail(None, None)
            // }
            // TaskDescribe::NavigateTo(navigate_to) => {
            //     trace!("navigate_to: {:?}", navigate_to);
            //     self.send_fail(None, None)
            // }
            // TaskDescribe::RuntimeGetProperties(get_properties) => {
            //     let resp = self.convert_to_page_response(
            //         Some(&get_properties.common_fields),
            //         PageResponse::RuntimeGetProperties(get_properties.task_result),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::RuntimeCallFunctionOn(task) => {
            //     let resp = self.convert_to_page_response(
            //         Some(&task.common_fields),
            //         PageResponse::RuntimeCallFunctionOn(task.task_result),
            //     );
            //     Ok(resp.into())
            // }
            // TaskDescribe::PrintToPDF(task) => {
            //     let resp = self.convert_to_page_response(
            //         Some(&task.common_fields),
            //         PageResponse::PrintToPDF(task.task_result),
            //     );
            //     Ok(resp.into())
            // }
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
