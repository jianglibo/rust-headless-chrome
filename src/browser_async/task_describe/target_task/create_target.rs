use crate::browser_async::{create_msg_to_send, MethodDestination};
use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace};
use crate::protocol::target;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CreateTargetTask {
    pub common_fields: CommonDescribeFields,
    pub url: String,
    #[builder(default = "None")]
    pub width: Option<i32>,
    #[builder(default = "None")]
    pub height: Option<i32>,
    #[builder(default = "None")]
    pub browser_context_id: Option<target::BrowserContextID>,
    #[builder(default = "None")]
    pub enable_begin_frame_control: Option<bool>,
    #[builder(default = "None")]
    pub task_result: Option<target::TargetId>,
}

// impl From<CreateTargetTask> for TaskDescribe {
//     fn from(task: CreateTargetTask) -> Self {
//         TaskDescribe::CreateTarget(Box::new(task))
//     }
// }

impl TargetCallMethodTaskFace for CreateTargetTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = target::methods::CreateTarget {
            url: self.url.as_ref(),
            width: self.width,
            height: self.height,
            browser_context_id: self.browser_context_id.as_ref().map(String::as_str),
            enable_begin_frame_control: self.enable_begin_frame_control,
        };
        create_msg_to_send(method, MethodDestination::Browser, self.get_call_id())
    }
}
