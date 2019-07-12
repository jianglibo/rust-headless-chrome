use crate::protocol::{dom, page, target, network};
use super::super::browser_async::{TaskId};
use super::super::browser_async::task_describe::{network_tasks, network_events, runtime_tasks, runtime_events, dom_tasks, page_tasks, page_events, emulation_tasks};
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


#[derive(Debug)]
pub struct PageResponseWrapper {
    pub target_id: Option<target::TargetId>,
    pub task_id: Option<TaskId>,
    pub page_response: PageResponse,
}

impl PageResponseWrapper {
    pub fn new(page_response: PageResponse) -> Self {
        Self {
            target_id: None,
            task_id: None,
            page_response,
        }
    }
}

impl std::default::Default for PageResponseWrapper {
    fn default() -> Self { 
        Self {
            target_id: None,
            task_id: None,
            page_response: PageResponse::Fail,
        }
    }
}

// pub type PageResponseWithTargetIdTaskId = (Option<target::TargetId>, Option<TaskId>, PageResponse);

#[derive(Debug)]
pub enum ReceivedEvent {
    PageCreated(usize),
    PageAttached(target::TargetInfo, target::SessionID),
    FrameAttached(page::FrameId),
    FrameStartedLoading(page::FrameId),
    FrameNavigated(page_events::FrameNavigated),
    FrameStoppedLoading(page::FrameId),
    LoadEventFired(network::MonotonicTime),
    SetChildNodesOccurred(dom::NodeId),
    ExecutionContextCreated(runtime_events::ExecutionContextCreated),
    ResponseReceived(network::RequestId),
    RequestIntercepted(network::RequestId),
    RequestWillBeSent(network::RequestId),
    LoadingFinished(network_events::LoadingFinished),
    DataReceived(network_events::DataReceived),
    LoadingFailed(network_events::LoadingFailed),
}

#[derive(Debug)]
pub enum MethodCallDone {
    PageEnabled(page_tasks::PageEnableTask),
    RuntimeEnabled(runtime_tasks::RuntimeEnableTask),
    QuerySelector(dom_tasks::QuerySelectorTask),
    PrintToPdf(page_tasks::PrintToPdfTask),
    GetLayoutMetrics(page_tasks::GetLayoutMetricsTask),
    BringToFront(page_tasks::BringToFrontTask),
    DescribeNode(dom_tasks::DescribeNodeTask),
    GetBoxModel(dom_tasks::GetBoxModelTask),
    GetContentQuads(dom_tasks::GetContentQuadsTask),
    GetDocument(dom_tasks::GetDocumentTask),
    CaptureScreenshot(page_tasks::CaptureScreenshotTask),
    Evaluate(runtime_tasks::RuntimeEvaluateTask),
    GetProperties(runtime_tasks::RuntimeGetPropertiesTask),
    CallFunctionOn(runtime_tasks::RuntimeCallFunctionOnTask),
    SetIgnoreCertificateErrors(bool),
    GetResponseBodyForInterception(network_tasks::GetResponseBodyForInterceptionTask),
    TargetAttached(page_tasks::AttachToTargetTask),
    CanEmulate(emulation_tasks::CanEmulateTask),
    SetDeviceMetricsOverride(emulation_tasks::SetDeviceMetricsOverrideTask),
}

// just wait for things happen. don't care who caused happen.
#[derive(Debug)]
pub enum PageResponse {
    ChromeConnected,
    SecondsElapsed(usize),
    ReceivedEvent(ReceivedEvent),
    MethodCallDone(MethodCallDone),
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