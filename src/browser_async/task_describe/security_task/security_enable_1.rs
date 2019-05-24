use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{page, target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SecurityEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(SecurityEnableTask);

impl AsMethodCallString for SecurityEnableTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = page::methods::Enable{};
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::SecurityEnable, SecurityEnableTask);

// {\"method\":\"Target.sendMessageToTarget\",\"id\":3,\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.enable\\\",\\\"id\\\":2,\\\"params\\\":{}}\"}}
// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"id\\\":2,\\\"result\\\":{}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}