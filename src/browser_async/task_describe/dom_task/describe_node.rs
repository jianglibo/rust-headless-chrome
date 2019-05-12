use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{dom, runtime};
use crate::browser::transport::{SessionId};

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct DescribeNodeTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    pub backend_node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    pub found_node: Option<dom::Node>,
    #[builder(default = "None")]
    pub selector: Option<&'static str>,
    #[builder(default = "Some(0)")]
    pub depth: Option<i8>,
    #[builder(default = "None")]
    pub object_id: Option<runtime::types::RemoteObjectId>,
    #[builder(default = "false")]
    pub pierce: bool,
}

impl From<DescribeNodeTask> for TaskDescribe {
    fn from(describe_node: DescribeNodeTask) -> Self {
        TaskDescribe::DescribeNode(Box::new(describe_node))
    }
}


impl CreateMethodCallString for DescribeNodeTask {
    fn create_method_call_string(&self, session_id: Option<&SessionId>, call_id: usize) -> String {
        let method = dom::methods::DescribeNode {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            depth: self.depth,
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}