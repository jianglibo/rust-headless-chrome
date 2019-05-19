use super::super::{TaskDescribe, CommonDescribeFields,TargetCallMethodTask, TargetCallMethodTaskFace};
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

impl TargetCallMethodTaskFace for QuerySelectorTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = dom::methods::QuerySelector {
            node_id: self.node_id.unwrap(),
            selector: self.selector.as_str(),
        };
        self._to_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::QuerySelector, QuerySelectorTask);