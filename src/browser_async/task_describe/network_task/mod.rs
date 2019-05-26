pub mod network_enable;
pub mod network_events;
pub mod set_request_interception;

pub use network_enable::{NetworkEnableTask, NetworkEnableTaskBuilder};
pub use set_request_interception::{SetRequestInterceptionTask, SetRequestInterceptionTaskBuilder};

use crate::browser_async::debug_session::DebugSession;
use crate::browser_async::page_message::{PageResponseWrapper, PageResponse};
use super::super::protocol::{target};
use log::*;

#[derive(Debug)]
pub enum NetworkEvent {
    RequestWillBeSent(network_events::RequestWillBeSent),
    ResponseReceived(network_events::ResponseReceived),
    DataReceived(network_events::DataReceived),
    LoadingFinished(network_events::LoadingFinished),
    RequestIntercepted(network_events::RequestIntercepted),
}

#[allow(clippy::single_match_else)]
pub fn handle_network_event(
    debug_session: &mut DebugSession,
    network_event: NetworkEvent,
    maybe_session_id: Option<target::SessionID>,
    maybe_target_id: Option<target::TargetId>,
) -> Result<PageResponseWrapper, failure::Error> {
    match network_event {
        NetworkEvent::ResponseReceived(event) => {
                let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
                let response_details = event.into_raw_parameters();
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: None,
                    page_response: PageResponse::ResponseReceived(response_details),
                });
        }
        NetworkEvent::RequestIntercepted(event) => {
            warn!("unhandled network_events RequestIntercepted");
        }
        NetworkEvent::RequestWillBeSent(event) => {
            warn!("unhandled network_events RequestWillBeSent");
        }
        NetworkEvent::LoadingFinished(event) => {
            warn!("unhandled network_events LoadingFinished");
        }
        NetworkEvent::DataReceived(event) => {
            warn!("unhandled network_events DataReceived");
        }
    }   
    Ok(PageResponseWrapper::default())
}
