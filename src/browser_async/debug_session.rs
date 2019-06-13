use super::chrome_browser::ChromeBrowser;
use super::chrome_debug_session::ChromeDebugSession;
use super::interval_page_message::IntervalPageMessage;
use super::page_message::{PageResponse, PageResponseWrapper, ReceivedEvent};
use super::Tab;
use super::task_describe::{
    self as tasks, handle_browser_method_call, handle_target_method_call, 
    DomEvent, RuntimeEnableTask, RuntimeEvent, SecurityEnableTask,
    SetDiscoverTargetsTask, SetIgnoreCertificateErrorsTask,  TargetEvent,
    TaskDescribe, handle_network_event, handle_page_event,
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
        self.chrome_debug_session.lock().expect("obtain chrome_debug_session should success.").poll()
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
    pub tabs: Vec<Tab>, // early created at front.
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

    pub fn activates_next_in_interval(&mut self, secs: u64) {
        let tab = self.tabs.iter_mut().find(|tb|tb.activated_at.is_some());
        let mut need_wrapup = false;
        if let Some(activated_tab) = tab {
            if let Some(activated_at) = activated_tab.activated_at {
                if activated_at.elapsed().as_secs() > secs {
                    let mut tabs = self.tabs.iter_mut().skip_while(|tb|tb.activated_at.is_none());
                    let __discard = tabs.next();
                    if let Some(tb) = tabs.next() {
                        tb.bring_to_front();
                    } else {
                        need_wrapup = true;
                    }
                }
            }
        } else {
            need_wrapup = true;
        }
        if need_wrapup {
            if let Some(tb) = self.tabs.get_mut(0) {
                tb.bring_to_front();
            }
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
    pub fn first_page(&self) -> Option<&Tab> {
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
        let task = self.runtime_enable_task();
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn runtime_enable_task(&mut self) -> TaskDescribe {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        RuntimeEnableTask { common_fields }.into()
    }

    pub fn execute_one_task(&mut self, task: TaskDescribe) {
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn execute_tasks(&mut self, tasks: Vec<TaskDescribe>) {
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(tasks);
    }

    pub fn set_ignore_certificate_errors(&mut self, ignore: bool) {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        let task = self.set_ignore_certificate_errors_task(ignore);
        self.execute_one_task(task);
    }

    pub fn set_ignore_certificate_errors_task(&mut self, ignore: bool) -> TaskDescribe {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        SetIgnoreCertificateErrorsTask {
            common_fields,
            ignore,
        }.into()
    }

    pub fn set_discover_targets(&mut self, enable: bool) {
        let task = self.set_discover_targets_task(enable);
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task.into()]);
    }

    pub fn set_discover_targets_task(&mut self, enable: bool) -> TaskDescribe {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        SetDiscoverTargetsTask {
            common_fields,
            discover: enable,
        }.into()
    }

    pub fn security_enable(&mut self) {
        let task = self.security_enable_task();
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task.into()]);
    }

    pub fn security_enable_task(&mut self) -> TaskDescribe {
        let common_fields = tasks::CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        SecurityEnableTask { common_fields }.into()
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
                    page_response: PageResponse::ReceivedEvent(ReceivedEvent::SetChildNodesOccurred(parent_id)),
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
                        page_response: PageResponse::ReceivedEvent(ReceivedEvent::PageCreated(idx)),
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
                    // tab.session_id.replace(event.get_session_id());
                    tab.page_attached(event.get_session_id());
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
                    trace!("target info changed: {:?}", tab.target_info);
                } else {
                    warn!("target changed, no correspond tab. {:?}", target_info);
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
                    .runtime_execution_context_created(event.get_execution_context_description());
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::ExecutionContextCreated(event)),
                );
            }
            RuntimeEvent::ExecutionContextDestroyed(event) => {
                let execution_context_id = event.into_execution_context_id();
                let tab = self.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.runtime_execution_context_destroyed(execution_context_id);
            }
            RuntimeEvent::ExecutionContextsCleared(event) => {}
            RuntimeEvent::InspectRequested(event) => {}
        }
        warn!("unhandled branch handle_runtime_event");
        Ok(PageResponseWrapper::default())
    }

    // fn handle_page_event(
    //     &mut self,
    //     page_event: PageEvent,
    //     maybe_session_id: Option<target::SessionID>,
    //     maybe_target_id: Option<target::TargetId>,
    // ) -> Result<PageResponseWrapper, failure::Error> {

    // }


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
                self.tabs.iter_mut().for_each(Tab::run_task_queue);
                Ok(Some(PageResponseWrapper::new(PageResponse::SecondsElapsed(
                    self.seconds_from_start,
                )))
                .into())
            }
            TaskDescribe::TargetCallMethod(task) => {
                Ok(handle_target_method_call(self, task, session_id, target_id)
                    .ok()
                    .into())
            }
            TaskDescribe::PageEvent(page_event) => Ok(handle_page_event(self, page_event, session_id, target_id)
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
                Ok(handle_browser_method_call(task, session_id, target_id)
                    .ok()
                    .into()),
            
            TaskDescribe::NetworkEvent(network_event) => 
                Ok(handle_network_event(self, network_event, session_id, target_id)
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
            Async::Ready(Some(item)) => {return self.send_page_message(item);}
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
                return self.send_page_message(item);
            }
            Async::Ready(None) if a_done => Ok(None.into()),
            Async::Ready(None) | Async::NotReady => Ok(Async::NotReady),
        }
    }
}
