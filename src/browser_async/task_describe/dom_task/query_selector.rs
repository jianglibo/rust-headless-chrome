use super::super::{TaskDescribe, CommonDescribeFields,TargetCallMethodTask, AsMethodCallString, HasCommonField, CanCreateMethodString,};
use crate::protocol::{dom, target};
use crate::browser_async::{page_message::PageResponse};
use failure;

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
    fn get_method_str(&self) -> Result<String, failure::Error> {
        failure::ensure!(self.node_id.is_some(), "node_id is a must for QuerySelectorTask.");
        let method = dom::methods::QuerySelector {
            node_id: self.node_id.unwrap(),
            selector: self.selector.as_str(),
        };
        Ok(self.create_method_str(method))
    }
}

impl QuerySelectorTask {
    pub fn into_page_response(self) -> PageResponse {
        PageResponse::QuerySelectorDone(self.selector.to_string(), self.task_result)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::QuerySelector, QuerySelectorTask);