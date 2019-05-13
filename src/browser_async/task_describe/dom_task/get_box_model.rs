use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{dom, runtime};
use crate::browser::transport::{SessionId};
use crate::browser::tab::element::BoxModel;


#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct GetBoxModelTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    pub selector: Option<String>,
    #[builder(default = "None")]
    pub backend_node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    pub object_id: Option<runtime::types::RemoteObjectId>,
    #[builder(setter(skip))]
    pub task_result: Option<BoxModel>,
}

impl From<GetBoxModelTask> for TaskDescribe {
    fn from(get_box_model: GetBoxModelTask) -> Self {
        TaskDescribe::GetBoxModel(Box::new(get_box_model))
    }
}

impl CreateMethodCallString for GetBoxModelTask {
    fn create_method_call_string(&self, session_id: Option<&SessionId>, call_id: usize) -> String {
        let method = dom::methods::GetBoxModel {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            object_id: self.object_id.as_ref().map(String::as_str)
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}