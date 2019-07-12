use super::super::{
    CommonDescribeFields, TaskDescribe, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString,
};
use crate::protocol::{emulation};
use serde::{Deserialize, Serialize};
use failure;

#[derive(Debug, Builder, Default, Deserialize, Serialize)]
#[builder(setter(into))]
#[serde(rename_all = "camelCase")]
pub struct CanEmulateTask {
    #[serde(skip)]
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    #[serde(skip)]
    pub task_result: Option<bool>,
}

impl_has_common_fields!(CanEmulateTask);

impl AsMethodCallString for CanEmulateTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = emulation::methods::CanEmulate {};
        Ok(self.create_method_str(method))
    }
}


impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::CanEmulate, CanEmulateTask);

