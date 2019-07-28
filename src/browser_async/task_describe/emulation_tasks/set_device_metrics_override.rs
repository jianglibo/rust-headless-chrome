use super::super::{
    CommonDescribeFields, TaskDescribe, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString, 
};
use crate::protocol::{emulation, page};
use failure;

#[derive(Debug, Builder, Default, Clone)]
#[builder(setter(into))]
pub struct SetDeviceMetricsOverrideTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub width: Option<u64>,
    #[builder(default = "None")]
    pub height: Option<u64>,
    #[builder(default = "0.0")]
    pub device_scale_factor: f64, // 0 disables the override
    #[builder(default = "false")]
    pub mobile: bool,
    #[builder(default = "None")]
    pub scale: Option<f64>,
    #[builder(default = "None")]
    pub screen_width: Option<u64>,
    #[builder(default = "None")]
    pub screen_height: Option<u64>,
    #[builder(default = "None")]
    pub position_x: Option<u64>,
    #[builder(default = "None")]
    pub position_y: Option<u64>,
    #[builder(default = "None")]
    pub dont_set_visible_size: Option<bool>,
    #[builder(default = "None")]
    pub screen_orientation: Option<emulation::ScreenOrientation>,
    #[builder(default = "None")]
    pub viewport: Option<page::Viewport>,
    #[builder(default = "None")]
    pub task_result: Option<bool>,
}

impl_has_common_fields!(SetDeviceMetricsOverrideTask, "SetDeviceMetricsOverrideTask");

impl AsMethodCallString for SetDeviceMetricsOverrideTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = emulation::methods::SetDeviceMetricsOverride {
            width: self.width.expect("set_device_metrics_override width parameter is mandatory."),
            height: self.height.expect("set_device_metrics_override height parameter is mandatory."),
            device_scale_factor: self.device_scale_factor,
            mobile: self.mobile,
            scale: self.scale,
            screen_width: self.screen_width,
            screen_height: self.screen_height,
            position_x: self.position_x,
            position_y: self.position_y,
            dont_set_visible_size: self.dont_set_visible_size,
            screen_orientation: self.screen_orientation.as_ref().cloned(),
            viewport: self.viewport.as_ref().cloned(),
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::SetDeviceMetricsOverride, SetDeviceMetricsOverrideTask);

