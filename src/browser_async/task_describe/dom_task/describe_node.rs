use super::super::{
    CommonDescribeFields, TaskDescribe, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString,
};
use crate::protocol::{dom, runtime, target};
use serde::{Deserialize, Serialize};
use failure;

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

impl_has_common_fields!(DescribeNodeTask);

impl AsMethodCallString for DescribeNodeTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let b = self.node_id.is_some() || self.backend_node_id.is_some() || self.object_id.is_some();
        failure::ensure!(b, "Either nodeId, backendNodeId or objectId must be specified");
        let method = dom::methods::DescribeNode {
            node_id: self.node_id,
            backend_node_id: self.backend_node_id,
            object_id: self.object_id.clone(),
            depth: self.depth,
        };
        Ok(self.create_method_str(method))
    }
}


impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::DescribeNode, DescribeNodeTask);


// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"A40CB7B0D59181D43BEC8EDC8C78EFB4\",\"message\":\"{\\\"error\\\":{\\\"code\\\":-32000,\\\"message\\\":\\\"Either nodeId, backendNodeId or objectId must be specified\\\"},\\\"id\\\":18}\",\"targetId\":\"FDFC29FA777DCB12E9FE09D48E0B40DE\"}}

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
