use super::{dom_tasks, network_tasks, page_tasks,runtime_tasks, input_tasks, emulation_tasks};

use super::super::debug_session::DebugSession;
use super::super::page_message::{PageResponse, PageResponseWrapper, MethodCallDone};
use crate::protocol::target;
use log::*;

use super::{HasCallId, HasTaskId};

#[derive(Debug)]
pub enum TargetCallMethodTask {
    NavigateTo(page_tasks::NavigateToTask),
    QuerySelector(dom_tasks::QuerySelectorTask),
    DescribeNode(dom_tasks::DescribeNodeTask),
    PrintToPDF(page_tasks::PrintToPdfTask),
    GetBoxModel(dom_tasks::GetBoxModelTask),
    GetContentQuads(dom_tasks::GetContentQuadsTask),
    GetDocument(dom_tasks::GetDocumentTask),
    PageEnable(page_tasks::PageEnableTask),
    PageClose(page_tasks::PageCloseTask),
    GetLayoutMetrics(page_tasks::GetLayoutMetricsTask),
    BringToFront(page_tasks::BringToFrontTask),
    RuntimeEnable(runtime_tasks::RuntimeEnableTask),
    CaptureScreenshot(page_tasks::CaptureScreenshotTask),
    RuntimeEvaluate(runtime_tasks::RuntimeEvaluateTask),
    RuntimeGetProperties(runtime_tasks::RuntimeGetPropertiesTask),
    RuntimeCallFunctionOn(runtime_tasks::RuntimeCallFunctionOnTask),
    NetworkEnable(network_tasks::NetworkEnableTask),
    SetRequestInterception(network_tasks::SetRequestInterceptionTask),
    ContinueInterceptedRequest(network_tasks::ContinueInterceptedRequestTask),
    GetResponseBodyForInterception(network_tasks::GetResponseBodyForInterceptionTask),
    PageReload(page_tasks::PageReloadTask),
    DispatchMouseEvent(input_tasks::DispatchMouseEventTask),
    CanEmulate(emulation_tasks::CanEmulateTask),
    SetDeviceMetricsOverride(emulation_tasks::SetDeviceMetricsOverrideTask),
}

impl HasCallId for TargetCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            TargetCallMethodTask::NavigateTo(task) => task.get_call_id(),
            TargetCallMethodTask::QuerySelector(task) => task.get_call_id(),
            TargetCallMethodTask::DescribeNode(task) => task.get_call_id(),
            TargetCallMethodTask::PrintToPDF(task) => task.get_call_id(),
            TargetCallMethodTask::GetBoxModel(task) => task.get_call_id(),
            TargetCallMethodTask::GetContentQuads(task) => task.get_call_id(),
            TargetCallMethodTask::GetDocument(task) => task.get_call_id(),
            TargetCallMethodTask::PageEnable(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeEnable(task) => task.get_call_id(),
            TargetCallMethodTask::CaptureScreenshot(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeEvaluate(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeGetProperties(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_call_id(),
            TargetCallMethodTask::NetworkEnable(task) => task.get_call_id(),
            TargetCallMethodTask::SetRequestInterception(task) => task.get_call_id(),
            TargetCallMethodTask::ContinueInterceptedRequest(task) => task.get_call_id(),
            TargetCallMethodTask::GetResponseBodyForInterception(task) => task.get_call_id(),
            TargetCallMethodTask::PageReload(task) => task.get_call_id(),
            TargetCallMethodTask::GetLayoutMetrics(task) => task.get_call_id(),
            TargetCallMethodTask::BringToFront(task) => task.get_call_id(),
            TargetCallMethodTask::PageClose(task) => task.get_call_id(),
            TargetCallMethodTask::DispatchMouseEvent(task) => task.get_call_id(),
            TargetCallMethodTask::CanEmulate(task) => task.get_call_id(),
            TargetCallMethodTask::SetDeviceMetricsOverride(task) => task.get_call_id(),
        }
    }
}

pub fn handle_target_method_call(
    debug_session: &mut DebugSession,
    target_call_method_task: TargetCallMethodTask,
    _maybe_session_id: Option<target::SessionID>,
    maybe_target_id: Option<target::TargetId>,
) -> Result<PageResponseWrapper, failure::Error> {
    match target_call_method_task {
        TargetCallMethodTask::GetDocument(task) => {
            let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
            tab.root_node = task.task_result.clone();
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetDocument(task)),
            });
        }
        TargetCallMethodTask::NavigateTo(task) => {
            trace!("navigate_to task returned: {:?}", task);
        }
        TargetCallMethodTask::QuerySelector(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::QuerySelector(task)),
            });
        }
        TargetCallMethodTask::DescribeNode(task) => {
            // let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
            // let node_id = task.task_result.as_ref().and_then(|n| Some(n.node_id));

            let v = Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::DescribeNode(task)),
            });
            return v;
        }
        TargetCallMethodTask::PrintToPDF(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::PrintToPdf(task)),
            });
        }
        TargetCallMethodTask::GetBoxModel(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetBoxModel(task)),
            });
        }
        TargetCallMethodTask::GetContentQuads(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetContentQuads(task)),
            });
        }
        TargetCallMethodTask::PageEnable(task) => {
            info!("page_enabled: {:?}", task);
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::PageEnabled(task)),
            });
        }
        TargetCallMethodTask::PageClose(task) => {
            info!("page_closed: {:?}", task);
            return Ok(PageResponseWrapper::default());
        }
        TargetCallMethodTask::RuntimeEnable(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::RuntimeEnabled(task)),
            });
        }
        TargetCallMethodTask::CaptureScreenshot(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::CaptureScreenshot(task)),
            });
        }
        TargetCallMethodTask::RuntimeEvaluate(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::Evaluate(task)),
            });
        }
        TargetCallMethodTask::RuntimeGetProperties(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetProperties(task)),
            });
        }
        TargetCallMethodTask::RuntimeCallFunctionOn(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::CallFunctionOn(task)),
            });
        }
        TargetCallMethodTask::SetRequestInterception(_task) => {
            warn!("ignored method return SetRequestInterception");
            return Ok(PageResponseWrapper::default());
        }
        TargetCallMethodTask::NetworkEnable(_task) => {
            warn!("ignored method return. NetworkEnable");
            return Ok(PageResponseWrapper::default());
        }
        TargetCallMethodTask::ContinueInterceptedRequest(_task) => {
            warn!("ignored method return. ContinueInterceptedRequest");
            return Ok(PageResponseWrapper::default());
        }
        TargetCallMethodTask::PageReload(_task) => {
            warn!("ignored method return. PageReload");
            return Ok(PageResponseWrapper::default());
        }
        TargetCallMethodTask::GetLayoutMetrics(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetLayoutMetrics(task)),
            });
        }
        TargetCallMethodTask::DispatchMouseEvent(_task) => {
            warn!("ignored method return. DispatchMouseEvent");
            return Ok(PageResponseWrapper::default());
        }
        TargetCallMethodTask::BringToFront(task) => {
            debug_session.bring_to_front_responded(maybe_target_id.clone())?;
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::BringToFront(task)),
            });
        }
        TargetCallMethodTask::GetResponseBodyForInterception(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetResponseBodyForInterception(task)),
            });
        }
        TargetCallMethodTask::CanEmulate(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::CanEmulate(task)),
            });
        }
        TargetCallMethodTask::SetDeviceMetricsOverride(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::SetDeviceMetricsOverride(task)),
            });
        }
    } 
    warn!("unhandled branch handle_target_method_call");
    Ok(PageResponseWrapper::default())
}
