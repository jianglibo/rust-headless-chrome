use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{page};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CaptureScreenshotTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub selector: Option<&'static str>,
    pub format: page::ScreenshotFormat,
    #[builder(default = "None")]
    pub clip: Option<page::Viewport>,
    #[builder(default = "None")]
    pub from_surface: Option<bool>,
    #[builder(default = "None")]
    pub task_result: Option<String>,
}

impl_has_common_fields!(CaptureScreenshotTask);

impl AsMethodCallString for CaptureScreenshotTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
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
            from_surface: self.from_surface.map_or(true, |o|o),
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::CaptureScreenshot, CaptureScreenshotTask);