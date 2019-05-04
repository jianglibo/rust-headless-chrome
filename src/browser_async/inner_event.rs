use serde::Deserialize;

pub mod inner_events {
    use serde::Deserialize;
    use crate::protocol::{dom as protocol_dom, runtime};

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
        pub timestamp: f32,
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
}

// https://serde.rs/enum-representations.html

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "method")]
#[allow(clippy::large_enum_variant)]
pub enum InnerEvent {
    #[serde(rename = "DOM.setChildNodes")]
    SetChildNodes(inner_events::SetChildNodes),
    #[serde(rename = "Page.loadEventFired")]
    LoadEventFired(inner_events::LoadEventFired),
    #[serde(rename = "Runtime.executionContextCreated")]
    ExecutionContextCreated(inner_events::ExecutionContextCreated),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum InnerEventWrapper {
    InnerEvent(InnerEvent)
}