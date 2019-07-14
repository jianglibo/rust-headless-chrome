use super::DomEvent;
use super::super::TaskDescribe;
use crate::browser_async::{embedded_events};
use crate::protocol::{dom};

#[derive(Debug)]
pub struct AttributeModified {}
#[derive(Debug)]
pub struct AttributeRemoved {}
#[derive(Debug)]
pub struct CharacterDataModified {}

// "{\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"8CD21A9AA6837F6F1E4A661A73763B83\",\"message\":\"{\\\"method\\\":\\\"DOM.childNodeCountUpdated\\\",\\\"params\\\":{\\\"nodeId\\\":4,\\\"childNodeCount\\\":50}}\",\"targetId\":\"CDFB010DEEB3CF620374B1CCB84610F6\"}}"
wrapper_raw_event!(
    TaskDescribe::DomEvent,
    DomEvent::ChildNodeCountUpdated,
    ChildNodeCountUpdated,
    embedded_events::ChildNodeCountUpdated
);

// #[derive(Debug)]
// pub struct ChildNodeCountUpdated {}

#[derive(Debug)]
pub struct ChildNodeInserted {}
#[derive(Debug)]
pub struct ChildNodeRemoved {}

#[derive(Debug)]
pub struct DocumentUpdated {}

// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"90B4CCC5C3EC932DDF521282916B6619\",\"message\":\"{\\\"method\\\":\\\"DOM.setChildNodes\\\",\\\"params\\\":{\\\"parentId\\\":3,\\\"nodes\\\":[{\\\"nodeId\\\":4,\\\"parentId\\\":3,\\\"backendNodeId\\\":7,\\\"nodeType\\\":10,\\\"nodeName\\\":\\\"html\\\",\\\"localName\\\":\\\"\\\",\\\"nodeValue\\\":\\\"\\\",\\\"publicId\\\":\\\"\\\",\\\"systemId\\\":\\\"\\\"},{\\\"nodeId\\\":5,\\\"parentId\\\":3,\\\"backendNodeId\\\":8,\\\"nodeType\\\":1,\\\"nodeName\\\":\\\"HTML\\\",\\\"localName\\\":\\\"html\\\",\\\"nodeValue\\\":\\\"\\\",\\\"childNodeCount\\\":2,\\\"attributes\\\":[],\\\"frameId\\\":\\\"6380315C01D59D24229303681DA7E88D\\\"}]}}\",\"targetId\":\"6380315C01D59D24229303681DA7E88D\"}}
wrapper_raw_event!(
    TaskDescribe::DomEvent,
    DomEvent::SetChildNodes,
    SetChildNodes,
    embedded_events::SetChildNodes
);

impl SetChildNodes {
    pub fn into_parent_children(self) -> (dom::NodeId, Vec<dom::Node>) {
        (self.raw_event.params.parent_id, self.raw_event.params.nodes)
    }
}

impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::AttributeModified,
    AttributeModified
);
impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::AttributeRemoved,
    AttributeRemoved
);
impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::ChildNodeInserted,
    ChildNodeInserted
);
impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::ChildNodeRemoved,
    ChildNodeRemoved
);
impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::CharacterDataModified,
    CharacterDataModified
);

impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::DocumentUpdated,
    DocumentUpdated
);
