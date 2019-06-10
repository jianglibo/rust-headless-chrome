// place holder.
pub mod print_to_pdf;
pub mod navigate_to;
pub mod capture_screenshot;
pub mod page_enable;
pub mod page_events;
pub mod page_reload;

use crate::browser_async::page_message::{PageResponseWrapper, PageResponse, ReceivedEvent,};
pub use print_to_pdf::{PrintToPdfTask, PrintToPdfTaskBuilder};
pub use navigate_to::{NavigateToTask, NavigateToTaskBuilder};
pub use capture_screenshot::{CaptureScreenshotTask, CaptureScreenshotTaskBuilder};
pub use page_enable::{PageEnableTask, PageEnableTaskBuilder};
pub use page_reload::{PageReloadTask, PageReloadTaskBuilder};
use super::super::protocol::{target};
use super::super::EventName;
use crate::browser_async::debug_session::DebugSession;
use log::*;

#[derive(Debug)]
pub enum PageEvent {
    DomContentEventFired(page_events::DomContentEventFired),
    FrameAttached(page_events::FrameAttached),
    FrameDetached(page_events::FrameDetached),
    FrameNavigated(page_events::FrameNavigated),
    FrameStartedLoading(page_events::FrameStartedLoading),
    FrameStoppedLoading(page_events::FrameStoppedLoading),
    LoadEventFired(page_events::LoadEventFired),
}

fn handle_event_return(
    maybe_target_id: Option<target::TargetId>,
    page_response: PageResponse,
) -> Result<PageResponseWrapper, failure::Error> {
    Ok(PageResponseWrapper {
        target_id: maybe_target_id,
        task_id: None,
        page_response: page_response,
    })
}

#[allow(clippy::single_match_else)]
pub fn handle_page_event(
    debug_session: &mut DebugSession,
    page_event: PageEvent,
    maybe_session_id: Option<target::SessionID>,
    maybe_target_id: Option<target::TargetId>,
) -> Result<PageResponseWrapper, failure::Error> {
        match page_event {
            PageEvent::DomContentEventFired(event) => {}
            // attached may not invoke, if invoked it's the first. then started, navigated, stopped.
            PageEvent::FrameAttached(event) => {
                let raw_parameters = event.into_raw_parameters();
                let frame_id = raw_parameters.frame_id.clone();
                info!(
                    "-----------------frame_attached-----------------{:?}",
                    frame_id
                );
                let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab._frame_attached(raw_parameters);
                return handle_event_return(maybe_target_id, PageResponse::ReceivedEvent(ReceivedEvent::FrameAttached(frame_id)));
            }
            PageEvent::FrameDetached(event) => {
                let frame_id = event.into_frame_id();
                info!(
                    "-----------------frame_detached-----------------{:?}",
                    frame_id.clone()
                );
                let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab._frame_detached(&frame_id);
            }
            PageEvent::FrameStartedLoading(event) => {
                let frame_id = event.into_frame_id();
                // started loading is first, then attached.
                info!(
                    "-----------------frame_started_loading-----------------{:?}",
                    frame_id
                );
                let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab._frame_started_loading(frame_id.clone());
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::FrameStartedLoading(frame_id)),
                );
            }
            PageEvent::FrameNavigated(event) => {
                info!(
                    "-----------------frame_navigated-----------------{:?}",
                    event
                );
                let frame = event.get_frame();
                debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())
                    .expect("FrameNavigated event should have target_id.")
                    ._frame_navigated(event.clone_frame());
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::FrameNavigated(event)),
                );
            }
            PageEvent::FrameStoppedLoading(event) => {
                // TaskDescribe::FrameStoppedLoading(frame_id, common_fields) => {
                info!(
                    "-----------------frame_stopped_loading-----------------{:?}",
                    event
                );
                let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let frame_id = event.into_frame_id();
                tab._frame_stopped_loading(frame_id.clone());
                return handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::FrameStoppedLoading(frame_id)),
                );
            }
            PageEvent::LoadEventFired(event) => {
                let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.event_statistics.event_happened(EventName::LoadEventFired);
                return handle_event_return(maybe_target_id, event.into_page_response());
            }
        }
        warn!("unhandled branch handle_page_event");
        Ok(PageResponseWrapper::default())
}