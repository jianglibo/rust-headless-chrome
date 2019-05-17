use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace, create_msg_to_send_with_session_id};
use crate::protocol::{dom, runtime, target};
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

// impl From<GetBoxModelTask> for TaskDescribe {
//     fn from(get_box_model: GetBoxModelTask) -> Self {
//         TaskDescribe::GetBoxModel(Box::new(get_box_model))
//     }
// }
impl TargetCallMethodTaskFace for GetBoxModelTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = dom::methods::GetBoxModel {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            object_id: self.object_id.as_ref().map(String::as_str)
        };
        self._to_method_str(method)
    }
}
