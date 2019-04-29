pub type ExecutionContextId = u16;
pub type TimeDelta = u32;

pub mod methods {
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};
    use super::*;

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct PropertyPreview {
        pub name: String,
        #[serde(rename = "type")]
        pub object_type: String,
        pub value: Option<String>,
        pub value_preview: Option<Box<PropertyPreview>>,
        pub subtype: Option<String>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ObjectPreview {
        #[serde(rename = "type")]
        pub object_type: String,
        pub subtype: Option<String>,
        pub description: Option<String>,
        pub overflow: bool,
        pub properties: Vec<PropertyPreview>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct RemoteObject {
        #[serde(rename = "type")]
        pub object_type: String,
        pub subtype: Option<String>,
        pub description: Option<String>,
        pub class_name: Option<String>,
        pub value: Option<serde_json::Value>,
        pub unserializable_value: Option<String>,
        pub preview: Option<ObjectPreview>,
    }

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionOn<'a> {
        pub object_id: &'a str,
        pub function_declaration: &'a str,
        pub return_by_value: bool,
        pub generate_preview: bool,
        pub silent: bool,
        pub await_promise: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionOnReturnObject {
        pub result: RemoteObject,
    }
    impl<'a> Method for CallFunctionOn<'a> {
        const NAME: &'static str = "Runtime.callFunctionOn";
        type ReturnObject = CallFunctionOnReturnObject;
    }

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct Evaluate<'a> {
        pub expression: &'a str,
        pub object_group: Option<&'a str>,
        pub include_command_line_a_p_i: bool,
        pub silent: bool,
        pub context_id: ExecutionContextId,
        pub return_by_value: bool,
        pub generate_preview: bool,
        pub user_gesture: bool,
        pub await_promise: bool,
        pub throw_on_side_effect: bool,
        pub time_out: TimeDelta,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EvaluateReturnObject {
        pub result: RemoteObject,
    }

    impl<'a> Method for Evaluate<'a> {
        const NAME: &'static str = "Runtime.evaluate";
        type ReturnObject = EvaluateReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EnableReturnObject {}
    impl Method for Enable {
        const NAME: &'static str = "Runtime.enable";
        type ReturnObject = EnableReturnObject;
    }
}
