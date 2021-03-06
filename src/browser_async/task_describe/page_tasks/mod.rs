// place holder.
pub mod print_to_pdf;
pub mod navigate_to;
pub mod capture_screenshot;
pub mod page_enable;
pub mod page_close;
pub mod page_events;
pub mod page_reload;
pub mod get_layout_metrics;
pub mod bring_to_front;
pub mod attach_to_target;
pub mod set_life_cycle_events_enable;

use crate::browser_async::page_message::{PageResponseWrapper, PageResponse, ReceivedEvent,};
pub use print_to_pdf::{PrintToPdfTask, PrintToPdfTaskBuilder};
pub use navigate_to::{NavigateToTask, NavigateToTaskBuilder};
pub use capture_screenshot::{CaptureScreenshotTask, CaptureScreenshotTaskBuilder};
pub use page_enable::{PageEnableTask, PageEnableTaskBuilder};
pub use page_close::{PageCloseTask, PageCloseTaskBuilder};
pub use page_reload::{PageReloadTask, PageReloadTaskBuilder};
pub use get_layout_metrics::{GetLayoutMetricsTask, GetLayoutMetricsTaskBuilder};
pub use bring_to_front::{BringToFrontTask, BringToFrontTaskBuilder};
pub use attach_to_target::{AttachToTargetTask, AttachToTargetTaskBuilder};
pub use set_life_cycle_events_enable::{SetLifecycleEventsEnabledTask, SetLifecycleEventsEnabledTaskBuilder};

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
    LifeCycle(page_events::LifeCycle),
    WindowOpen(page_events::WindowOpen),
    FrameRequestedNavigation(page_events::FrameRequestedNavigation),
    FrameResized(page_events::FrameResized),
}

fn handle_event_return(
    maybe_target_id: Option<target::TargetId>,
    page_response: PageResponse,
) -> Result<PageResponseWrapper, failure::Error> {
    Ok(PageResponseWrapper {
        target_id: maybe_target_id,
        task_id: None,
        page_response
    })
}

#[allow(clippy::single_match_else)]
pub fn handle_page_event(
    debug_session: &mut DebugSession,
    page_event: PageEvent,
    _maybe_session_id: Option<target::SessionID>,
    maybe_target_id: Option<target::TargetId>,
) -> Result<PageResponseWrapper, failure::Error> {
        match page_event {
            PageEvent::DomContentEventFired(event) => {
                trace!("unhandled DomContentEventFired: {:?}", event);
                Ok(PageResponseWrapper::default())
            }
            // attached may not invoke, if invoked it's the first. then started, navigated, stopped.
            PageEvent::FrameAttached(event) => {
                let raw_parameters = event.into_raw_parameters();
                let frame_id = raw_parameters.frame_id.clone();
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                info!(
                    "-----------------frame_attached-----------------{:?}, tab url: {:?}",
                    frame_id, tab.get_url()
                );
                tab.changing_frames._frame_attached(raw_parameters);
                handle_event_return(maybe_target_id, PageResponse::ReceivedEvent(ReceivedEvent::FrameAttached(frame_id)))
            }
            PageEvent::FrameDetached(event) => {
                let frame_id = event.into_frame_id();
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                info!(
                    "-----------------frame_detached-----------------{:?}, tab url: {:?}",
                    frame_id.clone(), tab.get_url()
                );
                tab.changing_frames._frame_detached(&frame_id);
                Ok(PageResponseWrapper::default())
            }
            PageEvent::FrameStartedLoading(event) => {
                let frame_id = event.into_frame_id();
                // started loading is first, then attached.
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                info!(
                    "-----------------frame_started_loading-----------------{:?}, tab url: {:?}",
                    frame_id, tab.get_url()
                );
                tab.changing_frames._frame_started_loading(frame_id.clone());
                handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::FrameStartedLoading(frame_id)),
                )
            }
            PageEvent::FrameNavigated(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())
                    .expect("FrameNavigated event should have target_id.");
                info!(
                    "-----------------frame_navigated-----------------{:?}, tab url: {:?}",
                    event, tab.get_url()
                );

                tab._frame_navigated(event.clone_frame());
                handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::FrameNavigated(event)),
                )
            }
            PageEvent::FrameStoppedLoading(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                info!(
                    "-----------------frame_stopped_loading-----------------{:?}, tab url: {:?}",
                    event, tab.get_url()
                );
                let frame_id = event.into_frame_id();
                tab.changing_frames._frame_stopped_loading(frame_id.clone());
                handle_event_return(
                    maybe_target_id,
                    PageResponse::ReceivedEvent(ReceivedEvent::FrameStoppedLoading(frame_id)),
                )
            }
            PageEvent::LoadEventFired(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.event_statistics.event_happened(EventName::LoadEventFired);
                handle_event_return(maybe_target_id, event.into_page_response())
            }
            PageEvent::LifeCycle(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.life_cycles.life_cycle_happened(event);
                handle_event_return(maybe_target_id, 
                PageResponse::ReceivedEvent(ReceivedEvent::LifeCycle))
            }
            PageEvent::WindowOpen(event) => {
                handle_event_return(maybe_target_id,
                 PageResponse::ReceivedEvent(ReceivedEvent::WindowOpen(event)))
            }
            PageEvent::FrameRequestedNavigation(event) => {
                handle_event_return(maybe_target_id,
                 PageResponse::ReceivedEvent(ReceivedEvent::FrameRequestedNavigation(event)))
            }
            PageEvent::FrameResized(event) => {
                handle_event_return(maybe_target_id,
                 PageResponse::ReceivedEvent(ReceivedEvent::FrameResized(event)))
            }
        }
}