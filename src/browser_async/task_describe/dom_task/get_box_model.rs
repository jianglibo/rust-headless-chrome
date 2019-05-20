use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTask, AsMethodCallString, create_msg_to_send_with_session_id, HasCommonField, CanCreateMethodString,};
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

impl_has_common_fields!(GetBoxModelTask);

impl AsMethodCallString for GetBoxModelTask {
    fn get_method_str(&self) -> String {
        let method = dom::methods::GetBoxModel {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            object_id: self.object_id.as_ref().map(String::as_str)
        };
        self.create_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetBoxModel, GetBoxModelTask);