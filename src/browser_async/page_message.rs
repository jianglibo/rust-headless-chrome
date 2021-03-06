use super::task_describe::{
    browser_tasks, dom_tasks, emulation_tasks, network_events, network_tasks, page_events,
    page_tasks, runtime_events, runtime_tasks, target_tasks,
};
use super::TaskId;
use crate::protocol::{dom, network, page, target};
use log::*;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

// TODO main frame may not fire FrameAttached.
#[derive(Debug, Clone)]
pub enum ChangingFrame {
    Attached(page::events::FrameAttachedParams),
    StartedLoading(String),
    Navigated(page::Frame),
    StoppedLoading(page::Frame),
    StopLoadingFrameId,
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
    PageCreated,
    PageAttached(target::TargetInfo, target::SessionID),
    LifeCycle,
    FrameAttached(page::FrameId),
    FrameStartedLoading(page::FrameId),
    FrameNavigated(page_events::FrameNavigated),
    WindowOpen(page_events::WindowOpen),
    FrameRequestedNavigation(page_events::FrameRequestedNavigation),
    FrameResized(page_events::FrameResized),
    FrameStoppedLoading(page::FrameId),
    LoadEventFired(network::MonotonicTime),
    SetChildNodesOccurred(dom::NodeId),
    ExecutionContextCreated(runtime_events::ExecutionContextCreated),
    ResponseReceived(network::RequestId),
    RequestIntercepted(network::RequestId),
    RequestWillBeSent(network::RequestId),
    LoadingFinished(network_events::LoadingFinished),
    DataReceived(network_events::DataReceived),
    LoadingFailed(network::RequestId),
    ResourceChangedPriority(network_events::ResourceChangedPriority),
    RequestServedFromCache(network_events::RequestServedFromCache),
}

#[derive(Debug)]
pub enum MethodCallDone {
    PageEnabled(page_tasks::PageEnableTask),
    PageClosed(bool), // running its beforeunload hooks by Page.close or close by Target.closeTarget.
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
    Evaluate(runtime_tasks::EvaluateTask),
    GetProperties(runtime_tasks::GetPropertiesTask),
    CallFunctionOn(runtime_tasks::CallFunctionOnTask),
    SetIgnoreCertificateErrors(bool),
    GetResponseBodyForInterception(network_tasks::GetResponseBodyForInterceptionTask),
    TargetAttached(page_tasks::AttachToTargetTask),
    CanEmulate(emulation_tasks::CanEmulateTask),
    SetDeviceMetricsOverride(emulation_tasks::SetDeviceMetricsOverrideTask),
    GetTargets(target_tasks::GetTargetsTask),
    GetBrowserCommandLine(browser_tasks::GetBrowserCommandLineTask),
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

pub fn write_base64_str_to<P: AsRef<Path>, C: AsRef<str>>(
    path: P,
    base64_str: Option<C>,
) -> std::io::Result<()> {
    if let Some(c) = base64_str {
        let slice = c.as_ref();
        match base64::decode(slice) {
            Ok(vu8) => {
                let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
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
    use super::*;
    use std::path::Path;

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
