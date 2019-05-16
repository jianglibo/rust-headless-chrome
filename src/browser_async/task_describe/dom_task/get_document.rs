use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{dom, target};

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct GetDocumentTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "Some(0)")]
    pub depth: Option<u8>,
    #[builder(default = "false")]
    pub pierce: bool,
    #[builder(setter(skip))]
    pub task_result: Option<dom::Node>,
}

impl From<GetDocumentTask> for TaskDescribe {
    fn from(get_document: GetDocumentTask) -> Self {
        TaskDescribe::GetDocument(Box::new(get_document))
    }
}

impl CreateMethodCallString for GetDocumentTask {
    fn create_method_call_string(&self, session_id: Option<&target::SessionID>, call_id: usize) -> String {
        let method = dom::methods::GetDocument {
            depth: self.depth.or(Some(0)),
            pierce: Some(self.pierce),
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}