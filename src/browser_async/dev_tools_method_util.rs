use crate::protocol;
use crate::protocol::target;
use failure;
use log::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use serde_json;

pub type MethodBeforSendResult = Result<(usize, String, Option<usize>), failure::Error>;

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

impl std::convert::From<websocket::result::WebSocketError>
    for ChannelBridgeError
{
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
    QuerySelectorNoResult {
        selector: &'static str
    },
}

#[derive(Debug)]
pub struct MethodUtil;

impl MethodUtil {

    // protocol::Response has three field, call_id => id, result, error
    // protocol::Message maybe is an Event, a response, event wrapped a response.
    pub fn match_chrome_response(message: protocol::Message, mid: &usize) -> Option<protocol::Response> {
        if let Some(resp) = Self::get_chrome_response(message) {
            if &resp.call_id == mid {
                trace!("got response, call_id: {:?}", mid);
                return Some(resp);
            } else {
                trace!("waiting for response, call_id: {:?}", mid);
            }
        } else {
            trace!("waiting for response, call_id: {:?}", mid);
        }
        None
    }
    pub fn get_chrome_response(message: protocol::Message) -> Option<protocol::Response> {
            match message {
                protocol::Message::Response(browser_response) => {
                    info!("got chrome response. {:?}", browser_response);
                    return Some(browser_response);
                }
                protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(
                    target_message_event,
                )) => {
                    let message_field = &target_message_event.params.message;
                    if let Ok(protocol::Message::Response(resp)) =
                        protocol::parse_raw_message(&message_field)
                    {
                        info!("got message from target response. {:?}", resp);
                        return Some(resp);
                    } else {
                        error!("got unknown message0: {:?}", target_message_event);
                    }
                }
                other => {
                    error!("got unknown message1: {:?}", other);
                }
            }
            None
    }

    pub fn is_received_message_from_target_event<'a>(message: &'a protocol::Message) -> Option<&'a protocol::target::events::ReceivedMessageFromTargetParams> {
            match message {
                protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(
                    target_message_event,
                )) => {
                    return Some(&target_message_event.params);
                }
                other => {
                    info!("got ignored event message: {:?}", other);
                }
            }
            None
    }
    //{ session_id: "5566C53458FD05F52B70FD9F07336F5D", target_id: "224A448D4698866E1FA56CBBD0455384", message: "{\"method\":\"Page.loadEventFired\",\"params\":{\"timestamp\":345434.504916}}" } }

    pub fn is_page_load_event_fired<'a>(message: &'a protocol::Message) -> Option<&'a protocol::target::events::ReceivedMessageFromTargetParams> {
        if let Some(mg) = Self::is_received_message_from_target_event(&message) {
            if let Ok(inner_msg) = Self::parse_target_message_event(&mg.message) {
                match inner_msg {
                    serde_json::Value::Object(map) => {
                        if let Some(serde_json::Value::String(method_name)) = map.get("method") {
                            if method_name == "Page.loadEventFired" {
                                return Some(&mg);
                            }
                        }
                    }
                    _ => ()
                }
            }
        }
        None
    }

    pub fn parse_target_message_event(raw_message: &str) -> Result<serde_json::Value, failure::Error> {
        Ok(serde_json::from_str::<serde_json::Value>(raw_message)?)
    }

    pub fn is_page_event_create(message: protocol::Message) -> Option<protocol::target::TargetInfo> {
        if let protocol::Message::Event(any_event_from_server) = message
        {
            if let protocol::Event::TargetCreated(target_created_event) = any_event_from_server {
                let target_type = &(target_created_event.params.target_info.target_type);
                match target_type {
                    protocol::target::TargetType::Page => {
                        trace!(
                            "receive page create event. {:?}",
                            target_created_event.params.target_info
                        );
                        return Some(target_created_event.params.target_info);
                    }
                    _ => (),
                }
            }
        }
        None
    }

    pub fn is_page_attach_event(message: protocol::Message) -> Option<(String, protocol::target::TargetInfo)> {
                if let protocol::Message::Event(any_event_from_server) = message
        {

        if let protocol::Event::AttachedToTarget(event) = any_event_from_server
        {
            let attach_to_target_params: protocol::target::events::AttachedToTargetParams =
                event.params;
            let target_info: protocol::target::TargetInfo = attach_to_target_params.target_info;

            match target_info.target_type {
                protocol::target::TargetType::Page => {
                    info!(
                        "got attach to page event and sessionId: {}",
                        attach_to_target_params.session_id
                    );
                    return Some((attach_to_target_params.session_id, target_info));
                }
                _ => (),
            }
        }
        }
        None
    }

    // if you take self, you consume youself.
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
                info!("sending method call_id: {:?}, {:?}", call_id, message_text);
                Ok((call_id, message_text, mid))
            }
        }
    }
}

    // pub fn get_chrome_event(owned_message: &OwnedMessage) -> Option<protocol::Event> {
    //     if let Some(protocol::Message::Event(browser_event)) =
    //         Self::get_any_message_from_chrome(owned_message)
    //     {
    //         info!("parsed chrome message: {:?}", browser_event);
    //         match browser_event {
    //             protocol::Event::ReceivedMessageFromTarget(target_message_event) => {
    //                 let session_id: SessionId = target_message_event.params.session_id.into();
    //                 let raw_message = target_message_event.params.message;

    //                 if let Ok(target_message) = protocol::parse_raw_message(&raw_message) {
    //                     match target_message {
    //                         protocol::Message::Event(target_event) => {
    //                             info!("get event {:?}", target_event);
    //                             return Some(target_event);
    //                         }
    //                         protocol::Message::Response(resp) => {
    //                             return None;
    //                         }
    //                         protocol::Message::ConnectionShutdown => None,
    //                     }
    //                 } else {
    //                     trace!(
    //                         "Message from target isn't recognised: {:?}",
    //                         &raw_message[..30]
    //                     );
    //                     return None;
    //                 }
    //             }
    //             _ => Some(browser_event),
    //         }
    //     } else {
    //         None
    //     }
    // }
    // protocol::Message cover all possible messages from chrome.
    // #[allow(clippy::single_match_else)]
    // fn get_any_message_from_chrome(owned_message: &OwnedMessage) -> Option<protocol::Message> {
    //     match owned_message {
    //         OwnedMessage::Text(msg) => {
    //             if let Ok(m) = protocol::parse_raw_message(&msg) {
    //                 trace!("got protocol message catch all: {:?}", msg);
    //                 return Some(m);
    //             } else {
    //                 error!("got unparsable message from chrome. {}", msg);
    //             }
    //         }
    //         _ => {
    //             error!("got None text message from chrome. {:?}", owned_message);
    //             ()
    //         }
    //     };
    //     None
    // }



