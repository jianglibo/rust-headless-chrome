use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace, TargetCallMethodTask};
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

impl TargetCallMethodTaskFace for CaptureScreenshotTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
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
        self._to_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::CaptureScreenshot, CaptureScreenshotTask);