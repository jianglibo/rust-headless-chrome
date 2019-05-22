use serde::Deserialize;
use crate::protocol::{dom as protocol_dom, runtime, network};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetChildNodes {
    pub params: SetChildNodesParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SetChildNodesParams {
    pub parent_id: protocol_dom::NodeId,
    pub nodes: Vec<protocol_dom::Node>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadEventFired {
    pub params: LoadEventFiredParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadEventFiredParams {
    pub timestamp: network::types::MonotonicTime,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomContentEventFired {
    pub params: DomContentEventFiredParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomContentEventFiredParams {
    pub timestamp: network::types::MonotonicTime,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextCreated {
    pub params: ExecutionContextCreatedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextCreatedParams {
    pub context: runtime::types::ExecutionContextDescription,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextDestroyed {
    pub params: ExecutionContextDestroyedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextDestroyedParams {
    pub execution_context_id: runtime::types::ExecutionContextId,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleAPICalled {
    pub params: ConsoleAPICalledParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ConsoleAPICalledParams {
    #[serde(rename = "type")]
    pub object_type: String,
    pub args: Vec<runtime::types::RemoteObject>,
    pub execution_context_id: runtime::types::ExecutionContextId,
    pub stack_trace: Option<runtime::types::StackTrace>,
    pub context: Option<String>,
}

// https://serde.rs/enum-representations.html

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "method")]
#[allow(clippy::large_enum_variant)]
pub enum EmbeddedEvent {
    #[serde(rename = "DOM.setChildNodes")]
    SetChildNodes(SetChildNodes),
    #[serde(rename = "Page.loadEventFired")]
    LoadEventFired(LoadEventFired),
    #[serde(rename = "Runtime.executionContextCreated")]
    ExecutionContextCreated(ExecutionContextCreated),
    #[serde(rename = "Runtime.executionContextDestroyed")]
    ExecutionContextDestroyed(ExecutionContextDestroyed),
    #[serde(rename = "Runtime.consoleAPICalled")]
    ConsoleAPICalled(ConsoleAPICalled),
    #[serde(rename = "Page.domContentEventFired")]
    DomContentEventFired(DomContentEventFired),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum EmbeddedEventWrapper {
    EmbeddedEvent(EmbeddedEvent)
}