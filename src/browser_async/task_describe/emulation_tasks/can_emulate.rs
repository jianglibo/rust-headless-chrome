use super::super::{
    CommonDescribeFields, TaskDescribe, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString, 
};
use crate::protocol::{emulation};
use failure;

#[derive(Debug, Builder, Default, Clone)]
#[builder(setter(into))]
pub struct CanEmulateTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub task_result: Option<bool>,
}

impl_has_common_fields!(CanEmulateTask, "CanEmulateTask");

impl AsMethodCallString for CanEmulateTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = emulation::methods::CanEmulate {};
        Ok(self.create_method_str(method))
    }
}


impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::CanEmulate, CanEmulateTask);

