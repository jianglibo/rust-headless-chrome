use std::collections::HashMap;

use serde::{Deserialize, Deserializer};

pub type NodeId = u16;

pub type NodeAttributes = HashMap<String, String>;

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RGBA {
    r: u8,
    g: u8,
    b: u8,
    a: Option<f64>,
}


#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    x: u64,
    y: u64,
    width: u64,
    height: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub node_id: NodeId,
    pub backend_node_id: NodeId,
    pub children: Option<Vec<Node>>,
    pub parent_id: Option<NodeId>,
    pub node_value: String,
    pub node_name: String,
    pub node_type: u8,
    #[serde(default, deserialize_with = "attribute_deser")]
    pub attributes: Option<NodeAttributes>,
    pub local_name: String,
    pub child_node_count: Option<u32>,
    #[serde(rename = "documentURL")]
    pub document_url: Option<String>,
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    pub public_id: Option<String>,
    pub system_id: Option<String>,
    pub internal_subset: Option<String>,
    pub xml_version: Option<String>,
    pub name: Option<String>,
    pub value: Option<String>,
    pub pseudo_type: Option<PseudoType>,
    pub shadow_root_type: Option<ShadowRootType>,
    pub frame_id: Option<String>,
    pub content_document: Option<Box<Node>>,
    pub shadow_roots: Option<Vec<Node>>,
    pub pseudo_elements: Option<Vec<Node>>,
    pub imported_document: Option<Box<Node>>,
    pub distributed_nodes: Option<Vec<BackendNode>>,
    #[serde(rename = "isSVG")]
    pub is_svg: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BackendNode {
    node_type: NodeId,
    node_name: String,
    backend_node_id: NodeId,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum PseudoType {
    FirstLine,
    FirstLetter,
    Before,
    After,
    Backdrop,
    Selection,
    FirstLineInherited,
    Scrollbar,
    ScrollbarThumb,
    ScrollbarButton,
    ScrollbarTrack,
    ScrollbarTrackPiece,
    ScrollbarCorner,
    Resizer,
    InputListButton,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum ShadowRootType {
    UserAgent,
    Open,
    Close,
}

fn attribute_deser<'de, D>(d: D) -> Result<Option<NodeAttributes>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<Vec<String>> = Option::deserialize(d)?;
    Ok(opt.map(|attr| {
        let mut map = HashMap::new();
        let mut iter = attr.into_iter();
        while let Some(n) = iter.next() {
            map.insert(n, iter.next().unwrap());
        }
        map
    }))
}

impl Node {
    /// Returns the first node for which the given closure returns true.
    ///
    /// Nodes are inspected breadth-first down their children.
    pub fn find<F: FnMut(&Self) -> bool>(&self, predicate: F) -> Option<&Self> {
        let mut s = SearchVisitor::new(predicate);
        s.visit(self);
        s.item
    }
}

struct SearchVisitor<'a, F> {
    predicate: F,
    item: Option<&'a Node>,
}

impl<'a, F: FnMut(&Node) -> bool> SearchVisitor<'a, F> {
    fn new(predicate: F) -> Self {
        SearchVisitor {
            predicate,
            item: None,
        }
    }

    fn visit(&mut self, n: &'a Node) {
        if (self.predicate)(n) {
            self.item = Some(n);
        } else if self.item.is_none() {
            if let Some(c) = &n.children {
                c.iter().for_each(|n| self.visit(n))
            }
        }
    }
}

pub mod methods {
    use serde::{Deserialize, Serialize};
    use crate::protocol::runtime;

    use crate::protocol::Method;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetDocument {
        pub depth: Option<u8>,
        pub pierce: Option<bool>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetDocumentReturnObject {
        pub root: super::Node,
    }
    impl Method for GetDocument {
        const NAME: &'static str = "DOM.getDocument";
        type ReturnObject = GetDocumentReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct DescribeNode {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<runtime::RemoteObjectId>,
        pub depth: Option<i8>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DescribeNodeReturnObject {
        pub node: super::Node,
    }
    impl Method for DescribeNode {
        const NAME: &'static str = "DOM.describeNode";
        type ReturnObject = DescribeNodeReturnObject;
    }

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct Focus {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FocusReturnObject {}
    impl Method for Focus {
        const NAME: &'static str = "DOM.focus";
        type ReturnObject = FocusReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetFileInputFiles<'a> {
        pub files: &'a [&'a str],
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<String>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetFileInputFilesReturnObject {}
    impl<'a> Method for SetFileInputFiles<'a> {
        const NAME: &'static str = "DOM.setFileInputFiles";
        type ReturnObject = SetFileInputFilesReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QuerySelector<'a> {
        pub node_id: super::NodeId,
        pub selector: &'a str,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QuerySelectorReturnObject {
        pub node_id: super::NodeId,
    }
    impl<'a> Method for QuerySelector<'a> {
        const NAME: &'static str = "DOM.querySelector";
        type ReturnObject = QuerySelectorReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct QuerySelectorAll<'a> {
        pub node_id: super::NodeId,
        pub selector: &'a str,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct QuerySelectorAllReturnObject {
        pub node_ids: Vec<super::NodeId>,
    }
    impl<'a> Method for QuerySelectorAll<'a> {
        const NAME: &'static str = "DOM.querySelectorAll";
        type ReturnObject = QuerySelectorAllReturnObject;
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RemoteObject {
        pub object_id: Option<String>,
    }
    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ResolveNode {
        pub backend_node_id: Option<super::NodeId>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ResolveNodeReturnObject {
        pub object: RemoteObject,
    }
    impl Method for ResolveNode {
        const NAME: &'static str = "DOM.resolveNode";
        type ReturnObject = ResolveNodeReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetContentQuads<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<&'a str>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetContentQuadsReturnObject {
        pub quads: Vec<[f64; 8]>,
    }
    impl<'a> Method for GetContentQuads<'a> {
        const NAME: &'static str = "DOM.getContentQuads";
        type ReturnObject = GetContentQuadsReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetBoxModel<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub backend_node_id: Option<super::NodeId>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub object_id: Option<&'a str>,
    }
    #[derive(Debug, Deserialize)]
    pub struct BoxModel {
        pub content: [f64; 8],
        pub padding: [f64; 8],
        pub border: [f64; 8],
        pub margin: [f64; 8],
        pub width: u64,
        pub height: u64,
        // TODO shapeOutside
    }
    #[derive(Debug, Deserialize)]
    pub struct GetBoxModelReturnObject {
        pub model: BoxModel,
    }
    impl<'a> Method for GetBoxModel<'a> {
        const NAME: &'static str = "DOM.getBoxModel";
        type ReturnObject = GetBoxModelReturnObject;
    }

}
