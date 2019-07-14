use super::super::protocol::CallId;
use super::task_describe::{dom_tasks, input_tasks, HasCallId, TargetCallMethodTask, TaskDescribe};
use log::*;

#[derive(Debug)]
pub struct TaskGroup {
    waiting_tasks: Vec<TaskDescribe>,
    completed_tasks: Vec<TaskDescribe>,
}

impl std::default::Default for TaskGroup {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl TaskGroup {
    pub fn new(waiting_tasks: Vec<TaskDescribe>) -> Self {
        Self {
            waiting_tasks,
            completed_tasks: Vec::new(),
        }
    }

    pub fn contains_call_id(&self, call_id: CallId) -> bool {
        if let Some(task) = self.waiting_tasks.get(0) {
            match task {
                TaskDescribe::TargetCallMethod(target_call) => target_call.get_call_id() == call_id,
                TaskDescribe::BrowserCallMethod(browser_call) => {
                    browser_call.get_call_id() == call_id
                }
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn get_first_task(&mut self) -> TaskDescribe {
        self.waiting_tasks.remove(0)
    }

    pub fn get_first_task_ref(&self) -> &TaskDescribe {
        self.waiting_tasks
            .get(0)
            .expect("execute_next_and_return_remains got empty tasks.")
    }

    pub fn get_first_task_mut(&mut self) -> &mut TaskDescribe {
        self.waiting_tasks
            .get_mut(0)
            .expect("handle_next_task received empty tasks.")
    }

    pub fn get_last_task(&mut self) -> Option<TaskDescribe> {
        self.waiting_tasks.pop()
    }

    pub fn get_last_task_or_current(&mut self, current_task: TaskDescribe) -> TaskDescribe {
        if let Some(task) = self.get_last_task() {
            task
        } else {
            current_task
        }
    }

    pub fn is_empty(&self) -> bool {
        self.waiting_tasks.is_empty()
    }

    pub fn push_completed_task(&mut self, task_describe: TaskDescribe) {
        self.completed_tasks.push(task_describe);
    }

    pub fn find_get_document_task(&self) -> Option<&dom_tasks::GetDocumentTask> {
        self.completed_tasks.iter().find_map(|task| match task {
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetDocument(get_document)) => {
                Some(get_document)
            }
            _ => None,
        })
    }

    pub fn find_query_selector_task(&self) -> Option<&dom_tasks::QuerySelectorTask> {
        self.completed_tasks.iter().find_map(|task| match task {
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::QuerySelector(query_selector)) => {
                Some(query_selector)
            }
            _ => None,
        })
    }

    pub fn find_get_box_model_task(&self, request_full_page: bool) -> Option<&dom_tasks::GetBoxModelTask> {
        self.completed_tasks.iter().rev().find_map(|task| match task {
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetBoxModel(get_box_model)) => {
                if get_box_model.request_full_page == request_full_page {
                    Some(get_box_model)
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    pub fn find_get_content_quads_task(&self) -> Option<&dom_tasks::GetContentQuadsTask> {
        self.completed_tasks.iter().find_map(|task| match task {
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetContentQuads(
                get_content_quads,
            )) => Some(get_content_quads),
            _ => None,
        })
    }

    pub fn find_dispatch_mouse_event_task(&self) -> Option<&input_tasks::DispatchMouseEventTask> {
        self.completed_tasks
            .iter()
            .rev()
            .find_map(|task| match task {
                TaskDescribe::TargetCallMethod(TargetCallMethodTask::DispatchMouseEvent(
                    dispatch_mouse_event,
                )) => Some(dispatch_mouse_event),
                _ => None,
            })
    }

    pub fn full_fill_next_task(&mut self) {
        let start_len = self.waiting_tasks.len();
        let next_task = self.get_first_task();
        match next_task {
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::QuerySelector(
                mut query_selector,
            )) => {
                if query_selector.node_id.is_none() {
                    if let Some(node_id) = self
                        .find_get_document_task()
                        .and_then(|task| task.task_result.as_ref().and_then(|v| Some(v.node_id)))
                    {
                        query_selector.node_id.replace(node_id);
                    } else {
                        error!("cannot find node_id from get_document!");
                    }
                }
                self.waiting_tasks.insert(0, query_selector.into());
            }
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::DescribeNode(
                mut describe_node,
            )) => {
                if describe_node.node_id.is_none() {
                    if let Some(node_id) = self
                        .find_query_selector_task()
                        .and_then(|task| task.task_result)
                    {
                        describe_node.node_id.replace(node_id);
                    } else {
                        error!("cannot find node_id from query_selector!");
                    }
                    self.waiting_tasks.insert(0, describe_node.into());
                }
            }
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::GetBoxModel(
                mut get_box_model,
            )) => {
                if get_box_model.node_id.is_none() {
                    if let Some(node_id) = self
                        .find_query_selector_task()
                        .and_then(|task| task.task_result)
                    {
                        get_box_model.node_id.replace(node_id);
                    } else {
                        error!("cannot find node_id from query_selector!");
                    }
                    self.waiting_tasks.insert(0, get_box_model.into());
                }
            }
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::CaptureScreenshot(
                mut screen_shot,
            )) => {
                if let Some(mb) = self
                    .find_get_box_model_task(false)
                    .and_then(|v| v.task_result.as_ref())
                {
                    let viewport = mb.content_viewport();
                    screen_shot.clip = Some(viewport);
                } else {
                    error!("found_box is None!");
                }
                self.waiting_tasks.insert(0, screen_shot.into());
            }
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::DispatchMouseEvent(
                mut dispatch_mouse_event,
            )) => {
                // if the previous task is dispatch_mouse_event too.
                if let Some(TaskDescribe::TargetCallMethod(
                    TargetCallMethodTask::DispatchMouseEvent(dispatch_mouse_event_1),
                )) = self.completed_tasks.last()
                {
                    if dispatch_mouse_event.x.is_none() && dispatch_mouse_event.y.is_none() {
                        if dispatch_mouse_event_1.x.is_some() && dispatch_mouse_event_1.y.is_some()
                        {
                            dispatch_mouse_event.x.replace(
                                dispatch_mouse_event_1
                                    .x
                                    .expect("dispatch_mouse_event_1 missing x."),
                            );
                            dispatch_mouse_event.y.replace(
                                dispatch_mouse_event_1
                                    .y
                                    .expect("dispatch_mouse_event_1 missing y."),
                            );
                        } else {
                            warn!("dispatch_mouse_event_2 has part point. missing x or y");
                        }
                    } else {
                        warn!("dispatch_mouse_event_1 has part point. missing x or y.");
                    }
                } else if let Some(mid_point) = self
                    .find_get_content_quads_task()
                    .and_then(dom_tasks::GetContentQuadsTask::get_midpoint)
                {
                    dispatch_mouse_event.x.replace(mid_point.x);
                    dispatch_mouse_event.y.replace(mid_point.y);
                } else {
                    warn!("get_content_quads return empty result.");
                }
                self.waiting_tasks.insert(0, dispatch_mouse_event.into());
            }
            TaskDescribe::TargetCallMethod(TargetCallMethodTask::SetDeviceMetricsOverride(mut task)) => {
                if let Some(mb) = self
                    .find_get_box_model_task(true)
                    .and_then(|v| v.task_result.as_ref())
                {
                    let viewport = mb.border_viewport();
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    let (width, height) = (viewport.width as u64, viewport.height as u64);
                    task.width.replace(width);
                    task.height.replace(height);
                } else {
                    error!("found_box is None!");
                }
                self.waiting_tasks.insert(0, task.into());
            }
            task_describe => {
                self.waiting_tasks.insert(0, task_describe);
                info!("skipped full_fill_next_task.");
            }
        }
        let end_len = self.waiting_tasks.len();
        assert_eq!(
            start_len, end_len,
            "waiting_tasks should keep unchanged: {}, {}",
            start_len, end_len
        );
    }
}

#[derive(Debug)]
pub struct TaskManager {
    tasks_waiting_for_response: Vec<TaskGroup>,
}

impl std::default::Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks_waiting_for_response: Vec::new(),
        }
    }

    pub fn tasks_count(&self) -> usize {
        self.tasks_waiting_for_response.len()
    }

    pub fn find_task_vec_by_call_id(&self, call_id: usize) -> Option<usize> {
        self.tasks_waiting_for_response
            .iter()
            .position(|task_group| task_group.contains_call_id(call_id))
    }

    pub fn remove_task_vec(&mut self, idx: usize) -> TaskGroup {
        self.tasks_waiting_for_response.remove(idx)
    }

    pub fn push_task_vec(&mut self, task_vec: Vec<TaskDescribe>) {
        let tg = TaskGroup::new(task_vec);
        self.tasks_waiting_for_response.push(tg);
    }

    pub fn push_task_group(&mut self, task_group: TaskGroup) {
        self.tasks_waiting_for_response.push(task_group);
    }
}