// #[derive(Debug)]
// pub struct ChromePage {
//     pub page_target_info: Option<protocol::target::TargetInfo>,
//     pub waiting_call_id: Option<usize>, // this is direct browser response
//     pub waiting_message_id: Option<usize>, // this is message from target, but is response to user request.
//     pub session_id: Option<String>,
//     pub root_node: Option<dom::Node>,
// }

// impl<'a> ChromePage {
//     pub fn is_page_event_create(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
//         if let Some(protocol::Message::Event(any_event_from_server)) =
//             MethodUtil::get_any_message_from_chrome(owned_message)
//         {
//             if let protocol::Event::TargetCreated(target_created_event) = any_event_from_server {
//                 let target_type = &(target_created_event.params.target_info.target_type);
//                 match target_type {
//                     protocol::target::TargetType::Page => {
//                         trace!(
//                             "receive page create event. {:?}",
//                             target_created_event.params.target_info
//                         );
//                         self.page_target_info = Some(target_created_event.params.target_info);
//                         return Ok(false);
//                     }
//                     _ => (),
//                 }
//             }
//         }
//         Ok(true)
//     }

//     // when got message {\"method\":\"Target.receivedMessageFromTarget\" from chrome, it has a params field, which has a 'message' field,
//     // it's the response to your early method call.
//     pub fn get_document() -> Option<protocol::Response> {
//         None
//     }

//     fn create_msg_to_send<C>(
//         &mut self,
//         method: C,
//         destination: MethodDestination,
//     ) -> MethodBeforSendResult
//     where
//         C: protocol::Method + serde::Serialize,
//     {
//         MethodUtil::create_msg_to_send(method, destination, None)
//     }

//     pub fn create_attach_method(&mut self) -> MethodBeforSendResult {
//         let mut target_id: Option<String> = None;
//         if let Some(ti) = &mut self.page_target_info {
//             target_id = Some(ti.target_id.clone());
//         }

//         if let Some(ti) = target_id {
//             self.create_msg_to_send(
//                 target::methods::AttachToTarget {
//                     target_id: &ti,
//                     flatten: None,
//                 },
//                 MethodDestination::Browser,
//             )
//         } else {
//             Err(ChromePageError::TargetInfoMissing.into())
//         }
//     }

//     fn create_msg_to_send_with_session_id<C>(&mut self, method: C) -> MethodBeforSendResult
//     where
//         C: protocol::Method + serde::Serialize,
//     {
//         if let Some(session_id) = &self.session_id {
//             self.create_msg_to_send(method, MethodDestination::Target(session_id.clone().into()))
//         } else {
//             Err(ChromePageError::NoSession.into())
//         }
//     }

//     pub fn query_document_method(&mut self) -> MethodBeforSendResult {
//         self.create_msg_to_send_with_session_id(dom::methods::GetDocument {
//             depth: Some(0),
//             pierce: Some(false),
//         })
//     }

//     pub fn find_node_method(&'a mut self, selector: &'a str) -> MethodBeforSendResult {
//         if let Some(rn) = &self.root_node {
//             self.create_msg_to_send_with_session_id(dom::methods::QuerySelector {
//                 node_id: rn.node_id,
//                 selector,
//             })
//         } else {
//             Err(ChromePageError::NoRootNode.into())
//         }
//     }

