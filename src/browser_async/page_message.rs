use crate::protocol::{dom, page, target, runtime, network};
use crate::browser::tab::element::{BoxModel};
use crate::browser::transport::{SessionId};
use super::id_type as ids;
use std::path::Path;
use std::fs::OpenOptions;
use log::*;
use std::io::{Write};

#[derive(Debug, Clone)]
pub enum ChangingFrame {
    Attached(page::events::FrameAttachedParams),
    StartedLoading(String),
    Navigated(page::Frame),
    StoppedLoading(page::Frame),
}


pub type PageResponseWithTargetIdTaskId = (Option<target::TargetId>, Option<ids::Task>, PageResponse);

// just wait for things happen. don't care who caused happen.
#[derive(Debug)]
pub enum PageResponse {
    ChromeConnected,
    SecondsElapsed(usize),
    PageCreated(Option<String>),
    QuerySelector(String, Option<dom::NodeId>),
    PageAttached(target::TargetInfo, SessionId),
    PageEnable,
    RuntimeEnable,
    FrameAttached(page::types::FrameId),
    FrameStartedLoading(page::types::FrameId),
    FrameNavigated(page::types::FrameId),
    FrameStoppedLoading(page::types::FrameId),
    LoadEventFired(network::types::MonotonicTime),
    PrintToPDF(Option<String>),
    DescribeNode(Option<String>, Option<dom::NodeId>),
    GetBoxModel(Option<String>, Option<Box<BoxModel>>),
    SetChildNodes(dom::NodeId, Vec<dom::Node>),
    GetDocument,
    CaptureScreenshot(response_object::CaptureScreenshot),
    RuntimeEvaluate(Option<Box<runtime::types::RemoteObject>>, Option<Box<runtime::types::ExceptionDetails>>),
    RuntimeExecutionContextCreated(Option<page::types::FrameId>),
    RuntimeGetProperties(Option<runtime::methods::GetPropertiesReturnObject>),
    RuntimeCallFunctionOn(Option<runtime::methods::CallFunctionOnReturnObject>),
    Fail,
}

pub fn write_base64_str_to<P: AsRef<Path>, C: AsRef<str>>(path: P, base64_str: Option<C>) -> std::io::Result<()> {
    if let Some(c) = base64_str {
        let slice = c.as_ref();   
        match base64::decode(slice) {
            Ok(vu8) => {
                let mut file = OpenOptions::new().write(true)
                            .create_new(true)
                            .open(path)?;
                file.write_all(&vu8)?;
            }
            Err(error) => {
                error!("decode failed: {:?}", error);
            }
        }
    } else {
        error!("base64_str is None!");
    }
    Ok(())
}


pub mod response_object {
    use std::path::Path;
    use super::*;

    #[derive(Debug)]
    pub struct CaptureScreenshot {
        pub selector: Option<&'static str>,
        pub base64: Option<String>,
    }

    impl CaptureScreenshot {
        pub fn write_to<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
            write_base64_str_to(path, self.base64.as_ref())
        }
    }
}

// #[derive(Debug)]
// pub enum PageEventName {
//     DomContentEventFired,
//     FrameAttached,
//     FrameDetached,
//     FrameNavigated,
//     InterstitialHidden,
//     InterstitialShown,
//     JavascriptDialogClosed,
//     JavascriptDialogOpening,
//     LifecycleEvent,
//     LoadEventFired,
//     WindowOpen,
// }