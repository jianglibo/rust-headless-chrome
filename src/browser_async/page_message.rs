use crate::protocol::{dom, page, target};
use crate::browser::tab::element::{BoxModel};
use super::id_type as ids;
use super::dev_tools_method_util::{SessionId};

#[derive(Debug, Clone)]
pub enum ChangingFrame {
    Attached(String, String),
    StartLoading(String),
    Navigated(page::Frame),
    StoppedLoading(page::Frame),
}

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
    ChromeConnected,
    SecondsElapsed(usize),
    PageCreated(Option<&'static str>),
    QuerySelector(&'static str, Option<dom::NodeId>),
    PageAttached(target::TargetInfo, SessionId),
    PageEnable,
    FrameNavigated(ChangingFrame),
    LoadEventFired(f32),
    DescribeNode(Option<&'static str>, Option<dom::NodeId>),
    GetBoxModel(Option<&'static str>, Option<Box<BoxModel>>),
    SetChildNodes(dom::NodeId, Vec<dom::Node>),
    GetDocument,
    Screenshot(response_object::CaptureScreenshot),
    Fail,
    // MessageAvailable(protocol::Message),
    // GetFrameTree(protocol::page::methods::FrameTree),
    // TargetInfoChanged(target::TargetInfo),
    // PageEvent(PageEventName),
    // EnablePageDone(String),
    // Interval,
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