//     pub fn enable_discover_method(&mut self) -> MethodBeforSendResult {
//         self.create_msg_to_send(
//             target::methods::SetDiscoverTargets { discover: true },
//             MethodDestination::Browser,
//         )
//     }

//     pub fn enable_page_notifications(&mut self) -> MethodBeforSendResult {
//         self.create_msg_to_send(page::methods::Enable {}, MethodDestination::Browser)
//     }

//     pub fn match_response_by_call_id(
//         &self,
//         owned_message: &OwnedMessage,
//         call_id: usize,
//     ) -> Option<protocol::Response> {
//         if let Some(response) = MethodUtil::get_chrome_response(owned_message) {
//             if response.call_id == call_id {
//                 return Some(response);
//             }
//         }
//         None
//     }

//     pub fn match_document_by_call_id(
//         &self,
//         owned_message: &OwnedMessage,
//         call_id: usize,
//     ) -> Option<dom::Node> {
//         if let Some(response) = self.match_response_by_call_id(owned_message, call_id) {
//             if let Ok(c) =
//                 protocol::parse_response::<dom::methods::GetDocumentReturnObject>(response)
//             {
//                 info!("got document Node: {:?}", c.root);
//                 return Some(c.root);
//             }
//         }
//         None
//     }

//     pub fn match_query_selector_by_call_id(
//         &self,
//         owned_message: &OwnedMessage,
//         call_id: usize,
//     ) -> Option<u16> {
//         if let Some(response) = self.match_response_by_call_id(owned_message, call_id) {
//             if let Ok(c) =
//                 protocol::parse_response::<dom::methods::QuerySelectorReturnObject>(response)
//             {
//                 info!("got query selector Node: {:?}", c);
//                 return Some(c.node_id);
//             }
//         }
//         None
//     }

//     pub fn match_waiting_call_response(
//         &self,
//         owned_message: &OwnedMessage,
//     ) -> Option<protocol::Response> {
//         if let Some(response) = MethodUtil::get_chrome_response(owned_message) {
//             if Some(response.call_id) == self.waiting_call_id {
//                 return Some(response);
//             } else {
//                 info!(
//                     "got response with call_id: {}, but waiting call_id is: {:?}",
//                     response.call_id, self.waiting_call_id
//                 );
//             }
//         }
//         None
//     }

//     // return Ok(true) if keeping skip.
//     pub fn is_response_for_attach_page(
//         &mut self,
//         owned_message: &OwnedMessage,
//     ) -> Result<bool, ()> {
//         if let Some(response) = self.match_waiting_call_response(owned_message) {
//             if let Some(serde_json::value::Value::Object(value)) = response.result {
//                 if let Some(serde_json::value::Value::String(session_id)) = value.get("sessionId") {
//                     self.session_id = Some(session_id.clone());
//                     return Ok(false);
//                 }
//             }
//         }
//         Ok(true)
//     }

//     pub fn is_page_attach_event(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
//         if let Some(protocol::Event::AttachedToTarget(event)) =
//             MethodUtil::get_chrome_event(owned_message)
//         {
//             let attach_to_target_params: protocol::target::events::AttachedToTargetParams =
//                 event.params;
//             let target_info: protocol::target::TargetInfo = attach_to_target_params.target_info;

//             match target_info.target_type {
//                 protocol::target::TargetType::Page => {
//                     info!(
//                         "got attach to page event and sessionId: {}",
//                         attach_to_target_params.session_id
//                     );
//                     self.session_id = Some(attach_to_target_params.session_id);
//                     self.page_target_info = Some(target_info);
//                     return Ok(false);
//                 }
//                 _ => (),
//             }
//         }
//         Ok(true)
//     }

//     pub fn navigate_to(&mut self, url: &str) -> MethodBeforSendResult {
//         let c = Navigate { url };
//         let md = MethodDestination::Target(self.session_id.clone().unwrap().into());
//         MethodUtil::create_msg_to_send(c, md, None)
//     }

//     pub fn is_page_url_changed(&mut self, owned_message: &OwnedMessage) -> Result<bool, ()> {
//         if let Some(protocol::Event::TargetInfoChanged(event)) =
//             MethodUtil::get_chrome_event(owned_message)
//         {
//             let target_info: protocol::target::TargetInfo = event.params.target_info;
//             if let Some(self_ti) = &self.page_target_info {
//                 if (self_ti.target_id == target_info.target_id) && (target_info.url != self_ti.url)
//                 {
//                     info!(
//                         "got same target_id: {}, type: {:?}, url: {}",
//                         self_ti.target_id, target_info.target_type, target_info.url
//                     );
//                     self.page_target_info = Some(target_info);
//                     return Ok(false);
//                 } else {
//                     info!(
//                         "got different target_id1: {}, target_id2: {}",
//                         self_ti.target_id, target_info.target_id
//                     );
//                 }
//             }
//         }
//         Ok(true)
//     }
// }
