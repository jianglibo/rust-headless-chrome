use crate::protocol;
use crate::protocol::target;
use crate::browser::transport::{SessionId, MethodDestination};
use failure;
use log::*;

use serde_json;
use std::sync::atomic::{AtomicUsize, Ordering};
use super::task_describe::{TaskDescribe};



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
    #[fail(display = "isn't a target oriented method.")]
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

}
