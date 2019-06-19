#[macro_use]
pub mod sukosi_macro;

pub mod chrome_browser;
pub mod chrome_debug_session;
pub mod interval_page_message;
pub mod debug_session;
pub mod page_message;
pub mod json_assistor;
pub mod task_describe;
pub mod tab;
pub mod embedded_events;
pub mod event_statistics;
pub mod task_queue;
pub mod browser_context;

pub use event_statistics::{EventStatistics, EventName};
pub use chrome_debug_session::{ChromeDebugSession};
pub use task_queue::{TaskQueue};
pub use debug_session::{DebugSession};

use failure;
use task_describe::{self as tasks, TaskDescribe};
use crate::protocol::{self, target};
use std::sync::atomic::{AtomicUsize, Ordering};
use log::*;

pub use tab::{Tab, WaitingForPageAttachTaskName};
pub use browser_context::{BrowserContext, BrowserContexts};

pub type TaskId = String;

pub enum MethodDestination {
    Target(target::SessionID),
    Browser,
}

static METHOD_ID_ABOVE_10000: AtomicUsize = AtomicUsize::new(10001);

pub fn create_unique_if_no_manual_input(manual: Option<usize>) -> (usize, bool) {
    manual.map_or_else(||(METHOD_ID_ABOVE_10000.fetch_add(1, Ordering::SeqCst), false),|mid|(mid, true))
}

pub fn create_unique_usize() -> usize {
    METHOD_ID_ABOVE_10000.fetch_add(1, Ordering::SeqCst)
}

pub fn create_unique_task_id() -> String {
    let u = create_unique_usize();
    format!("task-id-{}", u)
}

pub fn create_unique_prefixed_id(prefix: &str) -> String {
    let u = create_unique_usize();
    format!("{}{}", prefix, u)
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


pub static GLOBAL_METHOD_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn next_call_id() -> protocol::CallId {
    GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst)
}
 pub fn create_msg_to_send_with_session_id<C>(
        method: C,
        session_id: Option<&target::SessionID>,
        call_id: usize,
    ) -> String
    where
        C: protocol::Method + serde::Serialize,
    {
        if let Some(s_id) = session_id {
            create_msg_to_send(method, MethodDestination::Target(s_id.clone()), call_id)
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
                let message_text = serde_json::to_string(&call).expect("message_text should created.");
                let target_method = target::methods::SendMessageToTarget {
                    target_id: None,
                    session_id: Some(session_id.as_str()),
                    message: &message_text,
                };
                create_msg_to_send(target_method, MethodDestination::Browser, GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst))
            }
            MethodDestination::Browser => {
                let call = method.to_method_call(call_id);
                serde_json::to_string(&call).expect("message_text should created.")
            }
        }
    }

// pub trait CreateMethodCallString {
//     fn create_method_call_string(&self, session_id: Option<&target::SessionID>, call_id: usize) -> String;
// }


pub fn get_common_fields_by_task_id(manual_task_id: Option<TaskId>) -> tasks::CommonDescribeFields {
        tasks::CommonDescribeFieldsBuilder::default()
            .task_id(manual_task_id)
            .build()
            .expect("build common_fields should success.")
    }