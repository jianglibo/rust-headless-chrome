use super::element_async::{BoxModel, Element, ElementQuad};
use crate::protocol::{self, dom, page, target};
use super::chrome_debug_session::{ChromeDebugSession};
use super::id_type as ids;
use super::task_describe as tasks;
use super::dev_tools_method_util::{SessionId};
use log::*;
use std::fmt;

#[derive(Debug)]
pub enum ChangingFrame {
    Attached(String, String),
    StartLoading(String),
    Navigated(page::Frame),
    StoppedLoading(page::Frame),
}

#[derive(Debug)]
pub struct ChangingFrameTree {
    pub changing_frame: Option<ChangingFrame>,
    pub child_changing_frames: ::std::collections::HashMap<String, ChangingFrame>,
}

impl ChangingFrameTree {
    pub fn new() -> Self {
        Self {
            changing_frame: None,
            child_changing_frames: ::std::collections::HashMap::new(),
        }
    }
}

// pub trait SelectorString {
//      fn get_selector(&self) -> Option<&'static str>;
// }

// pub trait TaskId {
//      fn get_task_id(&self) -> usize;
// }



// impl SelectorString for TaskDescribe {
//     fn get_selector(&self) -> Option<&'static str> {
//         match self {
//             TaskDescribe::QuerySelector(qs) => Some(qs.selector),
//             _ => None,
//         }
//     }
// }

// impl TaskId for TaskDescribe {
//     fn get_task_id(&self) -> usize {
//         match self {
//             TaskDescribe::QuerySelector(qs) => qs.task_id,
//             TaskDescribe::GetDocument(tid) => *tid,
//         }
//     }
// }



#[derive(Debug)]
pub enum PageEventName {
    domContentEventFired,
    frameAttached,
    frameDetached,
    frameNavigated,
    interstitialHidden,
    interstitialShown,
    javascriptDialogClosed,
    javascriptDialogOpening,
    lifecycleEvent,
    loadEventFired,
    windowOpen,
}

// just wait for things happen. don't care who caused happen.
#[derive(Debug)]
pub enum PageMessage {
    EnablePageDone(String),
    PageEvent(PageEventName),
    PageCreated(target::TargetInfo, Option<&'static str>),
    PageAttached(target::TargetInfo, SessionId),
    TargetInfoChanged(target::TargetInfo),
    NodeIdComing(dom::NodeId, tasks::TaskDescribe),
    NodeComing(dom::Node, tasks::TaskDescribe),
    // FindNode(Option<&'static str>, Option<dom::Node>),
    // DomQuerySelector(Option<&'static str>, Option<dom::NodeId>),
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
    // FrameNavigatedEvent(String, String, protocol::page::events::FrameNavigatedEvent),
    GetFrameTree(protocol::page::methods::FrameTree),
    // TargetInfoChanged(protocol::target::events::TargetInfoChangedEvent),
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
