use crate::browser::transport::{MethodDestination, SessionId};
use crate::protocol::{self, target};
use std::sync::atomic::{AtomicUsize, Ordering};
use log::*;

pub static GLOBAL_METHOD_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn next_call_id() -> usize {
    GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst)
}
 pub fn create_msg_to_send_with_session_id<C>(
        method: C,
        session_id: Option<&SessionId>,
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
                let message_text = serde_json::to_string(&call).unwrap();
                let target_method = target::methods::SendMessageToTarget {
                    target_id: None,
                    session_id: Some(session_id.as_str()),
                    message: &message_text,
                };
                create_msg_to_send(target_method, MethodDestination::Browser, GLOBAL_METHOD_CALL_COUNT.fetch_add(1, Ordering::SeqCst))
            }
            MethodDestination::Browser => {
                let call = method.to_method_call(call_id);
                serde_json::to_string(&call).unwrap()
            }
        }
    }

pub trait CreateMethodCallString {
    fn create_method_call_string(&self, session_id: Option<&SessionId>, call_id: usize) -> String;

}