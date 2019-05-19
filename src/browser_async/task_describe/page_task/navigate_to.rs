use super::super::{TaskDescribe, TargetCallMethodTaskFace, TargetCallMethodTask, CommonDescribeFields};
use crate::protocol::{page, target};

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

impl TargetCallMethodTaskFace for NavigateToTask{
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
                let method = page::methods::Navigate {
            url: self.url,
            referrer: self.referrer.clone(),
            transition_type: self.transition_type.clone(),
            frame_id: self.frame_id.clone(),
        };
        self._to_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::NavigateTo, NavigateToTask);

// {\"method\":\"Target.sendMessageToTarget\",\"id\":5,\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.navigate\\\",\\\"id\\\":4,\\\"params\\\":{\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\\\"}}\"}}
// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"id\\\":4,\\\"result\\\":{\\\"frameId\\\":\\\"74FEEFE9CACC814F52F89930129A15ED\\\",\\\"loaderId\\\":\\\"53524592197E3E19D8E72E1379A32393\\\"}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}