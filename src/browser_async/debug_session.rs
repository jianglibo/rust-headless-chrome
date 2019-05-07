use super::chrome_browser::ChromeBrowser;
use super::chrome_debug_session::ChromeDebugSession;
use super::dev_tools_method_util::ChromePageError;
use super::id_type as ids;
use super::interval_page_message::IntervalPageMessage;
use super::page_message::{response_object, PageResponse, PageResponseWithTargetIdTaskId};
use super::tab::Tab;
use super::task_describe::{self as tasks, CommonDescribeFields, TaskDescribe};
use crate::protocol::target;
use failure;
use futures::{Async, Poll};
use log::*;
use std::collections::HashMap;
use std::default::Default;
use std::sync::{Arc, Mutex};
use websocket::futures::Stream;

const DEFAULT_TAB_NAME: &str = "_default_tab_";

struct Wrapper {
    pub chrome_debug_session: Arc<Mutex<ChromeDebugSession>>,
}

impl Stream for Wrapper {
    type Item = TaskDescribe;
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
    tabs: HashMap<&'static str, Tab>,
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
            tabs: HashMap::new(),
            wrapper: Wrapper {
                chrome_debug_session: arc_cds,
            },
        }
    }
    pub fn get_tab_by_id_mut(
        &mut self,
        target_id: Option<&target::TargetId>,
    ) -> Result<&mut Tab, failure::Error> {
        if let Some(tab) = self
            .tabs
            .values_mut()
            .find(|t| Some(&t.target_info.target_id) == target_id)
        {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn get_tab_by_id(
        &self,
        target_id: Option<&target::TargetId>,
    ) -> Result<&Tab, failure::Error> {
        if let Some(tab) = self
            .tabs
            .values()
            .find(|t| Some(&t.target_info.target_id) == target_id)
        {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn main_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(DEFAULT_TAB_NAME)
    }
    pub fn main_tab(&self) -> Option<&Tab> {
        self.tabs.get(DEFAULT_TAB_NAME)
    }

    fn send_fail(
        &mut self,
        target_id: Option<target::TargetId>,
        task_id: Option<ids::Task>,
    ) -> Poll<Option<PageResponseWithTargetIdTaskId>, failure::Error> {
        let pr = (target_id, task_id, PageResponse::Fail);
        Ok(Some(pr).into())
    }

    pub fn runtime_enable(&mut self) {
        let cf = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .unwrap();
        self.chrome_debug_session
            .lock()
            .unwrap()
            .execute_task(vec![TaskDescribe::RuntimeEnable(cf)]);
    }

    pub fn set_discover_targets(&mut self, enable: bool) {
        let cf = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .unwrap();
        self.chrome_debug_session
            .lock()
            .unwrap()
            .execute_task(vec![TaskDescribe::TargetSetDiscoverTargets(enable, cf)]);
    }

    fn send_fail_1(
        &mut self,
        common_fields: Option<&CommonDescribeFields>,
    ) -> Poll<Option<PageResponseWithTargetIdTaskId>, failure::Error> {
        if let Some(cf) = common_fields {
            let pr = (cf.target_id.clone(), Some(cf.task_id), PageResponse::Fail);
            Ok(Some(pr).into())
        } else {
            let pr = (None, None, PageResponse::Fail);
            Ok(Some(pr).into())
        }
    }

    fn convert_to_page_response(
        &self,
        common_fields: Option<&CommonDescribeFields>,
        page_response: PageResponse,
    ) -> Option<PageResponseWithTargetIdTaskId> {
        trace!("got page response: {:?}", page_response);
        if let Some(cf) = common_fields {
            Some((cf.target_id.clone(), Some(cf.task_id), page_response))
        } else {
            Some((None, None, page_response))
        }
    }

    pub fn send_page_message(
        &mut self,
        item: TaskDescribe,
    ) -> Poll<Option<PageResponseWithTargetIdTaskId>, failure::Error> {
        match item {
            TaskDescribe::Interval => {
                self.seconds_from_start += 1;
                let pr = (
                    None,
                    None,
                    PageResponse::SecondsElapsed(self.seconds_from_start),
                );
                Ok(Some(pr).into())
            }
            TaskDescribe::PageCreated(target_info, page_name) => {
                trace!(
                    "receive page created event: {:?}, {:?}",
                    target_info,
                    page_name
                );
                let target_id = target_info.target_id.clone();
                let mut tab = Tab::new(target_info, Arc::clone(&self.chrome_debug_session));
                tab.attach_to_page();
                self.tabs.insert(page_name.unwrap_or(DEFAULT_TAB_NAME), tab);
                let pr = (Some(target_id), None, PageResponse::PageCreated(page_name));
                Ok(Some(pr).into())
            }
            TaskDescribe::PageAttached(target_info, session_id) => {
                trace!(
                    "receive page attached event: {:?}, {:?}",
                    target_info,
                    session_id.clone()
                );
                let tab = self.get_tab_by_id_mut(Some(&target_info.target_id))?;
                tab.session_id.replace(session_id.clone());
                tab.page_enable();
                let pr = (
                    Some(target_info.target_id.clone()),
                    None,
                    PageResponse::PageAttached(target_info, session_id),
                );
                Ok(Some(pr).into())
            }
            TaskDescribe::PageEnable(page_enable) => {
                let resp =
                    self.convert_to_page_response(Some(&page_enable), PageResponse::PageEnable);
                Ok(resp.into())
            }
            // attached may not invoke, if invoked it's the first. then started, navigated, stopped.
            TaskDescribe::FrameNavigated(frame, common_fields) => {
                // error!("-----------------frame_navigated-----------------{:?}", frame.id);
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                let frame_id = frame.id.clone();
                tab._frame_navigated(*frame);
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::FrameNavigated(frame_id),
                );
                Ok(resp.into())
            }
            TaskDescribe::FrameStartedLoading(frame_id, common_fields) => {
                // started loading is first, then attached.
                // error!("-----------------frame_started-----------------{:?}", frame_id);
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                tab._frame_started_loading(frame_id.clone());
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::FrameStartedLoading(frame_id),
                );
                Ok(resp.into())
            }
            TaskDescribe::FrameStoppedLoading(frame_id, common_fields) => {
                // error!("-----------------frame_stopped-----------------{:?}", frame_id);
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                tab._frame_stopped_loading(frame_id.clone());
                let pr = (
                    common_fields.target_id,
                    None,
                    PageResponse::FrameStoppedLoading(frame_id),
                );
                Ok(Some(pr).into())
            }
            TaskDescribe::FrameAttached(frame_attached_params, common_fields) => {
                // error!("-----------------frame_attached-----------------{:?}", frame_attached_params.frame_id);
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                let frame_id = frame_attached_params.frame_id.clone();
                tab._frame_attached(frame_attached_params);
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::FrameAttached(frame_id),
                );
                Ok(resp.into())
            }
            TaskDescribe::GetDocument(get_document) => {
                let common_fields = &get_document.common_fields;
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;

                tab.root_node = get_document.root_node;
                let resp =
                    self.convert_to_page_response(Some(common_fields), PageResponse::GetDocument);
                Ok(resp.into())
            }
            TaskDescribe::SetChildNodes(target_id, parent_node_id, nodes) => {
                let tab = self.get_tab_by_id_mut(Some(&target_id))?;
                tab.node_arrived(parent_node_id, nodes);
                let pr = (
                    Some(target_id),
                    None,
                    PageResponse::SetChildNodes(parent_node_id, vec![]),
                );
                Ok(Some(pr).into())
            }
            TaskDescribe::QuerySelector(query_selector) => {
                let pr = PageResponse::QuerySelector(
                    query_selector.selector,
                    query_selector.found_node_id,
                );
                let resp = self.convert_to_page_response(Some(&query_selector.common_fields), pr);
                Ok(resp.into())
            }
            TaskDescribe::DescribeNode(describe_node) => {
                let common_fields = &describe_node.common_fields;
                let node_id = describe_node
                    .found_node
                    .as_ref()
                    .and_then(|n| Some(n.node_id));
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;

                tab.node_returned(describe_node.found_node);
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::DescribeNode(describe_node.selector, node_id),
                );
                Ok(resp.into())
            }
            TaskDescribe::GetBoxModel(get_box_model) => {
                let common_fields = &get_box_model.common_fields;
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::GetBoxModel(
                        get_box_model.selector,
                        get_box_model.found_box.map(Box::new),
                    ),
                );
                Ok(resp.into())
            }
            TaskDescribe::LoadEventFired(target_id, timestamp) => {
                let pr = (
                    Some(target_id),
                    None,
                    PageResponse::LoadEventFired(timestamp),
                );
                Ok(Some(pr).into())
            }
            TaskDescribe::ScreenShot(screen_shot) => {
                let common_fields = &screen_shot.common_fields;
                let ro = response_object::CaptureScreenshot {
                    selector: screen_shot.selector,
                    base64: screen_shot.base64,
                };
                let resp = self
                    .convert_to_page_response(Some(&common_fields), PageResponse::Screenshot(ro));
                Ok(resp.into())
            }
            TaskDescribe::RuntimeEvaluate(runtime_evaluate) => {
                let common_fields = &runtime_evaluate.common_fields;
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::RuntimeEvaluate(
                        runtime_evaluate.result.map(Box::new),
                        runtime_evaluate.exception_details.map(Box::new),
                    ),
                );
                Ok(resp.into())
            }
            TaskDescribe::ChromeConnected => {
                let resp = Some((None, None, PageResponse::ChromeConnected));
                Ok(resp.into())
            }
            TaskDescribe::RuntimeEnable(common_fields) => {
                let resp = self
                    .convert_to_page_response(Some(&common_fields), PageResponse::RuntimeEnable);
                Ok(resp.into())
            }
            TaskDescribe::RuntimeExecutionContextCreated(
                runtime_execution_context_created,
                common_fields,
            ) => {
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                let frame_id = tab.runtime_execution_context_created(runtime_execution_context_created);
                let resp = self.convert_to_page_response(
                    Some(&common_fields),
                    PageResponse::RuntimeExecutionContextCreated(frame_id),
                );
                Ok(resp.into())
            }
            TaskDescribe::RuntimeExecutionContextDestroyed(
                runtime_execution_context_destroyed,
                common_fields,
            ) => {
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                tab.runtime_execution_context_destroyed(runtime_execution_context_destroyed);
                self.send_fail_1(Some(&common_fields))
            }
            TaskDescribe::RuntimeConsoleAPICalled(console_api_called, common_fields) => {
                let tab = self.get_tab_by_id_mut(common_fields.target_id.as_ref())?;
                // let execution_context_id = console_api_called.execution_context_id.clone();
                tab.verify_execution_context_id(&console_api_called);
                self.send_fail(None, None)
            }
            TaskDescribe::TargetSetDiscoverTargets(value, _common_fields) => {
                assert!(value);
                self.send_fail(None, None)
            }
            TaskDescribe::TargetInfoChanged(target_info, common_fields) => {
                if let Ok(tab) = self.get_tab_by_id_mut(common_fields.target_id.as_ref()) {
                    if tab.target_info.target_id == target_info.target_id {
                        trace!("got main target.");
                    } else {
                        warn!("got target_info, with different id {:?}", target_info);
                    }
                } else {
                    warn!("target changed, no correspond tab. {:?}", target_info);
                }
                self.send_fail(None, None)
            }
            TaskDescribe::NavigateTo(navigate_to) => {
                trace!("navigate_to: {:?}", navigate_to);
                self.send_fail(None, None)
            }
            _ => {
                warn!("debug_session got unknown task. {:?}", item);
                self.send_fail(None, None)
            }
        }
    }
}

impl Stream for DebugSession {
    type Item = PageResponseWithTargetIdTaskId;
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
