use super::element_async::{BoxModel, Element, ElementQuad};
use crate::protocol::{self, dom, page, target};
use super::chrome_debug_session::{ChromeDebugSession};
use super::id_type as ids;
use super::task_describe as tasks;
use super::dev_tools_method_util::{SessionId};
use log::*;
use std::fmt;
use super::tab::Tab;
use std::default::Default;
use std::convert::TryInto;

#[derive(Debug, Clone)]
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

impl Default for ChangingFrameTree {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangingFrameTree {
    pub fn new() -> Self {
        Self {
            changing_frame: None,
            child_changing_frames: ::std::collections::HashMap::new(),
        }
    }

    pub fn count(&self) -> usize {
        self.child_changing_frames.len()
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
    DomContentEventFired,
    FrameAttached,
    FrameDetached,
    FrameNavigated,
    InterstitialHidden,
    InterstitialShown,
    JavascriptDialogClosed,
    JavascriptDialogOpening,
    LifecycleEvent,
    LoadEventFired,
    WindowOpen,
}

pub type PageResponsePlusTabId = (Option<target::TargetId>, PageResponse);

// just wait for things happen. don't care who caused happen.
#[derive(Debug)]
pub enum PageResponse {
    EnablePageDone(String),
    SecondsElapsed(usize),
    PageEvent(PageEventName),
    PageCreated(Option<&'static str>),
    QuerySelector(&'static str, Option<dom::NodeId>),
    PageAttached(target::TargetInfo, SessionId),
    PageEnable,
    TargetInfoChanged(target::TargetInfo),
    FrameNavigated(ChangingFrame),
    NodeIdComing(dom::NodeId, tasks::TaskDescribe),
    NodeComing(dom::Node, tasks::TaskDescribe),
    DescribeNode(target::TargetId, Option<&'static str>, Option<dom::Node>),
    FindElement(Option<&'static str>, Option<Element>),
    GetBoxModel(Option<&'static str>, dom::NodeId, BoxModel),
    SetChildNodes(dom::NodeId, Vec<dom::Node>),
    GetDocument,
    Screenshot(
        Option<&'static str>,
        page::ScreenshotFormat,
        bool,
        Option<Vec<u8>>,
    ),
    MessageAvailable(protocol::Message),
    GetFrameTree(protocol::page::methods::FrameTree),
    Fail,
    Interval,
}