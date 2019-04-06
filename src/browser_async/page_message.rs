use super::element_async::{BoxModel, Element, ElementQuad};
use crate::protocol::{self, dom, page};
use log::*;
use std::fmt;

#[derive(Debug)]
pub enum PageMessage {
    NavigatingToTarget,
    DocumentAvailable,
    EnablePageDone,
    // FindNode(Option<&'static str>, Option<dom::Node>),
    DomQuerySelector(Option<&'static str>, Option<dom::NodeId>),
    DomDescribeNode(Option<&'static str>, Option<dom::Node>),
    FindElement(Option<&'static str>, Option<Element>),
    GetBoxModel(Option<&'static str>, dom::NodeId, BoxModel),
    Screenshot(
        Option<&'static str>,
        page::ScreenshotFormat,
        bool,
        Option<Vec<u8>>,
    ),
    MessageAvailable(protocol::Message),
    FrameNavigatedEvent(String, String, protocol::page::events::FrameNavigatedEvent),
    GetFrameTree(protocol::page::methods::FrameTree),
    TargetInfoChanged(protocol::target::events::TargetInfoChangedEvent),
    Interval,
    SecondsElapsed(usize),
}

// impl fmt::Debug for PageMessage {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         info!("----------------enter fmt---------------------------");
//         match self {
//             PageMessage::FindElement(selector, ele) => {
//                 let a = selector.map_or("None", |v| v);
//                 if let Some(el) = ele {
//                     write!(
//                         f,
//                         "selector: {}, remote_object_id: {}, backend_node_id: {}",
//                         a, el.remote_object_id, el.backend_node_id
//                     )
//                 } else {
//                     write!(f, "selector: {}, None", a)
//                 }
//             }
//             _ => write!(f, "{}", self),
//         }
//     }
// }
