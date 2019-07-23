use super::super::browser::tab::{element::BoxModel, point::Point};
use super::super::browser_async::{embedded_events, ChromeDebugSession, NetworkStatistics, TaskId};

use super::super::protocol::{self, dom, network, page, runtime, target};
use super::page_message::ChangingFrame;
use super::task_describe::{
    dom_tasks, input_tasks, network_events, network_tasks, page_events, page_tasks, runtime_tasks, HasSessionId,
    target_tasks, CommonDescribeFields, CommonDescribeFieldsBuilder, TaskDescribe, TargetCallMethodTask,
};
use super::{EventName, EventStatistics, TaskQueue};
use log::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;
mod box_model_func;
mod emulation_func;
mod evaluate_func;
mod screen_shot_func;

// #[derive(Debug, PartialEq, Eq, Hash)]
// pub enum WaitingForPageAttachTaskName {
//     RuntimeEnable,
//     PageEnable,
//     NetworkEnable,
//     SetLifecycleEventsEnabled,
//     BringToFront,
// }

// #[derive(Debug)]
// enum TabState {
//     Normal,
//     // Closing,
// }

#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub created_at: Instant,
    pub activated_at: Option<Instant>,
    pub target_info: protocol::target::TargetInfo,
    pub session_id: Option<target::SessionID>,
    pub root_node: Option<dom::Node>,
    pub page_name: Option<&'static str>,
    pub changing_frames: HashMap<page::FrameId, ChangingFrame>,
    pub temporary_node_holder: HashMap<dom::NodeId, Vec<dom::Node>>,
    pub execution_context_descriptions:
        HashMap<page::FrameId, runtime::ExecutionContextDescription>,
    // pub ongoing_request: HashMap<network::RequestId, network_events::RequestWillBeSent>,
    pub request_intercepted: HashMap<network::RequestId, network_events::RequestIntercepted>,
    pub response_received: HashMap<network::RequestId, network_events::ResponseReceived>,
    pub event_statistics: EventStatistics,
    pub task_queue: TaskQueue,
    // pub waiting_for_page_attach: HashSet<WaitingForPageAttachTaskName>,
    pub waiting_for_page_attach_tasks: Vec<TaskDescribe>,
    pub activating: bool,
    pub closing: bool,
    pub explicitly_close: bool,
    pub life_cycles: Vec<page_events::LifeCycle>,
    pub network_statistics: NetworkStatistics,
    pub box_model: Option<BoxModel>,
}

impl std::fmt::Display for Tab {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({:?}, activated_at: {:?}, {}, attached: {}, opener_id: {:?}, browser_context_id: {:?})", 
        self.created_at,
        self.activated_at,
        self.get_url(),
        self.target_info.attached,
        self.target_info.opener_id,
        self.target_info.browser_context_id,)
    }
}

impl Tab {
    pub fn new(
        target_info: protocol::target::TargetInfo,
        chrome_session: Arc<Mutex<ChromeDebugSession>>,
    ) -> Self {
        Self {
            target_info,
            chrome_session,
            session_id: None,
            root_node: None,
            page_name: None,
            changing_frames: HashMap::new(),
            temporary_node_holder: HashMap::new(),
            execution_context_descriptions: HashMap::new(),
            request_intercepted: HashMap::new(),
            response_received: HashMap::new(),
            created_at: Instant::now(),
            activated_at: None,
            // waiting_for_page_attach: HashSet::new(),
            waiting_for_page_attach_tasks: Vec::new(),
            activating: false,
            closing: false,
            explicitly_close: false,
            life_cycles: Vec::new(),
            event_statistics: EventStatistics::new(),
            network_statistics: NetworkStatistics::default(),
            task_queue: TaskQueue::new(),
            box_model: None,
        }
    }

    /// invoking mulitiple tasks and one event loop may lost response. So flatten the tasks. No!!!
    /// spliting tasks to groups to get each last task to respose to caller.
    pub fn run_task_queue_delayed(&mut self) {
        // let tasks = self.task_queue.retrieve_delayed_task_to_run().into_iter().flatten().collect::<Vec<TaskDescribe>>();
        // if !tasks.is_empty() {
        //     self.execute_tasks(tasks);
        // }
        for task_vec in self.task_queue.retrieve_delayed_task_to_run() {
            if !task_vec.is_empty() {
                self.execute_tasks(task_vec);
            }
        }
    }

