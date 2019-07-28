use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, BrowserCallMethodTask, HasCommonField, HasCallId, };
use crate::protocol::{security};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SecurityEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(SecurityEnableTask, "SecurityEnableTask");

impl AsMethodCallString for SecurityEnableTask  {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = security::methods::Enable {};
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::BrowserCallMethod, BrowserCallMethodTask::SecurityEnable, SecurityEnableTask);