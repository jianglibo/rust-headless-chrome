
pub mod dom {
    use serde::Deserialize;
    use crate::protocol::{self, dom as protocol_dom, input, page, page::methods::Navigate, target};

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SetChildNodes {
        pub method: String,
        pub params: SetChildNodesParams,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SetChildNodesParams {
        pub parentId: protocol_dom::NodeId,
        pub nodes: Vec<protocol_dom::Node>,
    }
}