use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{runtime};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeCallFunctionOnTask {
    pub common_fields: CommonDescribeFields,
    pub function_declaration: String,
    #[builder(default = "None")]
    pub object_id: Option<runtime::RemoteObjectId>,
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
    pub execution_context_id: Option<runtime::ExecutionContextId>,
    #[builder(default = "None")]
    pub object_group: Option<String>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::CallFunctionOnReturnObject>,
}

impl_has_common_fields!(RuntimeCallFunctionOnTask);

impl AsMethodCallString for RuntimeCallFunctionOnTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
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
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::RuntimeCallFunctionOn, RuntimeCallFunctionOnTask);