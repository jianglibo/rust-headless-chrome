pub mod continue_intercepted_request;
pub mod get_response_body_for_interception;
pub mod network_events;
pub mod set_request_interception;
pub mod network_enable;


use crate::browser_async::debug_session::DebugSession;
pub use continue_intercepted_request::{
    ContinueInterceptedRequestTask, ContinueInterceptedRequestTaskBuilder,
};
pub use get_response_body_for_interception::{
    GetResponseBodyForInterceptionTask, GetResponseBodyForInterceptionTaskBuilder,
};

pub use network_enable::{NetworkEnableTask, NetworkEnableTaskBuilder};
pub use set_request_interception::{SetRequestInterceptionTask, SetRequestInterceptionTaskBuilder};

use super::super::protocol::target;
use crate::browser_async::page_message::{PageResponse, PageResponseWrapper, ReceivedEvent};
use log::*;

#[derive(Debug)]
pub enum NetworkEvent {
    RequestWillBeSent(network_events::RequestWillBeSent),
    ResponseReceived(network_events::ResponseReceived),
    DataReceived(network_events::DataReceived),
    LoadingFinished(network_events::LoadingFinished),
    RequestIntercepted(network_events::RequestIntercepted),
    LoadingFailed(network_events::LoadingFailed),
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
            let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
            let request_id = event.get_request_id();
            tab.response_received.insert(request_id.clone(), event);
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: None,
                page_response: PageResponse::ReceivedEvent(ReceivedEvent::ResponseReceived(
                    request_id,
                )),
            });
        }
        NetworkEvent::RequestIntercepted(event) => {
            let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
            let request_id = event.get_interception_id();
            tab.request_intercepted.insert(request_id.clone(), event);
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: None,
                page_response: PageResponse::ReceivedEvent(ReceivedEvent::RequestIntercepted(
                    request_id,
                )),
            });
            // warn!("unhandled network_events RequestIntercepted");
        }
        NetworkEvent::RequestWillBeSent(event) => {
            let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
            let request_id = event.get_request_id();
            tab.request_will_be_sent(event);
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: None,
                page_response: PageResponse::ReceivedEvent(ReceivedEvent::RequestWillBeSent(
                    request_id,
                )),
            });
            // warn!("unhandled network_events RequestWillBeSent");
        }
        NetworkEvent::LoadingFinished(event) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: None,
                page_response: PageResponse::ReceivedEvent(ReceivedEvent::LoadingFinished(event)),
            });
            // warn!("unhandled network_events LoadingFinished");
        }
        NetworkEvent::DataReceived(event) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: None,
                page_response: PageResponse::ReceivedEvent(ReceivedEvent::DataReceived(event)),
            });
            // warn!("unhandled network_events DataReceived");
        }
        NetworkEvent::LoadingFailed(event) => {
            let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
            let request = tab.take_request(&event.get_request_id());
            error!("failed request: {:?}", request);
            error!("failed request event: {:?}", event);
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: None,
                page_response: PageResponse::ReceivedEvent(ReceivedEvent::LoadingFailed(event)),
            });
            // warn!("unhandled network_events DataReceived");
        }
    }
    Ok(PageResponseWrapper::default())
}
