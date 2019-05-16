use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{page, target};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CaptureScreenshotTask {
    pub common_fields: CommonDescribeFields,
    pub selector: Option<&'static str>,
    pub format: page::ScreenshotFormat,
    #[builder(default = "None")]
    pub clip: Option<page::Viewport>,
    #[builder(default = "false")]
    pub from_surface: bool,
    #[builder(default = "None")]
    pub task_result: Option<String>,
}

impl From<CaptureScreenshotTask> for TaskDescribe {
    fn from(capture_screenshot: CaptureScreenshotTask) -> Self {
        TaskDescribe::CaptureScreenshot(Box::new(capture_screenshot))
    }
}

impl CreateMethodCallString for CaptureScreenshotTask {
    fn create_method_call_string(&self, session_id: Option<&target::SessionID>, call_id: usize) -> String {
                let (format, quality) = match self.format {
            page::ScreenshotFormat::JPEG(quality) => {
                (page::InternalScreenshotFormat::JPEG, quality)
            }
            page::ScreenshotFormat::PNG => {
                (page::InternalScreenshotFormat::PNG, None)
            }
        };

        let method = page::methods::CaptureScreenshot {
            clip: self.clip.as_ref().cloned(),
            format,
            quality,
            from_surface: self.from_surface,
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
}
}