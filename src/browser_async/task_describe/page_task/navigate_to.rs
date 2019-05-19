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