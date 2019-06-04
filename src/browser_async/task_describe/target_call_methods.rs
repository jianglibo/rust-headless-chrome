use super::{dom_tasks, network_tasks, page_tasks,runtime_tasks};

use super::super::debug_session::DebugSession;
use super::super::page_message::{response_object, PageResponse, PageResponseWrapper, MethodCallDone};
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
    GetDocument(dom_tasks::GetDocumentTask),
    PageEnable(page_tasks::PageEnableTask),
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
}

impl HasCallId for TargetCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            TargetCallMethodTask::NavigateTo(task) => task.get_call_id(),
            TargetCallMethodTask::QuerySelector(task) => task.get_call_id(),
            TargetCallMethodTask::DescribeNode(task) => task.get_call_id(),
            TargetCallMethodTask::PrintToPDF(task) => task.get_call_id(),
            TargetCallMethodTask::GetBoxModel(task) => task.get_call_id(),
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
        }
    }
}

pub fn handle_target_method_call(
    debug_session: &mut DebugSession,
    target_call_method_task: TargetCallMethodTask,
    maybe_session_id: Option<target::SessionID>,
    maybe_target_id: Option<target::TargetId>,
) -> Result<PageResponseWrapper, failure::Error> {
    match target_call_method_task {
        TargetCallMethodTask::GetDocument(task) => {
            let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
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
            let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
            let node_id = task.task_result.as_ref().and_then(|n| Some(n.node_id));

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
                // page_response: PageResponse::MethodCallDone(MethodCallDone::GetBoxModel(
                //     task.selector,
                //     task.task_result.map(Box::new),
                // )),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetBoxModel(task)),
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
        TargetCallMethodTask::RuntimeEnable(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::RuntimeEnabled(task)),
            });
        }
        TargetCallMethodTask::CaptureScreenshot(task) => {
            let task_id = task.get_task_id();
            let ro = response_object::CaptureScreenshot {
                selector: task.selector,
                base64: task.task_result,
            };
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task_id),
                page_response: PageResponse::MethodCallDone(MethodCallDone::CaptureScreenshot(ro)),
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
        TargetCallMethodTask::SetRequestInterception(task) => {
            warn!("ignored method return SetRequestInterception");
        }
        TargetCallMethodTask::NetworkEnable(task) => {
            warn!("ignored method return. NetworkEnable");
        }
        TargetCallMethodTask::ContinueInterceptedRequest(task) => {
            warn!("ignored method return. ContinueInterceptedRequest");
        }
        TargetCallMethodTask::PageReload(task) => {
            warn!("ignored method return. PageReload");
        }
        TargetCallMethodTask::GetResponseBodyForInterception(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetResponseBodyForInterception(task)),
            });
        }
    } 
    warn!("unhandled branch handle_target_method_call");
    Ok(PageResponseWrapper::default())
}
