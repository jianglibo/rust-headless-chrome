use serde::Deserialize;
use crate::protocol::{dom as protocol_dom, runtime, network, page};

mod network_raw_event;

pub use network_raw_event::{RequestWillBeSent, ResponseReceived, DataReceived, LoadingFinished, ResponseReceivedParams};

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
    pub timestamp: network::MonotonicTime,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomContentEventFired {
    pub params: DomContentEventFiredParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DomContentEventFiredParams {
    pub timestamp: network::MonotonicTime,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextCreated {
    pub params: ExecutionContextCreatedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextCreatedParams {
    pub context: runtime::ExecutionContextDescription,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextDestroyed {
    pub params: ExecutionContextDestroyedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionContextDestroyedParams {
    pub execution_context_id: runtime::ExecutionContextId,
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
    pub args: Vec<runtime::RemoteObject>,
    pub execution_context_id: runtime::ExecutionContextId,
    pub stack_trace: Option<runtime::StackTrace>,
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
    #[serde(rename = "Network.requestWillBeSent")]
    RequestWillBeSent(RequestWillBeSent),
    #[serde(rename = "Network.responseReceived")]
    ResponseReceived(ResponseReceived),
    #[serde(rename = "Network.dataReceived")]
    DataReceived(DataReceived),
    #[serde(rename = "Network.loadingFinished")]
    LoadingFinished(LoadingFinished),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum EmbeddedEventWrapper {
    EmbeddedEvent(EmbeddedEvent)
}