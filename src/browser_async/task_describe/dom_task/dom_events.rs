use super::super::{DomEvent, TaskDescribe};
use crate::browser_async::embedded_events;

#[derive(Debug)]
pub struct AttributeModified {}
#[derive(Debug)]
pub struct AttributeRemoved {}
#[derive(Debug)]
pub struct CharacterDataModified {}
#[derive(Debug)]
pub struct ChildNodeCountUpdated {}

#[derive(Debug)]
pub struct ChildNodeInserted {}
#[derive(Debug)]
pub struct ChildNodeRemoved {}

#[derive(Debug)]
pub struct DocumentUpdated {}

wrapper_raw_event!(
    TaskDescribe::DomEvent,
    DomEvent::SetChildNodes,
    SetChildNodes,
    embedded_events::SetChildNodes
);

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
    DomEvent::ChildNodeCountUpdated,
    ChildNodeCountUpdated
);
impl_into_task_describe!(
    TaskDescribe::DomEvent,
    DomEvent::DocumentUpdated,
    DocumentUpdated
);
