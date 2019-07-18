use super::super::task_describe::{emulation_tasks, TaskDescribe};
use super::Tab;

impl Tab {
    pub fn can_emulate(&mut self) {
        let task = emulation_tasks::CanEmulateTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .build()
            .expect("CanEmulateTaskBuilder should success.");
        self.execute_one_task(task.into());
    }
    pub fn set_device_metrics_override_simple(&mut self, width: u64, height: u64) {
        let task = self.set_device_metrics_override_simple_task(width, height);
        self.execute_one_task(task);
    }

    pub fn set_device_metrics_override_simple_task(&mut self, width: u64, height: u64) -> TaskDescribe {
        let task = emulation_tasks::SetDeviceMetricsOverrideTaskBuilder::default()
            .common_fields(self.get_common_field(None))
            .width(width)
            .height(height)
            .build()
            .expect("SetDeviceMetricsOverrideTaskBuilder should success.");
        task.into()
    }

    pub fn display_full_page(&mut self) {
        let tasks = self.display_full_page_task();
        self.execute_tasks(tasks);
    }
    /// waiting page to get loaded so can get it's size.
    pub fn display_full_page_after_secs(&mut self, delay_secs: u64) {
        let tasks = self.display_full_page_task();
        self.execute_tasks_after_secs(tasks, delay_secs);
    }

    pub fn display_full_page_task(&mut self) -> Vec<TaskDescribe> {
        if let Some(bm) = self.box_model.as_ref() {
            let wh = bm.border_viewport().u64_width_height();
            let task = emulation_tasks::SetDeviceMetricsOverrideTaskBuilder::default()
                .common_fields(self.get_common_field(None))
                .width(wh.0)
                .height(wh.1)
                .build()
                .expect("SetDeviceMetricsOverrideTaskBuilder should success.");
            vec![task.into()]
        } else {
            let mut pre_tasks = self.get_body_box_model_task();
            let task = emulation_tasks::SetDeviceMetricsOverrideTaskBuilder::default()
                .common_fields(self.get_common_field(None))
                .build()
                .expect("SetDeviceMetricsOverrideTaskBuilder should success.");
            pre_tasks.push(task.into());
            pre_tasks
        }
    }
}
