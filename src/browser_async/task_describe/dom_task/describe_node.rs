use super::super::{
    create_msg_to_send_with_session_id, CommonDescribeFields, CreateMethodCallString, TaskDescribe,
};
use crate::browser::transport::SessionId;
use crate::protocol::{dom, runtime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Builder, Default, Deserialize, Serialize)]
#[builder(setter(into))]
#[serde(rename_all = "camelCase")]
pub struct DescribeNodeTask {
    #[serde(skip)]
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    pub backend_node_id: Option<dom::NodeId>,
    #[builder(default = "None")]
    #[serde(skip)]
    pub found_node: Option<dom::Node>,
    #[builder(default = "None")]
    pub selector: Option<String>,
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
        create_msg_to_send_with_session_id(method, session_id, call_id)
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::CommonDescribeFieldsBuilder;
    use super::*;
    use log::*;

    #[test]
    fn test_serde() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();
        {
            let common_fields = CommonDescribeFieldsBuilder::default().build().unwrap();
            let task = DescribeNodeTaskBuilder::default()
                .common_fields(common_fields)
                .build()
                .unwrap();
            
            let serialized = serde_json::to_string(&task).unwrap();
            info!("{:?}", serialized);

            // let ss = "{\"nodeId\":null,\"backendNodeId\":null,\"selector\":null,\"depth\":0,\"objectId\":null,\"pierce\":false}";
            let deserialized: DescribeNodeTask = serde_json::from_str(&serialized).unwrap();
            println!("deserialized = {:?}", deserialized);
        }
    }
}
