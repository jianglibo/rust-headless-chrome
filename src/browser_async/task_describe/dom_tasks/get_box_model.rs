use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTask, AsMethodCallString, HasCommonField, CanCreateMethodString, };
use super::super::super::protocol::{dom, runtime};
use crate::browser::tab::element::BoxModel;
use failure;


#[derive(Debug, Builder, Default, Clone)]
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
    pub object_id: Option<runtime::RemoteObjectId>,
    #[builder(setter(skip))]
    pub task_result: Option<BoxModel>,
    #[builder(default = "false")]
    pub request_full_page: bool,
}

impl_has_common_fields!(GetBoxModelTask, "GetBoxModelTask");

impl AsMethodCallString for GetBoxModelTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = dom::methods::GetBoxModel {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            object_id: self.object_id.as_ref().map(String::as_str)
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetBoxModel, GetBoxModelTask);