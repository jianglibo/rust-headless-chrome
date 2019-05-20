use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString,};
use crate::protocol::{dom, target};
use failure;

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct GetDocumentTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "Some(0)")]
    pub depth: Option<u8>,
    #[builder(default = "false")]
    pub pierce: bool,
    #[builder(setter(skip))]
    pub task_result: Option<dom::Node>,
}

impl_has_common_fields!(GetDocumentTask);

impl AsMethodCallString for GetDocumentTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = dom::methods::GetDocument {
            depth: self.depth.or(Some(0)),
            pierce: Some(self.pierce),
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetDocument, GetDocumentTask);

// {\"method\":\"Target.sendMessageToTarget\",\"id\":8,\"params\":{\"sessionId\":\"F85DC1275CA6B260B8CDB193EE948721\",\"message\":\"{\\\"method\\\":\\\"DOM.getDocument\\\",\\\"id\\\":6,\\\"params\\\":{\\\"depth\\\":0,\\\"pierce\\\":false}}\"}}
// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"F85DC1275CA6B260B8CDB193EE948721\",\"message\":\"{\\\"id\\\":6,\\\"result\\\":{\\\"root\\\":{\\\"nodeId\\\":1,\\\"backendNodeId\\\":3,\\\"nodeType\\\":9,\\\"nodeName\\\":\\\"#document\\\",\\\"localName\\\":\\\"\\\",\\\"nodeValue\\\":\\\"\\\",\\\"childNodeCount\\\":2,\\\"documentURL\\\":\\\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\\\",\\\"baseURL\\\":\\\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\\\",\\\"xmlVersion\\\":\\\"\\\"}}}\",\"targetId\":\"7A7BE708FCFEA9E7452B642492FD18EA\"}}