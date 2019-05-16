use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
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

impl From<NavigateToTask> for TaskDescribe {
    fn from(task: NavigateToTask) -> Self {
        TaskDescribe::NavigateTo(Box::new(task))
    }
}

impl CreateMethodCallString for NavigateToTask {
    fn create_method_call_string(&self, session_id: Option<&target::SessionID>, call_id: usize) -> String {
        let method = page::methods::Navigate {
            url: self.url,
            referrer: self.referrer.clone(),
            transition_type: self.transition_type.clone(),
            frame_id: self.frame_id.clone(),
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}