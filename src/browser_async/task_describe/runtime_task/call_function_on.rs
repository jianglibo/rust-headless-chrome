use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{runtime, target};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeCallFunctionOnTask {
    pub common_fields: CommonDescribeFields,
    pub function_declaration: String,
    #[builder(default = "None")]
    pub object_id: Option<runtime::types::RemoteObjectId>,
    #[builder(default = "None")]
    pub silent: Option<bool>,
    #[builder(default = "None")]
    pub return_by_value: Option<bool>,
    #[builder(default = "None")]
    pub generate_preview: Option<bool>,
    #[builder(default = "None")]
    pub user_gesture: Option<bool>,
    #[builder(default = "None")]
    pub await_promise: Option<bool>,
    #[builder(default = "None")]
    pub execution_context_id: Option<runtime::types::ExecutionContextId>,
    #[builder(default = "None")]
    pub object_group: Option<String>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::CallFunctionOnReturnObject>,
}

impl From<RuntimeCallFunctionOnTask> for TaskDescribe {
    fn from(task: RuntimeCallFunctionOnTask) -> Self {
        TaskDescribe::RuntimeCallFunctionOn(Box::new(task))
    }
}


impl CreateMethodCallString for RuntimeCallFunctionOnTask {
    fn create_method_call_string(&self, session_id: Option<&target::SessionID>, call_id: usize) -> String {
        let method = runtime::methods::CallFunctionOn {
                function_declaration: self.function_declaration.as_ref(),
                object_id: self.object_id.clone(),
                silent: self.silent,
                return_by_value: self.return_by_value,
                generate_preview: self.generate_preview,
                user_gesture: self.user_gesture,
                await_promise: self.await_promise,
                execution_context_id: self.execution_context_id,
                object_group: self.object_group.as_ref(),
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}