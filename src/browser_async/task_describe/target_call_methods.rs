use super::SecurityEnableTask;
use super::{CaptureScreenshotTask, NavigateToTask, PageEnableTask, PrintToPdfTask};
use super::{DescribeNodeTask, GetBoxModelTask, GetDocumentTask, QuerySelectorTask,
    RuntimeCallFunctionOnTask, RuntimeEnableTask, RuntimeEvaluateTask, RuntimeGetPropertiesTask,
};



use super::super::debug_session::DebugSession;
use super::super::page_message::{response_object, PageResponse, PageResponseWrapper};
use crate::protocol::target;
use log::*;

use super::{HasCallId, HasTaskId};

#[derive(Debug)]
pub enum TargetCallMethodTask {
    NavigateTo(NavigateToTask),
    QuerySelector(QuerySelectorTask),
    DescribeNode(DescribeNodeTask),
    PrintToPDF(PrintToPdfTask),
    GetBoxModel(GetBoxModelTask),
    GetDocument(GetDocumentTask),
    PageEnable(PageEnableTask),
    RuntimeEnable(RuntimeEnableTask),
    CaptureScreenshot(CaptureScreenshotTask),
    RuntimeEvaluate(RuntimeEvaluateTask),
    RuntimeGetProperties(RuntimeGetPropertiesTask),
    RuntimeCallFunctionOn(RuntimeCallFunctionOnTask),
    SecurityEnable(SecurityEnableTask),
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
            TargetCallMethodTask::SecurityEnable(task) => task.get_call_id(),
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
            let v = Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::GetDocumentDone,
            });
            tab.root_node = task.task_result;
            return v;
        }
        TargetCallMethodTask::NavigateTo(task) => {
            trace!("navigate_to task returned: {:?}", task);
        }
        TargetCallMethodTask::QuerySelector(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: task.into_page_response(),
            });
        }
        TargetCallMethodTask::DescribeNode(task) => {
            let tab = debug_session.get_tab_by_id_mut(maybe_target_id.as_ref())?;
            let node_id = task.task_result.as_ref().and_then(|n| Some(n.node_id));

            let v = Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::DescribeNodeDone(task.selector, node_id),
            });

            tab.node_returned(task.task_result);
            return v;
        }
        TargetCallMethodTask::PrintToPDF(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::PrintToPdfDone(task.task_result),
            });
        }
        TargetCallMethodTask::GetBoxModel(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::GetBoxModelDone(
                    task.selector,
                    task.task_result.map(Box::new),
                ),
            });
        }
        TargetCallMethodTask::PageEnable(task) => {
            info!("page_enabled: {:?}", task);
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::PageEnabled,
            });
        }
        TargetCallMethodTask::RuntimeEnable(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::RuntimeEnabled,
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
                page_response: PageResponse::CaptureScreenshotDone(ro),
            });
        }
        TargetCallMethodTask::RuntimeEvaluate(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::EvaluateDone(task.task_result),
            });
        }
        TargetCallMethodTask::RuntimeGetProperties(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::GetPropertiesDone(task.task_result),
            });
        }
        TargetCallMethodTask::RuntimeCallFunctionOn(task) => {
            return Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::CallFunctionOnDone(task.task_result),
            });
        }
        _ => {
            info!("ignored method return. {:?}", target_call_method_task);
        }
    }    warn!("unhandled branch handle_target_method_call");
    Ok(PageResponseWrapper::default())
}