    pub fn run_task_queue_manually(&mut self) {
        if let Some(tasks) = self.task_queue.retrieve_manually_task_to_run() {
            self.execute_tasks(tasks);
        }
    }

    pub fn is_blank_url(&self) -> bool {
        self.is_at_url("about:blank")
    }

    pub fn is_chrome_error_chromewebdata(&self) -> bool {
        self.is_at_url("chrome-error://chromewebdata/")
    }

    // pub fn request_will_be_sent(&mut self, event: network_events::RequestWillBeSent) {
    //     self.ongoing_request.insert(event.get_request_id(), event);
    // }

    /// where does page's url attribute live? The page target_info holds the url you intent navigate to,
    /// but if failed cause of some reason, please look into the main frame's url and unreachable_url attributes,
    /// These two will give you more information.
    pub fn is_at_url(&self, url: &str) -> bool {
        if let Some(mf) = self.main_frame() {
            mf.url == url
        } else {
            self.target_info.url == url
        }
    }

    pub fn life_cycle_happened(&mut self, life_cycle_event: page_events::LifeCycle) {
        self.life_cycles.push(life_cycle_event);
    }

    pub fn last_life_cycle_event(&self) -> &page_events::LifeCycle {
        self.life_cycles
            .last()
            .expect("when last_life_cycle_event is called, it should already have events exists.")
    }

    pub fn life_cycle_event_count(&self, name: &str) -> usize {
        self.life_cycles
            .iter()
            .filter(|lc| lc.get_name() == name)
            .count()
    }

    pub fn get_url(&self) -> &str {
        if let Some(mf) = self.main_frame() {
            &mf.url
        } else {
            &self.target_info.url
        }
    }

    pub fn url_in(&self, urls: Vec<&str>) -> bool {
        urls.contains(&self.get_url())
    }

    pub fn page_close(&mut self) {
        if self.closing {
            return;
        } else {
            self.closing = true;
            let task = page_tasks::PageCloseTaskBuilder::default()
                .common_fields(self.get_common_field(None))
                .build()
                .expect("build PageCloseTaskBuilder should success.");
            self.execute_one_task(task.into());
        }
    }

    pub fn close(&mut self) {
        if self.closing {
            return;
        } else {
            self.closing = true;
            let task = target_tasks::CloseTargetTaskBuilder::default()
                .common_fields(self.get_common_field(None))
                .build()
                .expect("build BringToFrontTaskBuilder should success.");
            self.execute_one_task(task.into());
        }
    }

    pub fn bring_to_front(&mut self) -> bool {
        if self.activated_at.is_some() || self.activating {
            return false;
        }
        self.activating = true;
        let task = self.bring_to_front_task();
        if self.session_id.is_some() {
            self.execute_one_task(task);
        } else {
            self.waiting_for_page_attach_tasks.push(task);
                // .insert(WaitingForPageAttachTaskName::BringToFront);
            // self.attach_to_page();
        }
        true
    }

    pub fn bring_to_front_responded(&mut self) {
        self.activated_at.replace(Instant::now());
        self.activating = false;
        info!(
            "page {:?} activated_at: {:?}",
            self.get_url(),
            self.activated_at
        );
    }

    fn bring_to_front_task(&mut self) -> TaskDescribe {
        let task = page_tasks::BringToFrontTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .build()
            .expect("build BringToFrontTaskBuilder should success.");
        task.into()
    }

    pub fn navigate_to_named(&mut self, url: &'static str, name: &str) {
        let task = self.navigate_to_task(url, Some(name.to_owned()));
        self.execute_one_task(task);
    }

    pub fn navigate_to(&mut self, url: &'static str) {
        let task = self.navigate_to_task(url, None);
        self.execute_one_task(task);
    }

    pub fn navigate_to_task(
        &self,
        url: &'static str,
        manual_task_id: Option<TaskId>,
    ) -> TaskDescribe {
        let task = page_tasks::NavigateToTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .url(url)
            .build()
            .expect("build NavigateToTaskBuilder should success.");
        task.into()
    }

    pub fn reload(&mut self, ignore_cache: bool) {
        let task = self.reload_task(ignore_cache, None);
        self.execute_one_task(task);
    }

