use super::task_describe::{HasCallId, TaskDescribe};

#[derive(Debug)]
pub struct TaskManager {
    tasks_waiting_for_response: Vec<Vec<TaskDescribe>>,
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
        self.tasks_waiting_for_response.iter().position(|tasks| {
            if let Some(task) = tasks.get(0) {
                match task {
                    TaskDescribe::TargetCallMethod(target_call) => {
                        target_call.get_call_id() == call_id
                    }
                    TaskDescribe::BrowserCallMethod(browser_call) => {
                        browser_call.get_call_id() == call_id
                    }
                    _ => false,
                }
            } else {
                false
            }
        })
    }

    pub fn remove_task_vec(&mut self, idx: usize) -> Vec<TaskDescribe> {
        self.tasks_waiting_for_response.remove(idx)
    }

    pub fn push_task_vec(&mut self, task_vec: Vec<TaskDescribe>) {
        self.tasks_waiting_for_response.push(task_vec);
    }
}
