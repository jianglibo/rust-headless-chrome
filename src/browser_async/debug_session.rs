use super::chrome_browser::ChromeBrowser;
use super::chrome_debug_session::ChromeDebugSession;
use super::interval_page_message::IntervalPageMessage;
use super::page_message::{PageResponse, PageResponseWrapper};
use super::task_describe::{
    handle_browser_method_call, handle_dom_event, handle_network_event, handle_page_event,
    handle_runtime_event, handle_target_event, handle_target_method_call, target_tasks, handle_log_event,
    CommonDescribeFieldsBuilder, RuntimeEnableTask, SecurityEnableTask, SetDiscoverTargetsTask, GetTargetsTask,
    SetIgnoreCertificateErrorsTask, TaskDescribe, GetBrowserCommandLineTask,
};
use super::{BrowserContexts, Tab};

use super::super::browser::process::{LaunchOptions, LaunchOptionsBuilder};
use super::protocol::target;
use super::ChromePageError;
use failure;
use futures::{Async, Poll};
use log::*;
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
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .poll()
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
    pub tabs: Vec<Tab>, // early created at front.
    wrapper: Wrapper,
    bring_to_front_in_process: bool,
}

impl Default for DebugSession {
    fn default() -> Self {
        Self::new_headless()
    }
}

impl DebugSession {
    pub fn new_headless() -> Self {
        let options = LaunchOptionsBuilder::default()
            .build()
            .expect("default launch_options should created.");
        Self::new(options)
    }

    pub fn new_visible() -> Self {
        let options = LaunchOptionsBuilder::default()
            .headless(false)
            .build()
            .expect("default launch_options should created.");
        Self::new(options)
    }

    pub fn new(launch_options: LaunchOptions) -> Self {
        let browser = ChromeBrowser::new(launch_options);
        Self::new_default(browser)
    }

