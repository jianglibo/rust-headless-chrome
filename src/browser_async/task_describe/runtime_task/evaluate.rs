use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{runtime, target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeEvaluateTask {
    pub common_fields: CommonDescribeFields,
    pub expression: String,
    #[builder(default = "None")]
    pub object_group: Option<String>,
    #[builder(default = "None")]
    pub include_command_line_a_p_i: Option<bool>,
    #[builder(default = "None")]
    pub silent: Option<bool>,
    #[builder(default = "None")]
    pub context_id: Option<runtime::types::ExecutionContextId>,
    #[builder(default = "None")]
    pub return_by_value: Option<bool>,
    #[builder(default = "None")]
    pub generate_preview: Option<bool>,
    #[builder(default = "None")]
    pub user_gesture: Option<bool>,
    #[builder(default = "None")]
    pub await_promise: Option<bool>,
    #[builder(default = "None")]
    pub throw_on_side_effect: Option<bool>,
    #[builder(default = "None")]
    pub time_out: Option<runtime::types::TimeDelta>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::EvaluateReturnObject>,
}

impl_has_common_fields!(RuntimeEvaluateTask);

impl AsMethodCallString for RuntimeEvaluateTask {
    fn get_method_str(&self) ->  Result<String, failure::Error>{
        let method = runtime::methods::Evaluate {
            expression: self.expression.as_str(),
            object_group: self.object_group.as_ref().map(String::as_str),
            include_command_line_a_p_i: self.include_command_line_a_p_i,
            silent: self.silent,
            context_id: self.context_id,
            return_by_value: self.return_by_value,
            generate_preview: self.generate_preview,
            user_gesture: self.user_gesture,
            await_promise: self.await_promise,
            throw_on_side_effect: self.throw_on_side_effect,
            time_out: self.time_out,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::RuntimeEvaluate, RuntimeEvaluateTask);