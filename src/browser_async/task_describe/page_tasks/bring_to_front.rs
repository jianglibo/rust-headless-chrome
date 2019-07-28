use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{page};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct BringToFrontTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(BringToFrontTask, "BringToFrontTask");

impl AsMethodCallString for BringToFrontTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = page::methods::BringToFront{};
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::BringToFront, BringToFrontTask);
