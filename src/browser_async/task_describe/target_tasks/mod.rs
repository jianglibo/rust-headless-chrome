pub mod create_target;
pub mod set_discover_target_task;
pub mod target_events;
pub mod close_target;

pub use create_target::{CreateTargetTask, CreateTargetTaskBuilder};
pub use close_target::{CloseTargetTask, CloseTargetTaskBuilder};
pub use set_discover_target_task::{SetDiscoverTargetsTask, SetDiscoverTargetsTaskBuilder};

use crate::browser_async::{DebugSession, Tab};
use crate::browser_async::page_message::{PageResponse, PageResponseWrapper, ReceivedEvent};
use crate::protocol::{target};
use log::*;
use std::sync::Arc;



#[derive(Debug)]
pub enum TargetEvent {
    ReceivedMessageFromTarget(target_events::ReceivedMessageFromTarget),
    TargetCreated(target_events::TargetCreated),
    TargetCrashed(target_events::TargetCrashed),
    TargetDestroyed(target_events::TargetDestroyed),
    TargetInfoChanged(target_events::TargetInfoChanged),
    AttachedToTarget(target_events::AttachedToTarget),
}

pub  fn handle_target_event(
        debug_session: &mut DebugSession,
        target_event: TargetEvent,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match target_event {
            TargetEvent::ReceivedMessageFromTarget(event) => {}
            TargetEvent::TargetCreated(event) => {
                if let target::TargetType::Page = event.get_target_type() {
                    // info!("receive page created event: {:?}", event);
                    let target_info = event.to_target_info();
                    let target_id = target_info.target_id.clone();
                    let tab = Tab::new(target_info, Arc::clone(&debug_session.chrome_debug_session));
                    debug_session.tabs.push(tab);
                    let idx = debug_session.tabs.len();
                    return Ok(PageResponseWrapper {
                        target_id: Some(target_id),
                        task_id: None,
                        page_response: PageResponse::ReceivedEvent(ReceivedEvent::PageCreated(idx)),
                    });
                } else {
                    info!("got other target_event: {:?}", event);
                }
            }
            TargetEvent::TargetCrashed(event) => {}
            TargetEvent::AttachedToTarget(event) => {
                if event.is_page_attached() {
                    let target_id = event.get_target_id();
                    let tab = debug_session
                        .find_tab_by_id_mut(Some(&target_id))
                        .expect("when the page attached, tab should have been exists.");
                    // tab.session_id.replace(event.get_session_id());
                    tab.page_attached(event.get_session_id());
                    return Ok(event
                        .try_into_page_attached()
                        .expect("should be a page attached."));
                } else {
                    info!("got AttachedToTarget event it's target_type was other than page.");
                }
            }
            TargetEvent::TargetInfoChanged(event) => {
                let target_info = event.into_target_info();
                if let Ok(tab) = debug_session.find_tab_by_id_mut(Some(&target_info.target_id)) {
                    tab.target_info = target_info;
                    trace!("target info changed: {:?}", tab.target_info);
                } else {
                    warn!("target changed, no correspond tab. {:?}", target_info);
                }
            }
            TargetEvent::TargetDestroyed(event) => {
                let target_id = event.get_target_id();
                if let Ok(_tab) = debug_session.find_tab_by_id_mut(Some(target_id)) {
                    debug_session.tab_closed(target_id);
                } else {
                    warn!("target destroyed, no correspond tab. {:?}", event);
                }
            }
        }
        warn!("unhandled branch handle_target_event");
        Ok(PageResponseWrapper::default())
    }