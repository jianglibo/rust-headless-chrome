
pub mod types {
    use serde::{Deserialize};

    pub type ExecutionContextId = u16;
    pub type TimeDelta = u32;
    pub type ScriptId = String;
    pub type RemoteObjectId = String;

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFrame {
        pub function_name: String,
        pub script_id: ScriptId,
        pub url: String,
        pub line_number: u32,
        pub column_number: u32,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct StackTrace {
        pub description: Option<String>,
        pub call_frames: Vec<CallFrame>,
        pub parent: Option<Box<StackTrace>>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ExceptionDetails {
        pub exception_id: u16,
        pub text: String,
        pub line_number: u16,
        pub column_number: u16,
        pub script_id: Option<ScriptId>,
        pub url: Option<String>,
        pub stack_trace: Option<StackTrace>,
        pub exception: Option<RemoteObject>,
        pub execution_context_id: Option<ExecutionContextId>,
    }

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
        pub class_name: Option<String>,
        pub value: Option<serde_json::Value>,
        pub unserializable_value: Option<String>,
        pub description: Option<String>,
        pub object_id: Option<RemoteObjectId>,
        pub preview: Option<ObjectPreview>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct PropertyDescriptor {
        pub name: String,
        pub value: Option<RemoteObject>,
        pub writable: Option<bool>,
        pub get: Option<RemoteObject>,
        pub set: Option<RemoteObject>,
        pub configurable: bool,
        pub enumerable: bool,
        pub was_thrown: Option<bool>,
        pub is_own: Option<bool>,
        pub symbol: Option<RemoteObject>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct InternalPropertyDescriptor {
        pub name: String,
        pub value: Option<RemoteObject>,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ExecutionContextDescription {
        pub id: ExecutionContextId,
        pub origin: String,
        pub name: String,
        pub aux_data: serde_json::Value,
    }
}

pub mod methods {
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};
    use super::types;

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionOn<'a> {
        pub function_declaration: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<types::RemoteObjectId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub silent: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub return_by_value: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub generate_preview: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_gesture: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub await_promise: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub execution_context_id: Option<types::ExecutionContextId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_group: Option<&'a String>,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionOnReturnObject {
        pub result: types::RemoteObject,
        pub exception_details: Option<types::ExceptionDetails>,
    }
    impl<'a> Method for CallFunctionOn<'a> {
        const NAME: &'static str = "Runtime.callFunctionOn";
        type ReturnObject = CallFunctionOnReturnObject;
    }

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct GetProperties<'a> {
        pub object_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub own_properties: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub accessor_properties_only: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub generate_preview: Option<bool>,
    }
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct GetPropertiesReturnObject {
        pub result: Vec<types::PropertyDescriptor>,
        pub internal_properties: Option<Vec<types::InternalPropertyDescriptor>>,
        // pub private_properties: Option<types::PrivatePropertyDescriptor>,
        pub exception_details: Option<types::ExceptionDetails>,

    }
    impl<'a> Method for GetProperties<'a> {
        const NAME: &'static str = "Runtime.getProperties";
        type ReturnObject = GetPropertiesReturnObject;
    }

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct Evaluate<'a> {
        pub expression: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_group: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub include_command_line_a_p_i: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub silent: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub context_id: Option<types::ExecutionContextId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub return_by_value: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub generate_preview: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub user_gesture: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub await_promise: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub throw_on_side_effect: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub time_out: Option<types::TimeDelta>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EvaluateReturnObject {
        pub result: types::RemoteObject,
        pub exception_details: Option<types::ExceptionDetails>,
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