    pub fn reload_task(&self, ignore_cache: bool, manual_task_id: Option<TaskId>) -> TaskDescribe {
        let task = page_tasks::PageReloadTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .ignore_cache(ignore_cache)
            .build()
            .expect("build PageReloadTaskBuilder should success.");
        task.into()
    }

    pub fn main_frame(&self) -> Option<&page::Frame> {
        self.changing_frames.values().find_map(|cf| match cf {
            ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm)
                if fm.parent_id.is_none() =>
            {
                Some(fm)
            }
            _ => None,
        })
    }

    pub fn get_response_body_for_interception(
        &mut self,
        interception_id: String,
        request_id: Option<network::RequestId>,
    ) {
        let task = network_tasks::GetResponseBodyForInterceptionTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .interception_id(interception_id)
            .request_id(request_id)
            .build()
            .expect("GetResponseBodyForInterceptionTaskBuilder should work.");

        self.execute_one_task(task.into());
    }

    pub fn continue_intercepted_request_with_raw_response(
        &mut self,
        interception_id: String,
        raw_response: Option<String>,
    ) {
        let task = if let Some(rr) = raw_response {
            network_tasks::ContinueInterceptedRequestTaskBuilder::default()
                .common_fields(self.get_common_field(None))
                .interception_id(interception_id)
                .raw_response(rr)
                .build()
                .expect("ContinueInterceptedRequestTaskBuilder should work.")
        } else {
            error!("intercept got empty body.");
            network_tasks::ContinueInterceptedRequestTaskBuilder::default()
                .common_fields(self.get_common_field(None))
                .error_reason("Failed".to_owned())
                .build()
                .expect("ContinueInterceptedRequestTaskBuilder should work.")
        };
        self.execute_one_task(task.into());
    }

    pub fn node_arrived(&mut self, parent_node_id: dom::NodeId, mut nodes: Vec<dom::Node>) {
        self.temporary_node_holder
            .entry(parent_node_id)
            .or_insert_with(|| vec![])
            .append(&mut nodes);
    }

    pub fn node_returned(&mut self, node: Option<dom::Node>) {
        if let Some(nd) = node {
            if let Some(parent_id) = nd.parent_id {
                self.temporary_node_holder
                    .entry(parent_id)
                    .or_insert_with(|| vec![])
                    .push(nd);
            } else {
                warn!("node_returned has no parent_id. treat as 0.");
                self.temporary_node_holder
                    .entry(0_u16)
                    .or_insert_with(|| vec![])
                    .push(nd);
            }
        } else {
            error!("return None Node.");
        }
    }

    pub fn find_node_by_id(&self, node_id: Option<dom::NodeId>) -> Option<&dom::Node> {
        self.temporary_node_holder
            .values()
            .flatten()
            .find(|nd| Some(nd.node_id) == node_id)
    }

    pub fn find_navigated_frame<F>(&self, mut filter: F) -> Option<&page::Frame>
    where
        F: FnMut(&page::Frame) -> bool,
    {
        self.changing_frames
            .values()
            .filter_map(|cf| match cf {
                ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm) => Some(fm),
                _ => None,
            })
            .find(|frame| filter(frame))
    }

    pub fn find_frame_by_id(&self, frame_id: &str) -> Option<&page::Frame> {
        match self.changing_frames.get(frame_id) {
            Some(ChangingFrame::Navigated(fm)) | Some(ChangingFrame::StoppedLoading(fm)) => {
                Some(fm)
            }
            _ => None,
        }
    }

    pub fn find_execution_context_id_by_frame_name(
        &self,
        frame_name: &'static str,
    ) -> Option<&runtime::ExecutionContextDescription> {
        let frame = self.changing_frames.values().find_map(|cf| match cf {
            ChangingFrame::Navigated(fr) | ChangingFrame::StoppedLoading(fr)
                if fr.name == Some(frame_name.into()) =>
            {
                Some(fr)
            }
            _ => None,
        });
        frame.and_then(|fr| self.execution_context_descriptions.get(&fr.id))
    }

    pub fn verify_execution_context_id(
        &self,
        console_api_called: &embedded_events::ConsoleAPICalledParams,
    ) {
        let ex = self
            .execution_context_descriptions
            .values()
            .find(|v| v.id == console_api_called.execution_context_id);
        if ex.is_none() {
            error!(
                "no execution_context_description found on tab. {:?}",
                console_api_called
            );
        }
    }

    pub fn runtime_execution_context_destroyed(
        &mut self,
        execution_context_id: runtime::ExecutionContextId,
    ) {
        self.execution_context_descriptions
            .retain(|_, v| v.id != execution_context_id);
    }

    pub fn runtime_execution_context_created(
        &mut self,
        execution_context: runtime::ExecutionContextDescription,
    ) -> Option<page::FrameId> {
        self.event_statistics
            .event_happened(EventName::ExecutionContextCreated);
        let aux_data = execution_context.aux_data.clone();
        if let Some(frame_id_str) = aux_data["frameId"].as_str() {
            let frame_id = frame_id_str.to_string();
            let old_value = self
                .execution_context_descriptions
                .insert(frame_id_str.to_string(), execution_context);
            if old_value.is_some() {
                warn!(
                    "execution context already saved, old: {:?}, new: {:?}",
                    old_value,
                    self.execution_context_descriptions.get(&frame_id)
                );
            }
            Some(frame_id)
        } else {
            warn!(
                "execution context has no frameId property. {:?}",
                execution_context
            );
            None
        }
    }

    pub fn _frame_navigated(&mut self, frame: page::Frame) {
        self.event_statistics
            .event_happened(EventName::FrameNavigated);
        if let Some(changing_frame) = self.changing_frames.get_mut(&frame.id) {
            *changing_frame = ChangingFrame::Navigated(frame);
        } else {
            info!(
                "Cannot found frame with id when got _frame_navigated, sometime chrome didn't emit other two events.: {:?}",
                frame,
            );
            self.changing_frames
                .insert(frame.id.clone(), ChangingFrame::Navigated(frame));
        }
    }

    pub fn _frame_started_loading(&mut self, frame_id: String) {
        if let Some(changing_frame) = self.changing_frames.get_mut(&frame_id) {
            *changing_frame = ChangingFrame::StartedLoading(frame_id);
        } else {
            self.changing_frames
                .insert(frame_id.clone(), ChangingFrame::StartedLoading(frame_id));
        }
    }

    pub fn _frame_stopped_loading<T: AsRef<str>>(&mut self, frame_id: T) {
        if let Some(changing_frame) = self.changing_frames.get_mut(frame_id.as_ref()) {
            if let ChangingFrame::Navigated(fm) = changing_frame {
                *changing_frame = ChangingFrame::StoppedLoading(fm.clone());
            } else {
                error!("-----------{:?}", changing_frame);
            }
        } else {
            error!(
                "Cannot found frame with id when got _frame_stopped_loading: {:?}",
                frame_id.as_ref()
            );
            error!("Current changing_frames: {:?}", self.changing_frames);
        }
    }

    pub fn _frame_attached(&mut self, frame_attached_params: page::events::FrameAttachedParams) {
        let frame_id = frame_attached_params.frame_id.clone();
        self.changing_frames.insert(
            frame_id.clone(),
            ChangingFrame::Attached(frame_attached_params),
        );
    }
    pub fn _frame_detached(&mut self, frame_id: &str) {
        self.changing_frames.remove(frame_id);
    }

    pub fn get_document(&mut self, depth: Option<u8>) {
        let task = self.get_document_task(depth);
        self.execute_one_task(task);
    }

    pub fn get_document_task(&mut self, depth: Option<u8>) -> TaskDescribe {
        self.get_document_task_impl(depth, None)
    }

    pub fn get_document_named(&mut self, depth: Option<u8>, name: &str) {
        let task = self.get_document_task_named(depth, name);
        self.execute_one_task(task);
    }

    pub fn get_document_task_named(&mut self, depth: Option<u8>, name: &str) -> TaskDescribe {
        self.get_document_task_impl(depth, Some(name.into()))
    }

    fn get_document_task_impl(
        &mut self,
        depth: Option<u8>,
        manual_task_id: Option<TaskId>,
    ) -> TaskDescribe {
        let task = dom_tasks::GetDocumentTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .depth(depth)
            .build()
            .expect("build GetDocumentTaskBuilder should success.");
        task.into()
    }

    pub fn query_selector_by_selector(&mut self, selector: &str) {
        self.query_selector_by_selector_impl(selector, None);
    }

    pub fn query_selector_by_selector_named(&mut self, selector: &str, name: &str) {
        self.query_selector_by_selector_impl(selector, Some(name.into()));
    }

    fn query_selector_by_selector_impl(&mut self, selector: &str, manual_task_id: Option<TaskId>) {
        let tasks = self.get_query_selector(selector, manual_task_id);
        self.execute_tasks(tasks);
    }

    pub fn describe_node_by_selector(
        &mut self,
        selector: &str,
        depth: Option<i8>,
        manual_task_id: Option<TaskId>,
    ) {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let describe_node = dom_tasks::DescribeNodeTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .selector(selector.to_owned())
            .depth(depth)
            .build()
            .expect("build DescribeNodeTaskBuilder should success.");
        pre_tasks.push(describe_node.into());
        self.execute_tasks(pre_tasks);
    }

    pub fn describe_node_named(
        &mut self,
        describe_node_task_builder: dom_tasks::DescribeNodeTaskBuilder,
        name: &str,
    ) {
        self.describe_node_impl(describe_node_task_builder, Some(name.into()));
    }
    pub fn describe_node(
        &mut self,
        describe_node_task_builder: dom_tasks::DescribeNodeTaskBuilder,
    ) {
        self.describe_node_impl(describe_node_task_builder, None);
    }

    fn describe_node_impl(
        &mut self,
        mut describe_node_task_builder: dom_tasks::DescribeNodeTaskBuilder,
        manual_task_id: Option<TaskId>,
    ) {
        match describe_node_task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build()
        {
            Ok(task) => self.execute_one_task(task.into()),
            Err(err) => error!("build describe_node task error: {:?}", err),
        }
    }

    pub fn query_selector(
        &mut self,
        mut query_selector_task_builder: dom_tasks::QuerySelectorTaskBuilder,
        manual_task_id: Option<TaskId>,
    ) {
        match query_selector_task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build()
        {
            Ok(task) => self.execute_one_task(task.into()),
            Err(err) => error!("build query_selector task error: {:?}", err),
        }
    }

    fn get_query_selector(
        &self,
        selector: &str,
        manual_task_id: Option<TaskId>,
    ) -> Vec<TaskDescribe> {
        let get_document = dom_tasks::GetDocumentTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .build()
            .expect("build GetDocumentTaskBuilder should success.");
        let query_select = dom_tasks::QuerySelectorTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .selector(selector)
            .build()
            .expect("build QuerySelectorTaskBuilder should success.");
        vec![get_document.into(), query_select.into()]
    }

    /// Moves the mouse to this point (dispatches a mouseMoved event)
    pub fn mouse_move_to_point_task(&self, point: Option<Point>) -> TaskDescribe {
        let task = input_tasks::DispatchMouseEventTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .event_type(input_tasks::MouseEventType::Moved)
            .x(point.map(|p| p.x))
            .y(point.map(|p| p.y))
            .build()
            .expect("move_mouse_to_point should build success.");
        task.into()
    }

    pub fn mouse_move_to_xy_task(&self, x: f64, y: f64) -> TaskDescribe {
        self.mouse_move_to_point_task(Some(Point { x, y }))
    }

    pub fn execute_task_after_secs(&mut self, task: TaskDescribe, delay_secs: u64) {
        self.task_queue.add_delayed(task, delay_secs);
    }

    pub fn execute_tasks_after_secs(&mut self, tasks: Vec<TaskDescribe>, delay_secs: u64) {
        self.task_queue.add_delayed_many(tasks, delay_secs);
    }

    pub fn execute_tasks_in_interval(&mut self, tasks: Vec<TaskDescribe>, delay_secs: u64) {
        for (idx, v) in tasks.into_iter().enumerate() {
            self.task_queue
                .add_delayed(v, delay_secs * ((idx + 1) as u64))
        }
    }

    pub fn move_mouse_random_after_secs(&mut self, delay_secs: u64) {
        self.execute_tasks_after_secs(self.move_mouse_random_tasks(), delay_secs);
    }

    pub fn move_mouse_random_tasks(&self) -> Vec<TaskDescribe> {
        vec![
            self.mouse_move_to_xy_task(100.0, 100.0),
            self.mouse_move_to_xy_task(200.0, 200.0),
            self.mouse_move_to_xy_task(300.0, 300.0),
        ]
    }

    pub fn mouse_press_at_point_task(&self, point: Option<Point>) -> TaskDescribe {
        let task = input_tasks::DispatchMouseEventTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .event_type(input_tasks::MouseEventType::Pressed)
            .x(point.map(|p| p.x))
            .y(point.map(|p| p.y))
            .button(input_tasks::MouseButton::Left)
            .click_count(1)
            .build()
            .expect("mouse_press_at_point_task should build success.");
        task.into()
    }
    pub fn mouse_release_at_point(&self, point: Option<Point>) -> TaskDescribe {
        let task = input_tasks::DispatchMouseEventTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .event_type(input_tasks::MouseEventType::Released)
            .x(point.map(|p| p.x))
            .y(point.map(|p| p.y))
            .button(input_tasks::MouseButton::Left)
            .click_count(1)
            .build()
            .expect("mouse_press_at_point_task should build success.");
        task.into()
    }

    pub fn mouse_click_on_remote_object_task(
        &self,
        remote_object_id: runtime::RemoteObjectId,
    ) -> Vec<TaskDescribe> {
        let mut tasks = self.mouse_click_on_point_task(None);
        tasks.insert(
            0,
            self.get_content_quads_by_object_id_task(remote_object_id),
        );
        tasks
    }

    pub fn mouse_click_on_remote_object(&mut self, remote_object_id: runtime::RemoteObjectId) {
        let tasks = self.mouse_click_on_remote_object_task(remote_object_id);
        self.execute_tasks(tasks);
    }

    pub fn mouse_click_on_point_task(&self, point: Option<Point>) -> Vec<TaskDescribe> {
        vec![
            self.mouse_move_to_point_task(point),
            self.mouse_press_at_point_task(point),
            self.mouse_release_at_point(point),
        ]
    }

    pub fn get_content_quads_by_object_id_task_named(
        &self,
        remote_object_id: runtime::RemoteObjectId,
        name: &str,
    ) -> TaskDescribe {
        let mut builder = dom_tasks::GetContentQuadsTaskBuilder::default();
        builder
            .common_fields(self.get_common_field(Some(name.into())))
            .object_id(remote_object_id);
        self.get_content_quads_task(builder)
    }

    pub fn get_content_quads_by_object_id_task(
        &self,
        remote_object_id: runtime::RemoteObjectId,
    ) -> TaskDescribe {
        let mut builder = dom_tasks::GetContentQuadsTaskBuilder::default();
        builder
            .common_fields(self.get_common_field(None))
            .object_id(remote_object_id);
        self.get_content_quads_task(builder)
    }

    pub fn get_content_quads_by_backend_node_id_task(
        &self,
        backend_node_id: dom::NodeId,
    ) -> TaskDescribe {
        let mut builder = dom_tasks::GetContentQuadsTaskBuilder::default();
        builder
            .common_fields(self.get_common_field(None))
            .backend_node_id(backend_node_id);
        self.get_content_quads_task(builder)
    }

    pub fn get_content_quads_task(
        &self,
        mut get_content_quads_task_builder: dom_tasks::GetContentQuadsTaskBuilder,
    ) -> TaskDescribe {
        let task = get_content_quads_task_builder
            .common_fields(self.get_common_field(None))
            .build()
            .expect("GetContentQuadsTaskBuilder should success.");
        task.into()
    }

    pub fn get_layout_metrics(&mut self) {
        self.get_layout_metrics_impl(None);
    }

    fn get_layout_metrics_impl(&mut self, name: Option<&str>) {
        let task = page_tasks::GetLayoutMetricsTaskBuilder::default()
            .common_fields(self.get_common_field(name.map(Into::into)))
            .build()
            .expect("build GetLayoutMetricsTaskBuilder should success.");
        self.execute_one_task(task.into());
    }

    pub fn get_common_field(&self, manual_task_id: Option<TaskId>) -> CommonDescribeFields {
        CommonDescribeFieldsBuilder::default()
            .target_id(self.target_info.target_id.clone())
            .session_id(self.session_id.clone())
            .task_id(manual_task_id)
            .build()
            .expect("build common_fields should success.")
    }

    pub fn set_request_interception_task_named(
        &self,
        name: &str,
    ) -> network_tasks::SetRequestInterceptionTask {
        network_tasks::SetRequestInterceptionTaskBuilder::default()
            .common_fields(self.get_common_field(Some(name.into())))
            .build()
            .expect("SetRequestInterceptionTaskBuilder should work.")
    }

    pub fn execute_one_task(&mut self, task: TaskDescribe) {
        self.chrome_session
            .lock()
            .expect("ob  chrome_session should success.")
            .execute_task(vec![task]);
    }

    pub fn execute_tasks(&mut self, tasks: Vec<TaskDescribe>) {
        self.chrome_session
            .lock()
            .expect("obtain chrome_session should success.")
            .execute_task(tasks);
    }

    pub fn print_to_pdf(
        &mut self,
        manual_task_id: Option<TaskId>,
        task_builder: Option<page_tasks::PrintToPdfTaskBuilder>,
    ) {
        let mut task_builder = if let Some(tb) = task_builder {
            tb
        } else {
            page_tasks::PrintToPdfTaskBuilder::default()
        };
        let task = task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build()
            .expect("build PrintToPdfTaskBuilder should success.");
        self.execute_one_task(task.into());
    }

    pub fn page_enable(&mut self) {
        let task = self.page_enable_task();
        if self.session_id.is_some() {
            self.execute_one_task(task);
        } else {
            self.waiting_for_page_attach_tasks.push(task);
        }
        
    }

    fn page_enable_task(&self) -> TaskDescribe {
        page_tasks::PageEnableTask {
            common_fields: self.get_common_field(None),
        }
        .into()
    }

    pub fn lifecycle_events_enable(&mut self) {
        let task = self.lifecycle_events_enable_task();
        if self.session_id.is_none() {
            self.waiting_for_page_attach_tasks.push(task);
        } else {
            self.execute_one_task(task);
        }
    }

    fn lifecycle_events_enable_task(&self) -> TaskDescribe {
        page_tasks::SetLifecycleEventsEnabledTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .enabled(true)
            .build()
            .expect("SetRequestInterceptionTaskBuilder should success.")
            .into()
    }

    // pub fn runtime_enable_named(&mut self, name: &str) {
    //     self.runtime_enable_impl(Some(name));
    // }

    pub fn runtime_enable(&mut self) {
        let task = self.runtime_enable_task_impl(None);
        if self.session_id.is_none() {
            self.waiting_for_page_attach_tasks.push(task);
        } else {
            self.execute_one_task(task);
        }
        // self.runtime_enable_impl(None);
    }

    pub fn runtime_enable_task(&mut self) -> TaskDescribe {
        self.runtime_enable_task_impl(None)
    }

    fn runtime_enable_task_impl(&mut self, name: Option<&str>) -> TaskDescribe {
        let common_fields = self.get_common_field(name.map(Into::into));
        runtime_tasks::RuntimeEnableTask { common_fields }.into()
    }

    // fn runtime_enable_impl(&mut self, name: Option<&str>) {
    //     let task = self.runtime_enable_task_impl(name);
    //     self.execute_one_task(task);
    // }

    pub fn network_enable(&mut self) {
        let task = self.network_enable_task_impl(None);
        if self.session_id.is_none() {
            self.waiting_for_page_attach_tasks.push(task);
        } else {
            self.execute_one_task(task);
        }
    }

    pub fn network_enable_named(&mut self, name: &str) {
        let task = self.network_enable_task_impl(Some(name));
        self.execute_one_task(task);
    }

    fn network_enable_task_impl(&mut self, manual_task_id: Option<&str>) -> TaskDescribe {
        let common_fields = self.get_common_field(manual_task_id.map(Into::into));
        let nwe = network_tasks::NetworkEnableTaskBuilder::default()
            .common_fields(common_fields)
            .build()
            .expect("NetworkEnableTaskBuilder should work.");
        nwe.into()
    }

    /// let fnd = "function() {return this.getAttribute('src');}";
    pub fn call_function_on_named(
        &mut self,
        call_function_on_task_builder: runtime_tasks::CallFunctionOnTaskBuilder,
        name: &str,
    ) {
        self.call_function_on_impl(call_function_on_task_builder, Some(name));
    }

    pub fn call_function_on(
        &mut self,
        call_function_on_task_builder: runtime_tasks::CallFunctionOnTaskBuilder,
    ) {
        self.call_function_on_impl(call_function_on_task_builder, None);
    }

    pub fn get_js_midpoint_task(
        &self,
        remote_object_id: runtime::RemoteObjectId,
        name: Option<&str>,
    ) -> TaskDescribe {
        self.call_function_on_remote_object_task(
            name,
            remote_object_id,
            "function(){ return this.getBoundingClientRect();}",
            Some(true),
        )
    }
    // pub fn call_js_fn(
    //     &self,
    //     function_declaration: &str,
    //     await_promise: bool,
    // ) -> Result<runtime::RemoteObject, Error> {
    //     let result = self
    //         .parent
    //         .call_method(runtime::methods::CallFunctionOn {
    //             object_id: Some(self.remote_object_id.clone()),
    //             function_declaration,
    //             return_by_value: Some(false),
    //             generate_preview: Some(true),
    //             silent: Some(false),
    //             await_promise: Some(await_promise),
    //             ..Default::default()
    //         })?
    //         .result;
    //     Ok(result)
    // }
    pub fn call_function_on_remote_object_task(
        &self,
        name: Option<&str>,
        remote_object_id: runtime::RemoteObjectId,
        fnd: &str,
        generate_preview: Option<bool>,
    ) -> TaskDescribe {
        self.call_function_on_remote_object_task_impl(name, remote_object_id, fnd, generate_preview)
    }

    fn call_function_on_remote_object_task_impl(
        &self,
        name: Option<&str>,
        remote_object_id: runtime::RemoteObjectId,
        fnd: &str,
        generate_preview: Option<bool>,
    ) -> TaskDescribe {
        let task = runtime_tasks::CallFunctionOnTaskBuilder::default()
            .common_fields(self.get_common_field(name.map(Into::into)))
            .object_id(remote_object_id)
            .function_declaration(fnd)
            .generate_preview(generate_preview)
            .build()
            .expect("CallFunctionOnTaskBuilder should work.");
        task.into()
    }

    fn call_function_on_impl(
        &mut self,
        call_function_on_task_builder: runtime_tasks::CallFunctionOnTaskBuilder,
        name: Option<&str>,
    ) {
        let task = self.call_function_on_task_impl(call_function_on_task_builder, name);
        self.execute_one_task(task);
    }

    fn call_function_on_task_impl(
        &self,
        mut call_function_on_task_builder: runtime_tasks::CallFunctionOnTaskBuilder,
        name: Option<&str>,
    ) -> TaskDescribe {
        let task = call_function_on_task_builder
            .common_fields(self.get_common_field(name.map(Into::into)))
            .build()
            .expect("build call_function_on task error.");
        task.into()
    }

    pub fn name_the_page(&mut self, page_name: &'static str) {
        self.page_name = Some(page_name);
    }

    pub fn name_is(&self, page_name: &str) -> bool {
        self.page_name == Some(page_name)
    }

    pub fn page_attached(&mut self, session_id: target::SessionID) {
        let session_id_cloned = session_id.clone();
        self.session_id.replace(session_id);

        let tasks: Vec<TaskDescribe> = self.waiting_for_page_attach_tasks.drain(..).into_iter()
            .filter_map(|td| if let TaskDescribe::TargetCallMethod(mut task) = td {
                    task.set_session_id(session_id_cloned.clone());
                    Some(task.into())
                } else {
                    None
                }
            )
            .collect();

        // let task_names: Vec<WaitingForPageAttachTaskName> =
        //     self.waiting_for_page_attach.drain().collect();

        // let tasks: Vec<TaskDescribe> = task_names
        //     .iter()
        //     .map(|n| match n {
        //         WaitingForPageAttachTaskName::BringToFront => self.bring_to_front_task(),
        //         WaitingForPageAttachTaskName::PageEnable => self.page_enable_task(),
        //         WaitingForPageAttachTaskName::RuntimeEnable => self.runtime_enable_task(),
        //         WaitingForPageAttachTaskName::SetLifecycleEventsEnabled => {
        //             self.lifecycle_events_enable_task()
        //         }
        //         WaitingForPageAttachTaskName::NetworkEnable => self.network_enable_task_impl(None),
        //     })
        //     .collect();
        self.execute_tasks(tasks);
    }

    pub fn attach_to_page(&mut self) {
        let task = self.attach_to_page_task();
        self.execute_one_task(task);
    }

    // pub fn attach_to_page_and_then(&mut self, tasks: Vec<WaitingForPageAttachTaskName>) {
    //     tasks.into_iter().for_each(|it| {
    //         self.waiting_for_page_attach.insert(it);
    //     });
    //     self.attach_to_page();
    // }

    pub fn attach_to_page_task(&mut self) -> TaskDescribe {
        let task = page_tasks::AttachToTargetTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .build()
            .expect("build AttachToTargetTaskBuilder should success.");
        task.into()
    }
}
