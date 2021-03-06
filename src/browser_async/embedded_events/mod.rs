use serde::Deserialize;
use super::super::protocol::{dom as protocol_dom, runtime, network, chrome_log, page};

pub use chrome_log::events::{LogEntryAdded};
use page::events::{WindowOpen, FrameRequestedNavigation, FrameResized};

pub mod network_raw_event;
pub use network_raw_event::{RequestWillBeSent, RequestWillBeSentParams, ResourceChangedPriority,
 ResponseReceived, DataReceived, LoadingFinished, ResponseReceivedParams, LoadingFailed, RequestServedFromCache};



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
pub struct ChildNodeCountUpdated {
    pub params: ChildNodeCountUpdatedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ChildNodeCountUpdatedParams {
    pub node_id: protocol_dom::NodeId,
    pub child_node_count: u64,
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
    #[serde(rename = "DOM.childNodeCountUpdated")]
    ChildNodeCountUpdated(ChildNodeCountUpdated),
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
    #[serde(rename = "Network.loadingFailed")]
    LoadingFailed(LoadingFailed),
    #[serde(rename = "Network.requestServedFromCache")]
    RequestServedFromCache(RequestServedFromCache),
    #[serde(rename = "Log.entryAdded")]
    LogEntryAdded(LogEntryAdded),
    #[serde(rename = "Page.windowOpen")]
    WindowOpen(WindowOpen),
    #[serde(rename = "Page.frameRequestedNavigation")]
    FrameRequestedNavigation(FrameRequestedNavigation),
    #[serde(rename = "Page.frameResized")]
    FrameResized(FrameResized),
    #[serde(rename = "Network.resourceChangedPriority")]
    ResourceChangedPriority(ResourceChangedPriority),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum EmbeddedEventWrapper {
    EmbeddedEvent(EmbeddedEvent)
}