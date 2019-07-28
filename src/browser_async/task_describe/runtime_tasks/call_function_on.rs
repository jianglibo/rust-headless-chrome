use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{runtime};
use crate::browser::tab::point::Point;
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct CallFunctionOnTask {
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

impl CallFunctionOnTask {
    pub fn get_midpoint(&self) -> Option<Point> {
        if let Some(task_return_object) = self.task_result.clone() {
            let properties = task_return_object.result
                .preview
                .expect("JS couldn't give us quad for element")
                .properties;
            let mut prop_map = std::collections::HashMap::new();

            for prop in properties {
                prop_map.insert(prop.name, prop.value.unwrap().parse::<f64>().unwrap());
            }

            let midpoint = Point {
                x: prop_map["x"] + (prop_map["width"] / 2.0),
                y: prop_map["y"] + (prop_map["height"] / 2.0),
            };
            Some(midpoint)
        } else {
            None
        }
    }
}

impl_has_common_fields!(CallFunctionOnTask, "CallFunctionOnTask");

impl AsMethodCallString for CallFunctionOnTask {
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

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::RuntimeCallFunctionOn, CallFunctionOnTask);