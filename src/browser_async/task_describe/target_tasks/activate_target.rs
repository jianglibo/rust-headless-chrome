use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, BrowserCallMethodTask,  HasCommonField, HasCallId, };
use crate::protocol::target;
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct ActivateTargetTask {
    pub common_fields: CommonDescribeFields,
    pub target_id: target::TargetId,
    #[builder(default = "None")]
    pub task_result: Option<bool>,
}

impl_has_common_fields!(ActivateTargetTask, "ActivateTargetTask");

impl AsMethodCallString for ActivateTargetTask {
    fn get_method_str(&self) ->  Result<String, failure::Error> {
        let target_id = self.target_id.clone();
        let method = target::methods::ActivateTarget {
            target_id,
        };
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::BrowserCallMethod, BrowserCallMethodTask::ActivateTarget, ActivateTargetTask);