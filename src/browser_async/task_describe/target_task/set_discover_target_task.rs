use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace, TargetCallMethodTask};
use crate::protocol::target;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SetDiscoverTargetsTask {
    pub common_fields: CommonDescribeFields,
    pub discover: bool,
}

impl TargetCallMethodTaskFace for SetDiscoverTargetsTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = target::methods::SetDiscoverTargets { discover: self.discover };
        create_msg_to_send(method, MethodDestination::Browser, self.get_call_id())
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::TargetSetDiscoverTargets, SetDiscoverTargetsTask);