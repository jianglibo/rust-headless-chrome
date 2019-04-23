use crate::protocol;
use crate::protocol::target;
use failure;
use log::*;

use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
pub type MethodTuple = (usize, String, Option<usize>);
pub type MethodBeforSendResult = Result<MethodTuple, failure::Error>;

pub static GLOBAL_METHOD_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SessionId {
    fn from(session_id: String) -> Self {
        Self(session_id)
    }
}

pub enum MethodDestination {
    Target(SessionId),
    Browser,
}

#[derive(Debug, failure::Fail)]
pub enum ChannelBridgeError {
    // #[fail(display = "invalid toolchain name: {}", name)]
    #[fail(display = "send to error")]
    Sending,
    // #[fail(display = "unknown toolchain version: {}", version)]
    #[fail(display = "receiving error.")]
    Receiving,
    #[fail(display = "ws error.")]
    Ws(websocket::result::WebSocketError),
}

impl std::convert::From<futures::sync::mpsc::SendError<websocket::message::OwnedMessage>>
    for ChannelBridgeError
{
    fn from(t: futures::sync::mpsc::SendError<websocket::message::OwnedMessage>) -> Self {
        ChannelBridgeError::Receiving
    }
}

impl std::convert::From<websocket::result::WebSocketError> for ChannelBridgeError {
    fn from(t: websocket::result::WebSocketError) -> Self {
        ChannelBridgeError::Ws(t)
    }
}

#[derive(Debug, failure::Fail)]
pub enum ChromePageError {
    #[fail(display = "page has no target_info.")]
    TargetInfoMissing,
    #[fail(display = "is'nt a target oriented method.")]
    NotTargetOrient,
    #[fail(display = "there is no session.")]
    NoSession,
    #[fail(display = "there is no root node.")]
    NoRootNode,
    #[fail(display = "selector {} return empty result.", selector)]
    QuerySelectorNoResult { selector: &'static str },
    #[fail(display = "I had wait {} seconds.", seconds)]
    WaitTimeout { seconds: usize },
    #[fail(display = "task describe convert to string fail.")]
    TaskDescribeConvert,
}

#[derive(Debug)]
pub struct MethodUtil;

impl MethodUtil {

    // protocol::Response has three field, call_id => id, result, error
    // protocol::Message maybe is an Event, a response, event wrapped a response.
    // pub fn match_chrome_response(
    //     message: protocol::Message,
    //     mid: usize,
    // ) -> Option<protocol::Response> {
    //     if let Some(resp) = Self::get_chrome_response(message) {
    //         if resp.call_id == mid {
    //             trace!("got response, call_id: {:?}", mid);
    //             return Some(resp);
    //         } else {
    //             trace!("waiting for response, call_id: {:?}", mid);
    //         }
    //     } else {
    //         trace!("waiting for response, call_id: {:?}", mid);
    //     }
    //     None
    // }
    // pub fn get_chrome_response(message: protocol::Message) -> Option<protocol::Response> {
    //     match message {
    //         protocol::Message::Response(browser_response) => {
    //             info!("got chrome response. {:?}", browser_response);
    //             return Some(browser_response);
    //         }
    //         protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(
    //             target_message_event,
    //         )) => {
    //             let message_field = &target_message_event.params.message;
    //             if let Ok(protocol::Message::Response(resp)) =
    //                 protocol::parse_raw_message(&message_field)
    //             {
    //                 info!("got message from target response. {:?}", resp);
    //                 return Some(resp);
    //             } else {
    //                 error!("got unknown message0: {:?}", target_message_event);
    //             }

    //         }
    //         other => {
    //             error!("got unknown message1: {:?}", other);
    //         }
    //     }
    //     None
    // }

    // pub fn is_received_message_from_target_event<'a>(
    //     message: &'a protocol::Message,
    // ) -> Option<&'a protocol::target::events::ReceivedMessageFromTargetParams> {
    //     match message {
    //         protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(
    //             target_message_event,
    //         )) => {
    //             return Some(&target_message_event.params);
    //         }
    //         other => {
    //             info!("got ignored event message: {:?}", other);
    //         }
    //     }
    //     None
    // }
    // //{ session_id: "5566C53458FD05F52B70FD9F07336F5D", target_id: "224A448D4698866E1FA56CBBD0455384", message: "{\"method\":\"Page.loadEventFired\",\"params\":{\"timestamp\":345434.504916}}" } }

