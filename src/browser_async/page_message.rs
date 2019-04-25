// use super::element_async::{BoxModel, Element, ElementQuad};
use crate::protocol::{self, dom, page, target};
use crate::browser::tab::element::{BoxModel};
use super::id_type as ids;
use super::dev_tools_method_util::{SessionId};
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

// #[derive(Debug)]
// pub struct ChangingFrameTree {
//     pub changing_frame: Option<ChangingFrame>,
//     pub child_changing_frames: ::std::collections::HashMap<String, ChangingFrame>,
// }

// impl Default for ChangingFrameTree {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl ChangingFrameTree {
//     pub fn new() -> Self {
//         Self {
//             changing_frame: None,
//             child_changing_frames: ::std::collections::HashMap::new(),
//         }
//     }

//     pub fn count(&self) -> usize {
//         self.child_changing_frames.len()
//     }
// }

// struct SearchVisitor<'a, F> {
//     predicate: F,
//     item: Option<&'a mut ChangingFrame>,
// }

// impl<'a, F: FnMut(&ChangingFrame) -> bool> SearchVisitor<'a, F> {
//     fn new(predicate: F) -> Self {
//         SearchVisitor {
//             predicate,
//             item: None,
//         }
//     }

//     fn visit(&mut self, n: &'a ChangingFrame) {
//         if (self.predicate)(n) {
//             self.item = Some(n);
//         } else if self.item.is_none() {
//             if let Some(c) = &n.child_changing_frames {
//                 c.iter().for_each(|n| self.visit(n))
//             }
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

pub type PageResponsePlusTabId = (Option<target::TargetId>, Option<ids::Task>, PageResponse);

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
    LoadEventFired(f32),
    DescribeNode(Option<&'static str>, Option<dom::NodeId>),
    GetBoxModel(Option<&'static str>, Option<BoxModel>),
    SetChildNodes(dom::NodeId, Vec<dom::Node>),
    GetDocument,
    Screenshot(response_object::CaptureScreenshot),
    MessageAvailable(protocol::Message),
    GetFrameTree(protocol::page::methods::FrameTree),
    Fail,
    Interval,
}

pub mod response_object {
    use std::path::Path;
    use std::fs::OpenOptions;
    use log::*;
    use std::io::{Write};

    #[derive(Debug)]
    pub struct CaptureScreenshot {
        pub selector: Option<&'static str>,
        pub base64: Option<String>,
    }

    impl CaptureScreenshot {
        pub fn write_to<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
            if let Some(base64_str) = &self.base64 {
                if let Ok(vu8) = base64::decode(base64_str) {
                   let mut file = OpenOptions::new().write(true)
                             .create_new(true)
                             .open(path)?;
                   file.write_all(&vu8)?;
                }
            } else {
                error!("decode base64 failed.");
            }
            Ok(())
        }
    }
}