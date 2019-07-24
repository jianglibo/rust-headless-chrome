use super::super::protocol::page;
use super::super::task_describe::{page_tasks, TaskDescribe};
use super::Tab;
use super::TaskId;

impl Tab {
    fn capture_screenshot_by_selector_task_impl(
        &mut self,
        selector: &'static str,
        format: page::ScreenshotFormat,
        from_surface: Option<bool>,
        save_to_file: Option<&'static str>,
        name: Option<&str>,
    ) -> Vec<TaskDescribe> {
        let screen_shot = page_tasks::CaptureScreenshotTaskBuilder::default()
            .common_fields(self.get_common_field(name.map(Into::into)))
            .selector(selector)
            .format(format)
            .from_surface(from_surface)
            .save_to_file(save_to_file)
            .build()
            .expect("build CaptureScreenshotTaskBuilder should success.");
        let mut pre_tasks = self.get_box_model_by_selector_task(selector);
        pre_tasks.push(screen_shot.into());
        pre_tasks
    }

    pub fn capture_screenshot_by_selector_jpeg_task(
        &mut self,
        selector: &'static str,
        quality: Option<u8>,
        from_surface: Option<bool>,
        save_to_file: Option<&'static str>,
        task_name: Option<&str>,
    ) -> Vec<TaskDescribe> {
        self.capture_screenshot_by_selector_task_impl(
            selector,
            page::ScreenshotFormat::JPEG(quality),
            from_surface,
            save_to_file,
            task_name,
        )
    }

    pub fn capture_screenshot_by_selector_png_task(
        &mut self,
        selector: &'static str,
        from_surface: Option<bool>,
        save_to_file: Option<&'static str>,
        task_name: Option<&str>,
    ) -> Vec<TaskDescribe> {
        self.capture_screenshot_by_selector_task_impl(
            selector,
            page::ScreenshotFormat::PNG,
            from_surface,
            save_to_file,
            task_name,
        )
    }

    pub fn capture_screenshot_view_jpeg(
        &mut self,
        quality: Option<u8>,
        save_to_file: Option<&'static str>,
    ) {
        let task = self.capture_screenshot_impl_task(
            page::ScreenshotFormat::JPEG(quality),
            Some(false),
            None,
            save_to_file,
        );
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_surface_jpeg(
        &mut self,
        quality: Option<u8>,
        save_to_file: Option<&'static str>,
    ) {
        let task = self.capture_screenshot_impl_task(
            page::ScreenshotFormat::JPEG(quality),
            Some(true),
            None,
            save_to_file,
        );
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_view_png(&mut self, save_to_file: Option<&'static str>) {
        let task = self.capture_screenshot_impl_task(
            page::ScreenshotFormat::PNG,
            Some(false),
            None,
            save_to_file,
        );
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_surface_png(&mut self, save_to_file: Option<&'static str>) {
        let task = self.capture_screenshot_impl_task(
            page::ScreenshotFormat::PNG,
            Some(true),
            None,
            save_to_file,
        );
        self.execute_one_task(task);
    }

    pub fn capture_screenshot_jpeg_task(
        &mut self,
        quality: Option<u8>,
        from_surface: Option<bool>,
        save_to_file: Option<&'static str>,
    ) -> TaskDescribe {
        self.capture_screenshot_impl_task(
            page::ScreenshotFormat::JPEG(quality),
            from_surface,
            None,
            save_to_file,
        )
    }

    pub fn capture_screenshot_png_task(
        &mut self,
        from_surface: Option<bool>,
        save_to_file: Option<&'static str>,
    ) -> TaskDescribe {
        self.capture_screenshot_impl_task(
            page::ScreenshotFormat::PNG,
            from_surface,
            None,
            save_to_file,
        )
    }

    pub fn capture_screenshot_task_named(
        &mut self,
        format: page::ScreenshotFormat,
        from_surface: Option<bool>,
        save_to_file: Option<&'static str>,
        name: &str,
    ) -> TaskDescribe {
        self.capture_screenshot_impl_task(format, from_surface, Some(name.into()), save_to_file)
    }

    fn capture_screenshot_impl_task(
        &mut self,
        format: page::ScreenshotFormat,
        from_surface: Option<bool>,
        manual_task_id: Option<TaskId>,
        save_to_file: Option<&'static str>,
    ) -> TaskDescribe {
        let screen_shot = page_tasks::CaptureScreenshotTaskBuilder::default()
            .common_fields(self.get_common_field(manual_task_id))
            .format(format)
            .from_surface(from_surface)
            .save_to_file(save_to_file)
            .build()
            .expect("build CaptureScreenshotTaskBuilder should success.");
        screen_shot.into()
    }
}
