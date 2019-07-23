use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTask, AsMethodCallString, HasCommonField, CanCreateMethodString,};
use crate::protocol::{dom, runtime};
use crate::browser::tab::point::Point;
use crate::browser::tab::element::{ElementQuad};
use failure;


#[derive(Debug, Builder, Default, Clone)]
#[builder(setter(into))]
pub struct GetContentQuadsTask {
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
    pub task_result: Option<Vec<[f64; 8]>>,
}

impl_has_common_fields!(GetContentQuadsTask);

impl AsMethodCallString for GetContentQuadsTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = dom::methods::GetContentQuads {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            object_id: self.object_id.as_ref().map(String::as_str)
        };
        Ok(self.create_method_str(method))
    }
}

impl GetContentQuadsTask {
    pub fn get_midpoint(&self) -> Option<Point> {
        if let Some(quads) = &self.task_result {
            let input_quad = ElementQuad::from_raw_points(quads.first().expect("empty quads array."));
            Some((input_quad.bottom_right + input_quad.top_left) / 2.0)
        } else {
            None
        }
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetContentQuads, GetContentQuadsTask);