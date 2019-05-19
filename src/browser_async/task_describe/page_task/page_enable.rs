use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace, TargetCallMethodTask};
use crate::protocol::{page, target};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct PageEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl TargetCallMethodTaskFace for PageEnableTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = page::methods::Enable{};
        self._to_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::PageEnable, PageEnableTask);

// {\"method\":\"Target.sendMessageToTarget\",\"id\":3,\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.enable\\\",\\\"id\\\":2,\\\"params\\\":{}}\"}}
// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"id\\\":2,\\\"result\\\":{}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}