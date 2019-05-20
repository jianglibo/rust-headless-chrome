use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask, HasCommonField, HasCallId};
use crate::protocol::target;
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SetDiscoverTargetsTask {
    pub common_fields: CommonDescribeFields,
    pub discover: bool,
}

impl_has_common_fields!(SetDiscoverTargetsTask);

impl AsMethodCallString for SetDiscoverTargetsTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = target::methods::SetDiscoverTargets { discover: self.discover };
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::TargetSetDiscoverTargets, SetDiscoverTargetsTask);