    fn new_default(browser: ChromeBrowser) -> Self {
        let chrome_debug_session = ChromeDebugSession::new(browser);
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
            bring_to_front_in_process: false,
        }
    }

    pub fn loaded_by_this_tab_id(&self, target_id: &str) -> Vec<&Tab> {
        self.tabs
            .iter()
            .filter(|tb| tb.target_info.opener_id == Some(target_id.to_string()))
            .collect()
    }

    pub fn loaded_by_this_tab_name_count(&self, name: &str) -> usize {
        if let Ok(tab) = self.find_tab_by_name(name) {
            self.tabs
                .iter()
                .filter(|tb| tb.target_info.opener_id.as_ref() == Some(&tab.target_info.target_id))
                .count()
        } else {
            0
        }
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    pub fn run_manually_tasks(&mut self) {
        self.tabs.iter_mut().for_each(Tab::run_task_queue_manually);
    }

    pub fn count_manually_tasks(&self) -> usize {
        self.tabs.iter().map(Tab::count_task_queue_manually).sum()
    }

    pub fn loaded_by_this_tab_name_mut(&mut self, name: &str) -> Vec<&mut Tab> {
        if let Ok(tab) = self.find_tab_by_name(name) {
            let target_id = tab.target_info.target_id.clone();
            self.tabs
                .iter_mut()
                .filter(|tb| tb.target_info.opener_id.as_ref() == Some(&target_id))
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn loaded_by_this_tab(&self, tab: &Tab) -> Vec<&Tab> {
        self.tabs
            .iter()
            .filter(|tb| tb.target_info.opener_id.as_ref() == Some(&tab.target_info.target_id))
            .collect()
    }

    pub fn browser_contexts(&mut self) -> BrowserContexts {
        BrowserContexts {
            all_tabs: &mut self.tabs[..],
        }
    }

    pub fn find_tabs_old_than(&mut self, secs: u64) -> Vec<&mut Tab> {
        self.tabs
            .iter_mut()
            .filter(|tb| !tb.explicitly_close)
            .filter(|tb| tb.created_at.elapsed().as_secs() > secs)
            .collect()
    }

    pub fn close_tab_old_than(&mut self, secs: u64) {
        self.find_tabs_old_than(secs)
            .into_iter()
            .for_each(Tab::page_close);
    }

    pub fn close_tab_by_close_target_old_than(&mut self, secs: u64) {
        self.find_tabs_old_than(secs)
            .into_iter()
            .for_each(Tab::close);
    }

    pub fn close_tab_by_window_close_old_than(&mut self, secs: u64) {
        self.find_tabs_old_than(secs)
            .into_iter()
            .for_each(Tab::close_by_window_close);
    }

    pub fn find_last_opened_tab(&mut self) -> Option<&mut Tab> {
        if let Some((first, rest)) = self.tabs.split_first_mut() {
            let mut last_opened = first;
            for tb in rest {
                if tb.created_at > last_opened.created_at {
                    last_opened = tb;
                }
            }
            Some(last_opened)
        } else {
            None
        }
    }

    pub fn activate_last_opened_tab(&mut self) {
        if let Some(tab) = self.find_last_opened_tab() {
            tab.bring_to_front();
        }
    }

    pub fn activates_next_in_interval(&mut self, secs: u64) {
        if self.bring_to_front_in_process || self.tabs.len() < 2 {
            return;
        }
        let tab = self.tabs.iter_mut().find(|tb| tb.activated_at.is_some());
        let mut need_return_back = false;
        if let Some(activated_tab) = tab {
            if let Some(activated_at) = activated_tab.activated_at {
                if activated_at.elapsed().as_secs() > secs {
                    let mut tabs = self
                        .tabs
                        .iter_mut()
                        .skip_while(|tb| tb.activated_at.is_none());
                    let __discard = tabs.next();
                    if let Some(tb) = tabs.next() {
                        tb.bring_to_front();
                    } else {
                        need_return_back = true;
                    }
                }
            }
        } else {
            // no tab was activated.
            need_return_back = true;
        }
        if need_return_back {
            if let Some(tb) = self.tabs.get_mut(0) {
                tb.bring_to_front();
            }
        }
    }

    pub fn get_tab_by_resp_mut(
        &mut self,
        page_response_wrapper: &PageResponseWrapper,
    ) -> Result<&mut Tab, failure::Error> {
        self.find_tab_by_id_mut(page_response_wrapper.target_id.as_ref())
    }
    pub fn find_tab_by_id_mut(
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

    pub fn find_tab_by_name(&self, name: &str) -> Result<&Tab, failure::Error> {
        if let Some(tab) = self.tabs.iter().find(|t| t.page_name == Some(name)) {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn find_tab_not_in_name_mut(&mut self, name: &str) -> Result<&mut Tab, failure::Error> {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.page_name != Some(name)) {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn find_tab_by_name_mut(&mut self, name: &str) -> Result<&mut Tab, failure::Error> {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.page_name == Some(name)) {
            Ok(tab)
        } else {
            Err(ChromePageError::TabNotFound.into())
        }
    }

    pub fn tab_closed(&mut self, target_id: &str) {
        self.tabs.retain(|tb| {
            if tb.target_info.target_id == target_id {
                info!("tab closed: {:?}", tb);
            }
            tb.target_info.target_id != target_id
        });
    }

    pub fn bring_to_front_responded(
        &mut self,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<(), failure::Error> {
        self.tabs.iter_mut().for_each(|tb| {
            tb.activated_at.take();
        });
        let tab = self.find_tab_by_id_mut(maybe_target_id.as_ref())?;
        tab.bring_to_front_responded();
        self.bring_to_front_in_process = false;
        Ok(())
    }

    pub fn create_new_tab(&mut self, url: &str) {
        let task = self.create_new_tab_task_impl(url, None);
        self.execute_one_task(task);
    }

    pub fn create_new_tab_named(&mut self, url: &str, name: &str) {
        let task = self.create_new_tab_task_impl(url, Some(name));
        self.execute_one_task(task);
    }

    fn create_new_tab_task_impl(&mut self, url: &str, name: Option<&str>) -> TaskDescribe {
        let common_fields = CommonDescribeFieldsBuilder::default()
            .task_id(name.map(Into::into))
            .build()
            .expect("create common_fields should success.");
        let task = target_tasks::CreateTargetTaskBuilder::default()
            .common_fields(common_fields)
            .url(url.to_owned())
            .build()
            .expect("CreateTargetTaskBuilder should success.");
        task.into()
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

    pub fn runtime_enable(&mut self) {
        let task = self.runtime_enable_task();
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn runtime_enable_task(&mut self) -> TaskDescribe {
        let common_fields = CommonDescribeFieldsBuilder::default()
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
        let task = self.set_ignore_certificate_errors_task(ignore);
        self.execute_one_task(task);
    }

    pub fn set_ignore_certificate_errors_task(&mut self, ignore: bool) -> TaskDescribe {
        let common_fields = CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        SetIgnoreCertificateErrorsTask {
            common_fields,
            ignore,
        }
        .into()
    }

    pub fn set_discover_targets(&mut self, enable: bool) {
        let task = self.set_discover_targets_task(enable);
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn set_discover_targets_task(&mut self, enable: bool) -> TaskDescribe {
        let common_fields = CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        SetDiscoverTargetsTask {
            common_fields,
            discover: enable,
        }
        .into()
    }

    pub fn security_enable(&mut self) {
        let task = self.security_enable_task();
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn security_enable_task(&self) -> TaskDescribe {
        let common_fields = CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        SecurityEnableTask { common_fields }.into()
    }

    pub fn get_browser_command_line(&mut self) {
        let task = self.get_browser_command_line_task();
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn get_browser_command_line_task(&self) -> TaskDescribe {
         let common_fields = CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        GetBrowserCommandLineTask { common_fields, task_result: None }.into()
    }

    pub fn get_targets(&mut self) {
        let task = self.get_targets_task();
        self.chrome_debug_session
            .lock()
            .expect("obtain chrome_debug_session should success.")
            .execute_task(vec![task]);
    }

    pub fn get_targets_task(&self) -> TaskDescribe {
         let common_fields = CommonDescribeFieldsBuilder::default()
            .build()
            .expect("build common_fields should success.");
        GetTargetsTask { common_fields, task_result: None }.into()
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
                self.tabs.iter_mut().for_each(Tab::run_task_queue_delayed);
                self.tabs
                    .iter_mut()
                    .for_each(Tab::move_mouse_random_interval);
                self.chrome_debug_session
                    .lock()
                    .expect("obtain chrome_debug_session should success.")
                    .check_stalled_tasks();
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
            TaskDescribe::PageEvent(page_event) => {
                Ok(handle_page_event(self, page_event, session_id, target_id)
                    .ok()
                    .into())
            }
            TaskDescribe::RuntimeEvent(runtime_event) => {
                Ok(
                    handle_runtime_event(self, runtime_event, session_id, target_id)
                        .ok()
                        .into(),
                )
            }
            TaskDescribe::TargetEvent(target_event) => {
                Ok(
                    handle_target_event(self, target_event, session_id, target_id)
                        .ok()
                        .into(),
                )
            }
            TaskDescribe::DomEvent(dom_event) => {
                Ok(handle_dom_event(self, dom_event, session_id, target_id)
                    .ok()
                    .into())
            }
            TaskDescribe::LogEvent(log_event) => {
                Ok(handle_log_event(self, log_event, session_id, target_id)
                    .ok()
                    .into())                
            }
            TaskDescribe::ChromeConnected => {
                let resp = Some(PageResponseWrapper::new(PageResponse::ChromeConnected));
                Ok(resp.into())
            }
            TaskDescribe::BrowserCallMethod(task) => {
                Ok(handle_browser_method_call(task, session_id, target_id)
                    .ok()
                    .into())
            }

            TaskDescribe::NetworkEvent(network_event) => {
                Ok(
                    handle_network_event(self, network_event, session_id, target_id)
                        .ok()
                        .into(),
                )
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
            Async::Ready(Some(item)) => {
                return self.send_page_message(item);
            }
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
