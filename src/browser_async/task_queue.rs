use std::time::{Instant};
use super::task_describe::TaskDescribe;
use log::*;

#[derive(Debug)]
pub struct TaskQueueItem {
    pub issued_at: Instant,
    pub delay_secs: Option<u64>,
    pub tasks: Vec<TaskDescribe>,
}

impl TaskQueueItem {
    pub fn new_delayed(delay_secs: u64, tasks: Vec<TaskDescribe>) -> Self {
        Self {
            issued_at: Instant::now(),
            delay_secs: Some(delay_secs),
            tasks,
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn new_mannuly(tasks: Vec<TaskDescribe>) -> Self {
        Self {
            issued_at: Instant::now(),
            delay_secs: None,
            tasks,
        }
    }

    pub fn is_mannuly(&self) -> bool {
        self.delay_secs.is_none()
    }

    pub fn is_time_out(&self) -> bool {
        if let Some(delay_secs) = self.delay_secs {
            self.issued_at.elapsed().as_secs() > delay_secs
        } else {
            false
        }
    }
}

/// The taskQueue keeps task group as when they were put in.
#[derive(Debug)]
pub struct TaskQueue {
    task_items: Vec<TaskQueueItem>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            task_items: Vec::new(),
        }
    }

    pub fn vec_len(&self) -> usize {
        self.task_items.len()
    }

    pub fn item_len(&self) -> usize {
        self.task_items.iter().map(TaskQueueItem::len).sum()
    }

    pub fn add_manually_many(&mut self, tasks: Vec<TaskDescribe>) {
        self.task_items.push(TaskQueueItem::new_mannuly(tasks));
    }

    pub fn add_manually(&mut self, task: TaskDescribe) {
        self.add_manually_many(vec![task]);
    }

    pub fn add_delayed(&mut self, task: TaskDescribe, delay_secs: u64) {
        self.add_delayed_many(vec![task], delay_secs);
    }

    pub fn add_delayed_many(&mut self, tasks: Vec<TaskDescribe>, delay_secs: u64) {
        self.task_items.push(TaskQueueItem::new_delayed(delay_secs, tasks));
    }

    pub fn retrieve_delayed_task_to_run(&mut self) -> Vec<Vec<TaskDescribe>> {
        // self.task_items.drain_filter(|ti| ti.issued_at.elapsed().as_secs() > ti.delay_secs).map(|ti|ti.task).collect()
        let mut to_run = Vec::<TaskQueueItem>::new();
        self.task_items = self.task_items.drain(..).filter_map(|ti| {
            if ti.is_time_out() {
                to_run.push(ti);
                None
            } else {
                Some(ti)
            }
        }).collect();
        if !to_run.is_empty() {
            trace!("got delayed to run: {:?}", to_run);
        }
        to_run.into_iter().map(|it|it.tasks).collect()
    }

    pub fn retrieve_manually_task_to_run(&mut self) -> Option<Vec<TaskDescribe>> {
        if let Some(i) = self.task_items.iter().position(TaskQueueItem::is_mannuly) {
            Some(self.task_items.remove(i).tasks)
        } else {
            None
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use log::*;


//     #[test]
//     fn test_task_queue() {
//         assert!(true);
//     }
// }
