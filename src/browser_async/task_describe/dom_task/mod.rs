pub mod describe_node;
pub mod get_box_model;
pub mod get_document;
pub mod query_selector;
pub mod dom_events;

pub use describe_node::{DescribeNodeTask, DescribeNodeTaskBuilder};
pub use get_box_model::{GetBoxModelTask, GetBoxModelTaskBuilder};
pub use get_document::{GetDocumentTask, GetDocumentTaskBuilder};
pub use query_selector::{QuerySelectorTask, QuerySelectorTaskBuilder};


#[derive(Debug)]
pub enum DomEvent {
    AttributeModified(dom_events::AttributeModified),
    AttributeRemoved(dom_events::AttributeRemoved),
    CharacterDataModified(dom_events::CharacterDataModified),
    ChildNodeCountUpdated(dom_events::ChildNodeCountUpdated),
    ChildNodeInserted(dom_events::ChildNodeInserted),
    ChildNodeRemoved(dom_events::ChildNodeRemoved),
    DocumentUpdated(dom_events::DocumentUpdated),
    SetChildNodes(dom_events::SetChildNodes),
}