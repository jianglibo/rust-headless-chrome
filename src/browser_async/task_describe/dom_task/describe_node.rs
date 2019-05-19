use super::super::{
    CommonDescribeFields, TaskDescribe, TargetCallMethodTaskFace, TargetCallMethodTask
};
use crate::protocol::{dom, runtime, target};
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
    pub selector: Option<String>,
    #[builder(default = "Some(0)")]
    pub depth: Option<i8>,
    #[builder(default = "None")]
    pub object_id: Option<runtime::types::RemoteObjectId>,
    #[builder(default = "false")]
    pub pierce: bool,
    #[builder(default = "None")]
    #[serde(skip)]
    pub task_result: Option<dom::Node>,
}

impl TargetCallMethodTaskFace for DescribeNodeTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = dom::methods::DescribeNode {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            depth: self.depth,
        };
        self._to_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::DescribeNode, DescribeNodeTask);

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
