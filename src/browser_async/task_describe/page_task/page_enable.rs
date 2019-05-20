use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{page, target};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct PageEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(PageEnableTask);

impl AsMethodCallString for PageEnableTask {
    fn get_method_str(&self) -> String {
        let method = page::methods::Enable{};
        self.create_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::PageEnable, PageEnableTask);

// {\"method\":\"Target.sendMessageToTarget\",\"id\":3,\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.enable\\\",\\\"id\\\":2,\\\"params\\\":{}}\"}}
// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"id\\\":2,\\\"result\\\":{}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}