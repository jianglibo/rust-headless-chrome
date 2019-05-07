use crate::protocol;
use crate::protocol::target;
use failure;
use log::*;

use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::task_describe::{TaskDescribe};

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

pub fn next_call_id() -> usize {
    GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, failure::Fail)]
pub enum ChannelBridgeError {
    #[fail(display = "send to error")]
    Sending,
    #[fail(display = "receiving error.")]
    Receiving,
    #[fail(display = "ws error.")]
    Ws(websocket::result::WebSocketError),
}

impl std::convert::From<futures::sync::mpsc::SendError<websocket::message::OwnedMessage>>
    for ChannelBridgeError
{
    fn from(_t: futures::sync::mpsc::SendError<websocket::message::OwnedMessage>) -> Self {
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
    #[fail(display = "next task execution failed.")]
    NextTaskExecution {
        tasks: Vec<TaskDescribe>,
        error: failure::Error,
    },
    #[fail(display = "cannot find tab.")]
    TabNotFound,
}

#[derive(Debug)]
pub struct MethodUtil;

impl MethodUtil {
    pub fn create_msg_to_send_with_session_id<C>(
        method: C,
        session_id: &Option<SessionId>,
        call_id: usize,
    ) -> String
    where
        C: protocol::Method + serde::Serialize,
    {
        if let Some(s_id) = session_id {
            Self::create_msg_to_send(method, MethodDestination::Target(s_id.clone()), call_id)
        } else {
            error!("no session_id exists.");
            panic!("no session_id exists.");
        }
    }


    pub fn create_msg_to_send<C>(
        method: C,
        destination: MethodDestination,
        call_id: usize,
    ) -> String
    where
        C: protocol::Method + serde::Serialize,
    {
        // If call method to target, it will not response with result, instead we will receive a message afterward. with the message id equal to call_id.
        match destination {
            MethodDestination::Target(session_id) => {
                let call = method.to_method_call(call_id);
                let message_text = serde_json::to_string(&call).unwrap();
                let target_method = target::methods::SendMessageToTarget {
                    target_id: None,
                    session_id: Some(session_id.as_str()),
                    message: &message_text,
                };
                Self::create_msg_to_send(target_method, MethodDestination::Browser, GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst))
            }
            MethodDestination::Browser => {
                let call = method.to_method_call(call_id);
                serde_json::to_string(&call).unwrap()
            }
        }
    }
}
