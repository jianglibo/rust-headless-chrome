use serde::Deserialize;

pub mod inner_events {
    use serde::Deserialize;
    use crate::protocol::{dom as protocol_dom };

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SetChildNodesEvent {
        pub params: SetChildNodesParams,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SetChildNodesParams {
        pub parentId: protocol_dom::NodeId,
        pub nodes: Vec<protocol_dom::Node>,
    }
}

// https://serde.rs/enum-representations.html

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "method")]
#[allow(clippy::large_enum_variant)]
pub enum InnerEvent {
    #[serde(rename = "DOM.setChildNodes")]
    SetChildNodes(inner_events::SetChildNodesEvent),
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum InnerEventWrapper {
    InnerEvent(InnerEvent)
}