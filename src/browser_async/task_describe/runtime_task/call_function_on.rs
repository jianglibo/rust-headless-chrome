use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace, TargetCallMethodTask};
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

impl TargetCallMethodTaskFace for RuntimeCallFunctionOnTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
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
        self._to_method_str(method)
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::RuntimeCallFunctionOn, RuntimeCallFunctionOnTask);