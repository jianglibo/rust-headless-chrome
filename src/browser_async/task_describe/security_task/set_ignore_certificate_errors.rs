use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask, HasCommonField, HasCallId};
use crate::protocol::{security};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SetIgnoreCertificateErrorsTask {
    pub common_fields: CommonDescribeFields,
    pub ignore: bool,
}

impl_has_common_fields!(SetIgnoreCertificateErrorsTask);

impl AsMethodCallString for SetIgnoreCertificateErrorsTask  {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = security::methods::SetIgnoreCertificateErrors {
            ignore: self.ignore,
        };
        Ok(create_msg_to_send(method, MethodDestination::Browser, self.get_call_id()))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::SetIgnoreCertificateErrors, SetIgnoreCertificateErrorsTask);