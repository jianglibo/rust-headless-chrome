use std::time::{Instant};
use super::task_describe::TaskDescribe;

#[derive(Debug)]
pub struct TaskItem {
    pub issued_at: Instant,
    pub delay_secs: u64,
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

    pub fn add(&mut self, task: TaskDescribe, delay_secs: u64) {
        self.task_items.push(TaskItem{
            issued_at: Instant::now(),
            delay_secs,
            task,
        });
    }

    pub fn retrieve_task_to_run(&mut self) -> Vec<TaskDescribe> {
        // self.task_items.drain_filter(|ti| ti.issued_at.elapsed().as_secs() > ti.delay_secs).map(|ti|ti.task).collect()
        let mut i = 0;
        let mut out_dated: Vec<TaskDescribe> = Vec::new();
        while i != self.task_items.len() {
            let ti = &self.task_items[i];
            if ti.issued_at.elapsed().as_secs() > ti.delay_secs {
               let val = self.task_items.remove(i);
               out_dated.push(val.task);
            } else {
               i += 1;
            }
        }
        out_dated
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
