use super::chrome_debug_session::ChromeDebugSession;
use super::dev_tools_method_util::{next_call_id, MethodDestination, MethodUtil, SessionId};
use super::id_type as ids;
use super::inner_event::{inner_events};
use super::page_message::ChangingFrame;
use super::task_describe::{self as tasks, TaskDescribe};
use crate::protocol::{self, dom, page, runtime, target};
use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub target_info: protocol::target::TargetInfo,
    pub session_id: Option<SessionId>,
    pub root_node: Option<dom::Node>,
    pub changing_frames: HashMap<String, ChangingFrame>,
    pub temporary_node_holder: HashMap<dom::NodeId, Vec<dom::Node>>,
    pub execution_context_descriptions:
        HashMap<page::types::FrameId, runtime::types::ExecutionContextDescription>,
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
            changing_frames: HashMap::new(),
            temporary_node_holder: HashMap::new(),
            execution_context_descriptions: HashMap::new(),
        }
    }
    pub fn navigate_to(&mut self, url: &'static str, manual_task_id: Option<ids::Task>) {
        let task = tasks::NavigateToBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .url(url)
            .build()
            .unwrap();
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);
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

    pub fn find_frame_by_id<T: AsRef<str>>(&self, frame_id: T) -> Option<&page::Frame> {
        match self.changing_frames.get(frame_id.as_ref()) {
            Some(ChangingFrame::Navigated(fm)) | Some(ChangingFrame::StoppedLoading(fm)) => {
                Some(fm)
            }
            _ => None,
        }
    }

    pub fn verify_execution_context_id(&self, console_api_called: &inner_events::ConsoleAPICalledParams) {
        let ex = self.execution_context_descriptions.values().find(|v|v.id == console_api_called.execution_context_id);
        if ex.is_none() {
            error!("no execution_context_description found on tab. {:?}", console_api_called);
        }
    }

    pub fn runtime_execution_context_destroyed(&mut self, execution_context_id: runtime::types::ExecutionContextId) {
        self.execution_context_descriptions.retain(|_, v| v.id != execution_context_id);
    }

    pub fn runtime_execution_context_created(
        &mut self,
        execution_context: runtime::types::ExecutionContextDescription,
    ) -> Option<page::types::FrameId> {
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
        if let Some(changing_frame) = self.changing_frames.get_mut(&frame.id) {
            *changing_frame = ChangingFrame::Navigated(frame);
        } else {
            error!(
                "Cannot found frame with id when got _frame_navigated: {:?}",
                frame
            );
            error!("Current changing_frames: {:?}", self.changing_frames);
        }
    }

    pub fn _frame_started_loading(&mut self, frame_id: String) {
        if let Some(changing_frame) = self.changing_frames.get_mut(&frame_id) {
            *changing_frame = ChangingFrame::StartedLoading(frame_id);
        } else {
            info!(
                "Cannot found frame with id when got _frame_attached: {:?}",
                &frame_id
            );
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

    pub fn get_document(&mut self, depth: Option<u8>, manual_task_id: Option<ids::Task>) {
        let task = tasks::GetDocumentBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .depth(depth)
            .build()
            .unwrap();
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);
    }

    pub fn dom_query_selector_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<usize>,
    ) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(self.get_query_selector(selector, manual_task_id));
    }

    pub fn describe_node_by_selector(
        &mut self,
        selector: &'static str,
        depth: Option<i8>,
        manual_task_id: Option<ids::Task>,
    ) {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let describe_node = tasks::DescribeNodeBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .depth(depth)
            .build()
            .unwrap();
        pre_tasks.push(describe_node.into());
        self.chrome_session.lock().unwrap().execute_task(pre_tasks);
    }

    pub fn describe_node(
        &mut self,
        mut describe_node_task_builder: tasks::DescribeNodeBuilder,
        manual_task_id: Option<ids::Task>,
    ) {
        match describe_node_task_builder.common_fields(self.get_c_f(manual_task_id)).build() {
            Ok(task) => self.chrome_session.lock().unwrap().execute_task(vec![task.into()]),
            Err(err) => error!("build describe_node task error: {:?}", err),
        }
    }

    pub fn query_selector(
        &mut self,
        mut query_selector_task_builder: tasks::QuerySelectorBuilder,
        manual_task_id: Option<ids::Task>,
    ) {
        match query_selector_task_builder.common_fields(self.get_c_f(manual_task_id)).build() {
            Ok(task) => self.chrome_session.lock().unwrap().execute_task(vec![task.into()]),
            Err(err) => error!("build query_selector task error: {:?}", err),
        }
    }

    fn get_query_selector(
        &self,
        selector: &'static str,
        manual_task_id: Option<ids::Task>,
    ) -> Vec<TaskDescribe> {
        let get_document = tasks::GetDocumentBuilder::default()
            .common_fields(self.get_c_f(None))
            .build()
            .unwrap();
        let query_select = tasks::QuerySelectorBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .build()
            .unwrap();
        vec![get_document.into(), query_select.into()]
    }

    fn get_box_model(
        &self,
        selector: &'static str,
        manual_task_id: Option<ids::Task>,
    ) -> Vec<TaskDescribe> {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let get_box_model = tasks::GetBoxModelBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .build()
            .unwrap();
        pre_tasks.push(get_box_model.into());
        pre_tasks
    }

    pub fn get_box_model_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<ids::Task>,
    ) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(self.get_box_model(selector, manual_task_id));
    }
    pub fn capture_screenshot_by_selector(
        &mut self,
        selector: &'static str,
        format: page::ScreenshotFormat,
        from_surface: bool,
        manual_task_id: Option<ids::Task>,
    ) {
        let screen_shot = tasks::ScreenShotBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .format(format)
            .from_surface(from_surface)
            .build()
            .unwrap();
        let mut pre_tasks = self.get_box_model(selector, None);
        pre_tasks.push(screen_shot.into());
        self.chrome_session.lock().unwrap().execute_task(pre_tasks);
    }

    fn get_c_f(&self, manual_task_id: Option<ids::Task>) -> tasks::CommonDescribeFields {
        tasks::CommonDescribeFieldsBuilder::default()
            .target_id(self.target_info.target_id.clone())
            .session_id(self.session_id.clone())
            .task_id(manual_task_id)
            .build()
            .unwrap()
    }

    pub fn page_enable(&mut self) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![TaskDescribe::PageEnable(self.get_c_f(None))]);
    }

    pub fn runtime_enable(&mut self, manual_task_id: Option<ids::Task>) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![TaskDescribe::RuntimeEnable(
                self.get_c_f(manual_task_id),
            )]);
    }

    pub fn runtime_evaluate_expression(&mut self, expression: String, manual_task_id: Option<ids::Task>) {
        let rt = tasks::RuntimeEvaluateBuilder::default()
            .expression(expression)
            .common_fields(self.get_c_f(manual_task_id))
            .build()
            .unwrap();
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![rt.into()]);
    }

    pub fn attach_to_page(&mut self) {
        let method_str = MethodUtil::create_msg_to_send(
            target::methods::AttachToTarget {
                target_id: &(self.target_info.target_id),
                flatten: None,
            },
            MethodDestination::Browser,
            next_call_id(),
        );
        self.chrome_session
            .lock()
            .unwrap()
            .send_message_direct(method_str);
    }
}
