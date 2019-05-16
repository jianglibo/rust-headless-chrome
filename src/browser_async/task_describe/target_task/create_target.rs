use crate::browser_async::{create_msg_to_send, CreateMethodCallString, TaskDescribe, MethodDestination};
use crate::protocol::target;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CreateTargetTask {
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

impl From<CreateTargetTask> for TaskDescribe {
    fn from(task: CreateTargetTask) -> Self {
        TaskDescribe::CreateTarget(Box::new(task))
    }
}

impl CreateMethodCallString for CreateTargetTask {
    fn create_method_call_string(&self, _session_id: Option<&target::SessionID>, call_id: usize) -> String {
        let method = target::methods::CreateTarget {
            url: self.url.as_ref(),
            width: self.width,
            height: self.height,
            browser_context_id: self.browser_context_id.as_ref().map(String::as_str),
            enable_begin_frame_control: self.enable_begin_frame_control,
        };
        create_msg_to_send(method, MethodDestination::Browser, call_id)
    }
}
