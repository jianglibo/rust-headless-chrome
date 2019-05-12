use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{page};
use crate::browser::transport::{SessionId};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct PageEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl From<PageEnableTask> for TaskDescribe {
    fn from(task: PageEnableTask) -> Self {
        TaskDescribe::PageEnable(task)
    }
}

impl CreateMethodCallString for PageEnableTask {
    fn create_method_call_string(&self, session_id: Option<&SessionId>, call_id: usize) -> String {
        let method = page::methods::Enable {};
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}