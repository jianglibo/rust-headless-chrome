use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{dom};
use crate::browser::transport::{SessionId};

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct QuerySelectorTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    pub found_node_id: Option<dom::NodeId>,
    pub selector: String,
}

impl From<QuerySelectorTask> for TaskDescribe {
    fn from(query_selector: QuerySelectorTask) -> Self {
        TaskDescribe::QuerySelector(query_selector)
    }
}

impl CreateMethodCallString for QuerySelectorTask {
    fn create_method_call_string(&self, session_id: Option<&SessionId>, call_id: usize) -> String {
        let method = dom::methods::QuerySelector {
            node_id: self.node_id.unwrap(),
            selector: self.selector.as_str(),
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}