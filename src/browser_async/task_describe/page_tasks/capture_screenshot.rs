use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use super::super::protocol::{page};
use super::super::super::page_message::{write_base64_str_to};
use std::fs;
use std::path::Path;
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CaptureScreenshotTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub selector: Option<&'static str>,
    #[builder(default = "None")]
    pub save_to_file: Option<&'static str>,
    pub format: page::ScreenshotFormat,
    #[builder(default = "None")]
    pub clip: Option<page::Viewport>,
    #[builder(default = "None")]
    pub from_surface: Option<bool>,
    #[builder(default = "None")]
    pub task_result: Option<String>,
}

impl_has_common_fields!(CaptureScreenshotTask);

impl CaptureScreenshotTask {
    pub fn save(&self) -> Result<&str, failure::Error> {
        if let Some(file_name) = self.save_to_file {
            if let Some(b64) = self.task_result.as_ref() {
                let path = Path::new(file_name);
                if path.exists() && path.is_file() {
                    fs::remove_file(file_name).unwrap();
                }
                write_base64_str_to(file_name, Some(b64))?;
                Ok(file_name)
            } else {
                failure::bail!("there is no result base64 string to write to.")
            }
        } else {
            failure::bail!("only if save_to_file exist can i save to a file.")
        }
    }
}

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