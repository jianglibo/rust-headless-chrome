use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, BrowserCallMethodTask, HasCommonField, HasCallId};
use crate::protocol::{target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct AttachToTargetTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(AttachToTargetTask);

impl AsMethodCallString for AttachToTargetTask  {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = target::methods::AttachToTarget {
                target_id: self.common_fields.target_id.as_ref().expect("target_id should exists in CommonDescribeFields."),
                flatten: None,
            };
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::BrowserCallMethod, BrowserCallMethodTask::AttachedToTarget, AttachToTargetTask);