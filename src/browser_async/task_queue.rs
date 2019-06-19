use std::time::{Instant};
use super::task_describe::TaskDescribe;

#[derive(Debug)]
pub struct TaskItem {
    pub issued_at: Instant,
    pub delay_secs: Option<u64>,
    pub task: TaskDescribe,
}

#[derive(Debug)]
pub struct TaskQueue {
    task_items: Vec<TaskItem>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            task_items: Vec::new(),
        }
    }

    pub fn add_manually(&mut self, task: TaskDescribe) {
        self.task_items.push(TaskItem{
            issued_at: Instant::now(),
            delay_secs: None,
            task,
        });        
    }

    pub fn add_delayed(&mut self, task: TaskDescribe, delay_secs: u64) {
        self.task_items.push(TaskItem{
            issued_at: Instant::now(),
            delay_secs: Some(delay_secs),
            task,
        });
    }

    pub fn retrieve_delayed_task_to_run(&mut self) -> Vec<TaskDescribe> {
        // self.task_items.drain_filter(|ti| ti.issued_at.elapsed().as_secs() > ti.delay_secs).map(|ti|ti.task).collect()
        let mut to_run = Vec::<TaskItem>::new();
        self.task_items = self.task_items.drain(..).filter_map(|ti| {
            if ti.delay_secs.is_some() && &ti.issued_at.elapsed().as_secs() > ti.delay_secs.as_ref().unwrap() {
                to_run.push(ti);
                None
            } else {
                Some(ti)
            }
        }).collect();
        to_run.into_iter().map(|it|it.task).collect()
    }

    pub fn retrieve_manually_task_to_run(&mut self) -> Option<TaskDescribe> {
        if let Some(i) = self.task_items.iter().position(|it| it.delay_secs.is_none()) {
            Some(self.task_items.remove(i).task)
        } else {
            None
        }
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;


    #[test]
    fn test_task_queue() {
        assert!(true);
    }
}