    // pub fn is_page_load_event_fired<'a>(
    //     message: &'a protocol::Message,
    // ) -> Option<&'a protocol::target::events::ReceivedMessageFromTargetParams> {
    //     if let Some(mg) = Self::is_received_message_from_target_event(&message) {
    //         if let Ok(inner_msg) = Self::parse_target_message_event(&mg.message) {
    //             match inner_msg {
    //                 serde_json::Value::Object(map) => {
    //                     if let Some(serde_json::Value::String(method_name)) = map.get("method") {
    //                         if method_name == "Page.loadEventFired" {
    //                             return Some(&mg);
    //                         }
    //                     }
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    //     None
    // }

    // pub fn parse_target_message_event(
    //     raw_message: &str,
    // ) -> Result<serde_json::Value, failure::Error> {
    //     Ok(serde_json::from_str::<serde_json::Value>(raw_message)?)
    // }

    // pub fn is_page_event_create(
    //     message: protocol::Message,
    // ) -> Option<protocol::target::TargetInfo> {
    //     if let protocol::Message::Event(any_event_from_server) = message {
    //         if let protocol::Event::TargetCreated(target_created_event) = any_event_from_server {
    //             let target_type = &(target_created_event.params.target_info.target_type);
    //             match target_type {
    //                 protocol::target::TargetType::Page => {
    //                     trace!(
    //                         "receive page create event. {:?}",
    //                         target_created_event.params.target_info
    //                     );
    //                     return Some(target_created_event.params.target_info);
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    //     None
    // }

    // pub fn is_page_attach_event(
    //     message: protocol::Message,
    // ) -> Option<(String, protocol::target::TargetInfo)> {
    //     if let protocol::Message::Event(any_event_from_server) = message {
    //         if let protocol::Event::AttachedToTarget(event) = any_event_from_server {
    //             let attach_to_target_params: protocol::target::events::AttachedToTargetParams =
    //                 event.params;
    //             let target_info: protocol::target::TargetInfo = attach_to_target_params.target_info;

    //             match target_info.target_type {
    //                 protocol::target::TargetType::Page => {
    //                     info!(
    //                         "got attach to page event and sessionId: {}",
    //                         attach_to_target_params.session_id
    //                     );
    //                     return Some((attach_to_target_params.session_id, target_info));
    //                 }
    //                 _ => (),
    //             }

    //         }
    //     }

    //     None
    // }
    
    pub fn create_msg_to_send_with_session_id<C>(
        method: C,
        session_id: &Option<SessionId>,
    ) -> MethodBeforSendResult
    where
        C: protocol::Method + serde::Serialize,
    {
        if let Some(s_id) = session_id {
            Self::create_msg_to_send(method, MethodDestination::Target(s_id.clone()), None)
        } else {
            error!("no session_id exists.");
            panic!("no session_id exists.");
        }
    }


    pub fn create_msg_to_send<C>(
        method: C,
        destination: MethodDestination,
        mid: Option<usize>,
    ) -> MethodBeforSendResult
    where
        C: protocol::Method + serde::Serialize,
    {
        let call_id = GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
        let call = method.to_method_call(call_id);
        let message_text = serde_json::to_string(&call).unwrap();

        match destination {
            // If call method to target, it will not response with result, instead we will receive a message afterward. with the message id equal to call_id.
            MethodDestination::Target(session_id) => {
                let target_method = target::methods::SendMessageToTarget {
                    target_id: None,
                    session_id: Some(session_id.as_str()),
                    message: &message_text,
                };
                Self::create_msg_to_send(target_method, MethodDestination::Browser, Some(call_id))
            }
            MethodDestination::Browser => {
                Ok((call_id, message_text, mid))
            }
        }
    }
}
