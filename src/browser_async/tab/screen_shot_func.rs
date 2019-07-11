use super::Tab;
use super::super::protocol::{page, runtime, target, input};
use super::{TaskId, ChromeDebugSession};
use super::super::task_describe::{
    network_tasks, page_tasks, runtime_tasks, target_tasks,
    CommonDescribeFields, CommonDescribeFieldsBuilder, TaskDescribe, input_tasks,
};

impl Tab {
    fn capture_screenshot_by_selector_task_impl(
        &mut self,
        selector: &'static str,
        format: page::ScreenshotFormat,
        from_surface: Option<bool>,
        name: Option<&str>,
    ) -> Vec<TaskDescribe> {
        let screen_shot = page_tasks::CaptureScreenshotTaskBuilder::default()
            .common_fields(self.get_common_field(name.map(Into::into)))
            .selector(selector)
            .format(format)
            .from_surface(from_surface)
            .build()
            .expect("build CaptureScreenshotTaskBuilder should success.");
        let mut pre_tasks = self.get_box_model_by_selector_task(selector);
        pre_tasks.push(screen_shot.into());
        pre_tasks
    }

    pub fn capture_screenshot_by_selector_jpeg_task(&mut self, selector: &'static str, quality: Option<u8>, from_surface: Option<bool>, task_name: Option<&str>) -> Vec<TaskDescribe> {
        self.capture_screenshot_by_selector_task_impl(selector, page::ScreenshotFormat::JPEG(quality), from_surface, task_name)
    }

    pub fn capture_screenshot_by_selector_png_task(&mut self, selector: &'static str, from_surface: Option<bool>, task_name: Option<&str>) -> Vec<TaskDescribe> {
        self.capture_screenshot_by_selector_task_impl(selector, page::ScreenshotFormat::PNG, from_surface, task_name)
    }

    pub fn capture_screenshot_view_jpeg(&mut self, quality: Option<u8>) {
        let task = self.capture_screenshot_impl_task(page::ScreenshotFormat::JPEG(quality),
         Some(false), None);
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_surface_jpeg(&mut self, quality: Option<u8>) {
        let task = self.capture_screenshot_impl_task(page::ScreenshotFormat::JPEG(quality),
         Some(true), None);
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_view_png(
        &mut self
    ) {
        let task = self.capture_screenshot_impl_task(page::ScreenshotFormat::PNG,
         Some(false), None);
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_surface_png(
        &mut self
    ) {
        let task = self.capture_screenshot_impl_task(page::ScreenshotFormat::PNG,
         Some(true), None);
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_jpeg_task(
        &mut self,
        quality: Option<u8>,
        from_surface: Option<bool>
    ) -> TaskDescribe {
        self.capture_screenshot_impl_task(page::ScreenshotFormat::JPEG(quality), from_surface, None)
    }

    pub fn capture_screenshot_png_task(
        &mut self,
        from_surface: Option<bool>
    ) -> TaskDescribe {
        self.capture_screenshot_impl_task(page::ScreenshotFormat::PNG, from_surface, None)
    }

    pub fn capture_screenshot_task_named(
        &mut self,
        format: page::ScreenshotFormat,
        from_surface: Option<bool>,
        name: &str
    ) -> TaskDescribe {
        self.capture_screenshot_impl_task(format, from_surface, Some(name.into()))
    }

    fn capture_screenshot_impl_task(
        &mut self,
        format: page::ScreenshotFormat,
        from_surface: Option<bool>,
        manual_task_id: Option<TaskId>,
    ) -> TaskDescribe {
        let screen_shot = page_tasks::CaptureScreenshotTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .format(format)
            .from_surface(from_surface)
            .build()
            .expect("build CaptureScreenshotTaskBuilder should success.");
        screen_shot.into()
    }
}