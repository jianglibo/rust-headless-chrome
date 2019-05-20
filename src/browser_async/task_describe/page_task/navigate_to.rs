use super::super::{TaskDescribe, AsMethodCallString, TargetCallMethodTask, CommonDescribeFields, HasCommonField, CanCreateMethodString,};
use crate::protocol::{page, target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct NavigateToTask {
    pub common_fields: CommonDescribeFields,
    pub url: &'static str,
    #[builder(default = "None")]
    pub referrer: Option<String>,
    #[builder(default = "None")]
    pub transition_type: Option<page::types::TransitionType>,
    #[builder(default = "None")]
    pub frame_id: Option<page::types::FrameId>,
    #[builder(default = "None")]
    pub task_result: Option<page::methods::NavigateReturnObject>,
}

impl_has_common_fields!(NavigateToTask);

impl AsMethodCallString for NavigateToTask{
    fn get_method_str(&self) -> Result<String, failure::Error>{
                let method = page::methods::Navigate {
            url: self.url,
            referrer: self.referrer.clone(),
            transition_type: self.transition_type.clone(),
            frame_id: self.frame_id.clone(),
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::NavigateTo, NavigateToTask);

// {\"method\":\"Target.sendMessageToTarget\",\"id\":5,\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.navigate\\\",\\\"id\\\":4,\\\"params\\\":{\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\\\"}}\"}}
// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"id\\\":4,\\\"result\\\":{\\\"frameId\\\":\\\"74FEEFE9CACC814F52F89930129A15ED\\\",\\\"loaderId\\\":\\\"53524592197E3E19D8E72E1379A32393\\\"}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}