use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{runtime};
use failure;
use log::*;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct EvaluateTask {
    pub common_fields: CommonDescribeFields,
    pub expression: String,
    #[builder(default = "None")]
    pub object_group: Option<String>,
    #[builder(default = "None")]
    pub include_command_line_a_p_i: Option<bool>,
    #[builder(default = "None")]
    pub silent: Option<bool>,
    #[builder(default = "None")]
    pub context_id: Option<runtime::ExecutionContextId>,
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
    pub time_out: Option<runtime::TimeDelta>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::EvaluateReturnObject>,
}

// task_result: Some(EvaluateReturnObject { result: RemoteObject { object_type: "number", subtype: None, class_name: None, value: Some(Number(20)), unserializable_value: None, description: Some("20"), object_id: None, preview: None }, exception_details: None }) }
// task_result: Some(EvaluateReturnObject { result: RemoteObject { object_type: "object", subtype: Some("array"), class_name: Some("NodeList"), value: None, unserializable_value: None, description: Some("NodeList(16)"), object_id: Some("{\"injectedScriptId\":11,\"id\":1}"), preview: None }, exception_details: None }) }
impl EvaluateTask {
    pub fn get_object_id(&self) -> Option<runtime::RemoteObjectId> {
        if let Some(ero) = self.task_result.as_ref() {
            if let Some(object_id) = ero.result.object_id.as_ref() {
                return Some(object_id).cloned();
            } else {
                error!("evaluate_result has empty object_id field. {:?}", self);
            }
        } else {
            error!("evaluate_result has empty result. {:?}", self);
        }
        None
    }

    pub fn get_string_result(&self) -> Option<&String> {
        if let Some(ero) = self.task_result.as_ref() {
            if let Some(serde_json::Value::String(jv)) = ero.result.value.as_ref() {
                return Some(jv);
            } else {
                error!("evaluate_result has empty value field. {:?}", self);
            }
        } else {
            error!("evaluate_result has empty result. {:?}", self);
        }
        None
    }
    pub fn get_u64_result(&self) -> Option<u64> {
        if let Some(ero) = self.task_result.as_ref() {
            if let Some(serde_json::Value::Number(jv)) = ero.result.value.as_ref() {
                return jv.as_u64();
            } else {
                error!("evaluate_result has empty value field. {:?}", self);
            }
        } else {
            error!("evaluate_result has empty result. {:?}", self);
        }
        None
    }
}

impl_has_common_fields!(EvaluateTask, "EvaluateTask");

impl AsMethodCallString for EvaluateTask {
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

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::Evaluate, EvaluateTask);