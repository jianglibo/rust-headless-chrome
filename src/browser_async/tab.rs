use super::chrome_debug_session::ChromeDebugSession;
use super::page_message::ChangingFrame;
use super::task_describe::{self as tasks, network_events, TaskDescribe, RuntimeEnableTask, NetworkEnableTaskBuilder, SetRequestInterceptionTask, SetRequestInterceptionTaskBuilder, GetResponseBodyForInterceptionTaskBuilder, ContinueInterceptedRequestTaskBuilder};
use super::super::browser_async::{MethodDestination, TaskId, create_msg_to_send, next_call_id, embedded_events, create_unique_prefixed_id};
use crate::protocol::{self, dom, page, runtime, target, network};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::*;

#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub target_info: protocol::target::TargetInfo,
    pub session_id: Option<target::SessionID>,
    pub root_node: Option<dom::Node>,
    pub page_name: Option<&'static str>,
    pub changing_frames: HashMap<page::FrameId, ChangingFrame>,
    pub temporary_node_holder: HashMap<dom::NodeId, Vec<dom::Node>>,
    pub execution_context_descriptions:
        HashMap<page::FrameId, runtime::ExecutionContextDescription>,
    pub ongoing_request: HashMap<network::RequestId, network_events::RequestWillBeSent>,
    pub load_event_fired_count: u16,
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
            ongoing_request: HashMap::new(),
            load_event_fired_count: 0,
        }
    }

    pub fn is_blank_url(&self) -> bool {
        self.is_at_url("about:blank")
    }

    pub fn is_chrome_error_chromewebdata(&self) -> bool {
        self.is_at_url("chrome-error://chromewebdata/")
    }

    pub fn request_will_be_sent(&mut self, event: network_events::RequestWillBeSent) {
        self.ongoing_request.insert(event.get_request_id(), event);
    }

    pub fn take_request(&mut self, request_id: &network::RequestId) -> network_events::RequestWillBeSent {
        self.ongoing_request.remove(request_id).expect("cannot find the request by request_id!")
    }

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

    pub fn get_url<'a>(&'a self) -> &'a str {
        if let Some(mf) = self.main_frame() {
            &mf.url
        } else {
            &self.target_info.url
        }
    }

    pub fn navigate_to_named(&mut self, url: &'static str, name: &str) {
        let task = self.navigate_to_task(url, Some(name.to_owned()));
        self.execute_one_task(task);
    }

    pub fn navigate_to(&mut self, url: &'static str) {
        let task = self.navigate_to_task(url, None);
        self.execute_one_task(task);
    }

    pub fn navigate_to_task(&self, url: &'static str, manual_task_id: Option<TaskId>) -> TaskDescribe {
        let task = tasks::NavigateToTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .url(url)
            .build()
            .unwrap();
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

    pub fn get_response_body_for_interception(&mut self, interception_id: String) {
        let task = GetResponseBodyForInterceptionTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .interception_id(interception_id)
            .build()
            .expect("GetResponseBodyForInterceptionTaskBuilder should work.");

        self.execute_one_task(task.into());
    }

    pub fn continue_intercepted_request_with_raw_response(&mut self, interception_id: String, raw_response: Option<String>) {
        let task = if let Some(rr) = raw_response {
            ContinueInterceptedRequestTaskBuilder::default()
        .common_fields(self.get_common_field(None))
        .interception_id(interception_id)
        .raw_response(rr)
        .build()
        .expect("ContinueInterceptedRequestTaskBuilder should work.")
        } else {
            error!("intercept got empty body.");
            ContinueInterceptedRequestTaskBuilder::default()
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

    pub fn find_frame_by_id(&self, frame_id: &page::FrameId) -> Option<&page::Frame> {
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
            trace!(
                "Cannot found frame with id when got _frame_started_loading, no it shouldn't.: {:?}",
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
    pub fn _frame_detached(&mut self, frame_id: &page::FrameId) {
        self.changing_frames.remove(frame_id);
    }

    pub fn get_document(&mut self, depth: Option<u8>) {
        self.get_document_impl(depth, None);
    }

    pub fn get_document_named(&mut self, depth: Option<u8>, name: &str) {
        self.get_document_impl(depth, Some(name.into()));
    }

    fn get_document_impl(&mut self, depth: Option<u8>, manual_task_id: Option<TaskId>) {
        let task = tasks::GetDocumentTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .depth(depth)
            .build()
            .unwrap();
        self.execute_one_task(task.into());
    }

    pub fn query_selector_by_selector(&mut self, selector: &str) {
        self.query_selector_by_selector_impl(selector, None);
    }

    pub fn query_selector_by_selector_named(&mut self, selector: &str, name: &str) {
        self.query_selector_by_selector_impl(selector, Some(name.into()));
    }

    fn query_selector_by_selector_impl(
        &mut self,
        selector: &str,
        manual_task_id: Option<TaskId>,
    ) {
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
        let describe_node = tasks::DescribeNodeTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .selector(selector.to_owned())
            .depth(depth)
            .build()
            .unwrap();
        pre_tasks.push(describe_node.into());
        self.execute_tasks(pre_tasks);
    }

    pub fn describe_node(
        &mut self,
        mut describe_node_task_builder: tasks::DescribeNodeTaskBuilder,
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
        mut query_selector_task_builder: tasks::QuerySelectorTaskBuilder,
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
        let get_document = tasks::GetDocumentTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .build()
            .unwrap();
        let query_select = tasks::QuerySelectorTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .selector(selector)
            .build()
            .unwrap();
        vec![get_document.into(), query_select.into()]
    }

    fn get_box_model(
        &self,
        selector: &'static str,
        manual_task_id: Option<TaskId>,
    ) -> Vec<TaskDescribe> {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let get_box_model = tasks::GetBoxModelTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .selector(selector.to_owned())
            .build()
            .unwrap();
        pre_tasks.push(get_box_model.into());
        pre_tasks
    }

    pub fn get_box_model_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<TaskId>,
    ) {
        let tasks = self.get_box_model(selector, manual_task_id);
        self.execute_tasks(tasks);
    }
    pub fn capture_screenshot_by_selector(
        &mut self,
        selector: &'static str,
        format: page::ScreenshotFormat,
        from_surface: bool,
        manual_task_id: Option<TaskId>,
    ) {
        let screen_shot = tasks::CaptureScreenshotTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .selector(selector)
            .format(format)
            .from_surface(from_surface)
            .build()
            .unwrap();
        let mut pre_tasks = self.get_box_model(selector, None);
        pre_tasks.push(screen_shot.into());
        self.execute_tasks(pre_tasks);
    }

    pub fn get_common_field(&self, manual_task_id: Option<TaskId>) -> tasks::CommonDescribeFields {
        tasks::CommonDescribeFieldsBuilder::default()
            .target_id(self.target_info.target_id.clone())
            .session_id(self.session_id.clone())
            .task_id(manual_task_id)
            .build()
            .unwrap()
    }

    pub fn set_request_interception_task_named(&self, name: &str) -> SetRequestInterceptionTask {
        SetRequestInterceptionTaskBuilder::default().common_fields(self.get_common_field(Some(name.into()))).build().expect("SetRequestInterceptionTaskBuilder should work.")
    }

    pub fn execute_one_task(&mut self, task: TaskDescribe) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![task]);
    }

    pub fn execute_tasks(&mut self, tasks: Vec<TaskDescribe>) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(tasks);
    }

    pub fn print_to_pdf(
        &mut self,
        manual_task_id: Option<TaskId>,
        task_builder: Option<tasks::PrintToPdfTaskBuilder>,
    ) {
        let mut task_builder = if let Some(tb) = task_builder {
            tb
        } else {
            tasks::PrintToPdfTaskBuilder::default()
        };
        let task = task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build()
            .unwrap();
        self.execute_one_task(task.into());
    }

    pub fn page_enable(&mut self) {
        let task = self.page_enable_task();
        self.execute_one_task(task);
    }

    pub fn page_enable_task(&self) -> TaskDescribe {
        tasks::PageEnableTask {
            common_fields: self.get_common_field(None),
        }.into()
    }

    pub fn runtime_enable(&mut self, manual_task_id: Option<TaskId>) {
        let common_fields = self.get_common_field(manual_task_id);
        let task = RuntimeEnableTask{common_fields};
        self.execute_one_task(task.into());
    }

    pub fn network_enable(&mut self, manual_task_id: Option<TaskId>) {
        let task = self.network_enable_task(manual_task_id);
        self.execute_one_task(task.into());
    }

    pub fn network_enable_task(&mut self, manual_task_id: Option<TaskId>) -> TaskDescribe {
        let common_fields = self.get_common_field(manual_task_id);
        let nwe = NetworkEnableTaskBuilder::default().common_fields(common_fields).build().expect("NetworkEnableTaskBuilder should work.");
        nwe.into()
    }

    pub fn evaluate_expression(&mut self, expression: &str) {
        self.runtime_evaluate_expression_impl(expression, None);
    }

    pub fn evaluate_expression_prefixed(&mut self, expression: &str, prefix: &str) {
        let name = create_unique_prefixed_id(prefix);
        self.evaluate_expression_named(expression, name.as_str());
    }

    pub fn evaluate_expression_named(&mut self, expression: &str, name: &str) {
        self.runtime_evaluate_expression_impl(expression, Some(name.to_owned()));
    }

    fn runtime_evaluate_expression_impl(
        &mut self,
        expression: &str,
        manual_task_id: Option<TaskId>,
    ) {
        let task = tasks::RuntimeEvaluateTaskBuilder::default()
            .expression(expression.to_string())
            .common_fields(self.get_common_field(manual_task_id))
            .build()
            .unwrap();
        self.execute_one_task(task.into());
    }

    pub fn runtime_evaluate(
        &mut self,
        mut evaluate_task_builder: tasks::RuntimeEvaluateTaskBuilder,
        manual_task_id: Option<TaskId>,
    ) {
        let task = evaluate_task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build();
        match task {
            Ok(task) => self.execute_one_task(task.into()),
            Err(err) => error!("build evaluate task error: {:?}", err),
        }
    }

    pub fn runtime_call_function_on(
        &mut self,
        mut call_function_on_task_builder: tasks::RuntimeCallFunctionOnTaskBuilder,
        manual_task_id: Option<TaskId>,
    ) {
        let task = call_function_on_task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build();
        match task {
            Ok(task) => self.execute_one_task(task.into()),
            Err(err) => error!("build call_function_on task error: {:?}", err),
        }
    }

    pub fn runtime_get_properties_by_object_id(
        &mut self,
        object_id: runtime::RemoteObjectId,
        manual_task_id: Option<TaskId>,
    ) {
        let task = tasks::RuntimeGetPropertiesTaskBuilder::default()
            .object_id(object_id)
            .common_fields(self.get_common_field(manual_task_id))
            .build()
            .unwrap();
        self.execute_one_task(task.into());
    }

    pub fn runtime_get_properties(
        &mut self,
        mut get_properties_task_builder: tasks::RuntimeGetPropertiesTaskBuilder,
        manual_task_id: Option<TaskId>,
    ) {
        let task = get_properties_task_builder
            .common_fields(self.get_common_field(manual_task_id))
            .build();
        match task {
            Ok(task) => self.execute_one_task(task.into()),
            Err(err) => error!("build get_properties_task_builder error: {:?}", err),
        }
    }

    pub fn name_the_page(&mut self, page_name: &'static str) {
        self.page_name = Some(page_name);
    }

    pub fn attach_to_page(&mut self) {
        let method_str = create_msg_to_send(
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
