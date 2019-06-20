pub mod describe_node;
pub mod get_box_model;
pub mod get_document;
pub mod query_selector;
pub mod dom_events;

pub use describe_node::{DescribeNodeTask, DescribeNodeTaskBuilder};
pub use get_box_model::{GetBoxModelTask, GetBoxModelTaskBuilder};
pub use get_document::{GetDocumentTask, GetDocumentTaskBuilder};
pub use query_selector::{QuerySelectorTask, QuerySelectorTaskBuilder};

use crate::browser_async::{DebugSession};
use crate::browser_async::page_message::{PageResponse, PageResponseWrapper, ReceivedEvent};
use crate::protocol::{target};
use log::*;


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

pub    fn handle_dom_event(
        debug_session: &mut DebugSession,
        dom_event: DomEvent,
        _maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match dom_event {
            DomEvent::AttributeModified(_event) => {}
            DomEvent::AttributeRemoved(_event) => {}
            DomEvent::CharacterDataModified(_event) => {}
            DomEvent::ChildNodeCountUpdated(_event) => {}
            DomEvent::ChildNodeInserted(_event) => {}
            DomEvent::ChildNodeRemoved(_event) => {}
            DomEvent::DocumentUpdated(_event) => {}
            DomEvent::SetChildNodes(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                let (parent_id, nodes) = event.into_parent_children();
                tab.node_arrived(parent_id, nodes);
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: None,
                    page_response: PageResponse::ReceivedEvent(ReceivedEvent::SetChildNodesOccurred(parent_id)),
                });
            }
        }
        warn!("unhandled branch handle_dom_event");
        Ok(PageResponseWrapper::default())
    }