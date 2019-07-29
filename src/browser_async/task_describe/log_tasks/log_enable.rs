use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{chrome_log};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct LogEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(LogEnableTask, "LogEnableTask");

impl AsMethodCallString for LogEnableTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = chrome_log::methods::Enable{};
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::LogEnable, LogEnableTask);
