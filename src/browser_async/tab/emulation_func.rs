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
            .device_scale_factor(0)
            .mobile(false)
            .build()
            .expect("SetDeviceMetricsOverrideTaskBuilder should success.");
        task.into()
    }
}
