use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, HasCallId, BrowserCallMethodTask, HasCommonField,};
use crate::protocol::{target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CloseTargetTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub task_result: Option<bool>,
}

impl_has_common_fields!(CloseTargetTask);

impl AsMethodCallString for CloseTargetTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let target_id = self.common_fields.target_id.as_ref().expect("target_id should exists. close_target");
        let method = target::methods::CloseTarget {
            target_id,
        };
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::BrowserCallMethod, BrowserCallMethodTask::CloseTarget, CloseTargetTask);