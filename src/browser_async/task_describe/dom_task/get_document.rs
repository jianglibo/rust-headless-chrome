use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace};
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

// impl From<GetDocumentTask> for TaskDescribe {
//     fn from(get_document: GetDocumentTask) -> Self {
//         TaskDescribe::GetDocument(Box::new(get_document))
//     }
// }

impl TargetCallMethodTaskFace for GetDocumentTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = dom::methods::GetDocument {
            depth: self.depth.or(Some(0)),
            pierce: Some(self.pierce),
        };
        self._to_method_str(method)
    }
}
