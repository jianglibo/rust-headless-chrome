use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{network};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct NetworkEnableTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub max_total_buffer_size: Option<u32>,
    #[builder(default = "None")]
    pub max_resource_buffer_size: Option<u32>,
    #[builder(default = "None")]
    pub max_post_data_size: Option<u32>,
}

impl_has_common_fields!(NetworkEnableTask, "NetworkEnableTask");

impl AsMethodCallString for NetworkEnableTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = network::methods::Enable{
            max_total_buffer_size: self.max_total_buffer_size,
            max_resource_buffer_size: self.max_resource_buffer_size,
            max_post_data_size: self.max_post_data_size,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::NetworkEnable, NetworkEnableTask);
