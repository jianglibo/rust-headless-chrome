use super::super::super::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, BrowserCallMethodTask,  HasCommonField, HasCallId, };
use super::super::super::super::protocol::target;
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct GetTargetsTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub task_result: Option<Vec<target::TargetInfo>>,
}

impl_has_common_fields!(GetTargetsTask, "GetTargetsTask");

impl AsMethodCallString for GetTargetsTask {
    fn get_method_str(&self) ->  Result<String, failure::Error> {
        let method = target::methods::GetTargets {};
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::BrowserCallMethod, BrowserCallMethodTask::GetTargets, GetTargetsTask);