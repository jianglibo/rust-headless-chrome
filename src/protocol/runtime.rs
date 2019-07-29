
    use serde::{Deserialize};

    pub type ExecutionContextId = u16;
    pub type TimeDelta = u32;
    pub type ScriptId = String;
    pub type RemoteObjectId = String;
    pub type Timestamp = u64;
    

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

    /// RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":2}"), preview: None }
    /// { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function entries() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":19}"), preview: None }
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

    ///  name: "0", value: Some(RemoteObject), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None 
    ///  name: "entries", value: Some(RemoteObject ), writable: Some(true), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(false), symbol: None 
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

pub mod methods {
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};
    use super::*;

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionOn<'a> {
        pub function_declaration: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<RemoteObjectId>,
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
        pub execution_context_id: Option<ExecutionContextId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_group: Option<&'a String>,
    }
    
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct CallFunctionOnReturnObject {
        pub result: RemoteObject,
        pub exception_details: Option<ExceptionDetails>,
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

    /// result: [PropertyDescriptor { name: "0", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":2}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "1", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":3}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "2", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":4}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "3", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":5}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "4", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":6}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "5", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":7}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "6", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":8}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "7", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":9}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "8", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":10}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "9", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":11}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "10", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":12}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "11", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":13}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "12", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":14}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "13", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":15}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "14", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":16}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "15", value: Some(RemoteObject { object_type: "object", subtype: Some("node"), class_name: Some("HTMLSpanElement"), value: None, unserializable_value: None, description: Some("span.text"), object_id: Some("{\"injectedScriptId\":11,\"id\":17}"), preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "length", value: Some(RemoteObject { object_type: "number", subtype: None, class_name: None, value: Some(Number(16)), unserializable_value: None, description: Some("16"), object_id: None, preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(true), symbol: None }, PropertyDescriptor { name: "item", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function item() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":18}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor {}, PropertyDescriptor { name: "forEach", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function forEach() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":20}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "keys", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function keys() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":21}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "values", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function values() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":22}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: true, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "constructor", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function NodeList() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":23}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "Symbol(Symbol.toStringTag)", value: Some(RemoteObject { object_type: "string", subtype: None, class_name: None, value: Some(String("NodeList")), unserializable_value: None, description: None, object_id: None, preview: None }), writable: Some(false), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: Some(RemoteObject { object_type: "symbol", subtype: None, class_name: None, value: None, unserializable_value: None, description: Some("Symbol(Symbol.toStringTag)"), object_id: Some("{\"injectedScriptId\":11,\"id\":24}"), preview: None }) }, PropertyDescriptor { name: "Symbol(Symbol.iterator)", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function values() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":25}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: Some(RemoteObject { object_type: "symbol", subtype: None, class_name: None, value: None, unserializable_value: None, description: Some("Symbol(Symbol.iterator)"), object_id: Some("{\"injectedScriptId\":11,\"id\":26}"), preview: None }) }, PropertyDescriptor { name: "__defineGetter__", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function __defineGetter__() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":27}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "__defineSetter__", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function __defineSetter__() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":28}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "hasOwnProperty", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function hasOwnProperty() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":29}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "__lookupGetter__", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function __lookupGetter__() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":30}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "__lookupSetter__", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function __lookupSetter__() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":31}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "isPrototypeOf", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function isPrototypeOf() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":32}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "propertyIsEnumerable", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function propertyIsEnumerable() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":33}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "toString", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function toString() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":34}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "valueOf", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function valueOf() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":35}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "__proto__", value: None, writable: None, get: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function get __proto__() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":36}"), preview: None }), set: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function set __proto__() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":37}"), preview: None }), configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }, PropertyDescriptor { name: "toLocaleString", value: Some(RemoteObject { object_type: "function", subtype: None, class_name: Some("Function"), value: None, unserializable_value: None, description: Some("function toLocaleString() { [native code] }"), object_id: Some("{\"injectedScriptId\":11,\"id\":38}"), preview: None }), writable: Some(true), get: None, set: None, configurable: true, enumerable: false, was_thrown: None, is_own: Some(false), symbol: None }], internal_properties: None, exception_details: None 
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct GetPropertiesReturnObject {
        pub result: Vec<PropertyDescriptor>,
        pub internal_properties: Option<Vec<InternalPropertyDescriptor>>,
        // pub private_properties: Option<PrivatePropertyDescriptor>,
        pub exception_details: Option<ExceptionDetails>,

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
        pub context_id: Option<ExecutionContextId>,
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
        pub time_out: Option<TimeDelta>,
    }
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct EvaluateReturnObject {
        pub result: RemoteObject,
        pub exception_details: Option<ExceptionDetails>,
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