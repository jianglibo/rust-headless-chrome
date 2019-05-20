use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString,};
use crate::protocol::{runtime, target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(RuntimeEnableTask);

impl AsMethodCallString for RuntimeEnableTask {
    fn get_method_str(&self) ->  Result<String, failure::Error>{
        let method = runtime::methods::Enable {};
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::RuntimeEnable, RuntimeEnableTask);