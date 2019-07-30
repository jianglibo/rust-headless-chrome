use super::super::{
    AsMethodCallString, BrowserCallMethodTask, CommonDescribeFields, HasCallId, HasCommonField,
    TaskDescribe,
};
use crate::browser_async::{create_msg_to_send, MethodDestination};
use crate::protocol::browser;
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct GetBrowserCommandLineTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub task_result: Option<browser::methods::GetBrowserCommandLineReturnObject>,
}

impl_has_common_fields!(GetBrowserCommandLineTask, "GetBrowserCommandLineTask");

impl AsMethodCallString for GetBrowserCommandLineTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = browser::methods::GetBrowserCommandLine {};
        Ok(create_msg_to_send(
            method,
            MethodDestination::Browser,
            self.get_call_id(),
        ))
    }
}

impl_into_task_describe!(
    TaskDescribe::BrowserCallMethod,
    BrowserCallMethodTask::GetBrowserCommandLine,
    GetBrowserCommandLineTask
);
