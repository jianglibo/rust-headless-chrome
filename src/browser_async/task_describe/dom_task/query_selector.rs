use super::super::{TaskDescribe, CommonDescribeFields,TargetCallMethodTask, AsMethodCallString, HasCommonField, CanCreateMethodString,};
use crate::protocol::{dom, target};

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct QuerySelectorTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub node_id: Option<dom::NodeId>,
    pub selector: String,
    #[builder(default = "None")]
    pub task_result: Option<dom::NodeId>,
}

impl_has_common_fields!(QuerySelectorTask);

impl AsMethodCallString for QuerySelectorTask {
    fn get_method_str(&self) -> String {
        let method = dom::methods::QuerySelector {
            node_id: self.node_id.unwrap(),
            selector: self.selector.as_str(),
        };
        self.create_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::QuerySelector, QuerySelectorTask);