use super::{
    dom_tasks, emulation_tasks, input_tasks, log_tasks, network_tasks, page_tasks, runtime_tasks,
    HasTaskId, TaskDescribe,
};

use super::super::debug_session::DebugSession;
use super::super::page_message::{MethodCallDone, PageResponse, PageResponseWrapper};
use super::super::protocol::target;
use log::*;

#[derive(Debug, Clone)]
pub enum TargetCallMethodTask {
    NavigateTo(page_tasks::NavigateToTask),
    QuerySelector(dom_tasks::QuerySelectorTask),
    DescribeNode(dom_tasks::DescribeNodeTask),
    PrintToPDF(page_tasks::PrintToPdfTask),
    GetBoxModel(dom_tasks::GetBoxModelTask),
    GetContentQuads(dom_tasks::GetContentQuadsTask),
    GetDocument(dom_tasks::GetDocumentTask),
    PageEnable(page_tasks::PageEnableTask),
    LogEnable(log_tasks::LogEnableTask),
    SetLifecycleEventsEnabled(page_tasks::SetLifecycleEventsEnabledTask),
    PageClose(page_tasks::PageCloseTask),
    GetLayoutMetrics(page_tasks::GetLayoutMetricsTask),
    BringToFront(page_tasks::BringToFrontTask),
    RuntimeEnable(runtime_tasks::RuntimeEnableTask),
    CaptureScreenshot(page_tasks::CaptureScreenshotTask),
    Evaluate(runtime_tasks::EvaluateTask),
    GetProperties(runtime_tasks::GetPropertiesTask),
    RuntimeCallFunctionOn(runtime_tasks::CallFunctionOnTask),
    NetworkEnable(network_tasks::NetworkEnableTask),
    SetRequestInterception(network_tasks::SetRequestInterceptionTask),
    ContinueInterceptedRequest(network_tasks::ContinueInterceptedRequestTask),
    GetResponseBodyForInterception(network_tasks::GetResponseBodyForInterceptionTask),
    PageReload(page_tasks::PageReloadTask),
    DispatchMouseEvent(input_tasks::DispatchMouseEventTask),
    CanEmulate(emulation_tasks::CanEmulateTask),
    SetDeviceMetricsOverride(emulation_tasks::SetDeviceMetricsOverrideTask),
}

impl std::convert::From<TargetCallMethodTask> for TaskDescribe {
    fn from(task: TargetCallMethodTask) -> Self {
        TaskDescribe::TargetCallMethod(task)
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
            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetDocument(task)),
            })
        }
        TargetCallMethodTask::NavigateTo(task) => {
            trace!("navigate_to task returned: {:?}", task);
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::QuerySelector(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::QuerySelector(task)),
        }),
        TargetCallMethodTask::DescribeNode(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::DescribeNode(task)),
        }),
        TargetCallMethodTask::PrintToPDF(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::PrintToPdf(task)),
        }),
        TargetCallMethodTask::GetBoxModel(task) => {
            if task.request_full_page {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                if let Some(bm) = task.task_result.as_ref().cloned() {
                    tab.box_model.replace(bm);
                }
            }

            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::GetBoxModel(task)),
            })
        }
        TargetCallMethodTask::GetContentQuads(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::GetContentQuads(task)),
        }),
        TargetCallMethodTask::PageEnable(task) => {
            info!("page_enabled: {:?}", task);
            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::PageEnabled(task)),
            })
        }
        TargetCallMethodTask::PageClose(task) => {
            info!("page_closed: {:?}", task);
            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::PageClosed(true)),
            })
        }
        TargetCallMethodTask::RuntimeEnable(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::RuntimeEnabled(task)),
        }),
        TargetCallMethodTask::CaptureScreenshot(task) => {
            task.save()?;
            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::CaptureScreenshot(
                    task,
                )),
            })
        }
        TargetCallMethodTask::Evaluate(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::Evaluate(task)),
        }),
        TargetCallMethodTask::GetProperties(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::GetProperties(task)),
        }),
        TargetCallMethodTask::RuntimeCallFunctionOn(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::CallFunctionOn(task)),
        }),
        TargetCallMethodTask::SetRequestInterception(_task) => {
            trace!("ignored method return SetRequestInterception");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::NetworkEnable(_task) => {
            trace!("ignored method return. NetworkEnable");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::LogEnable(_task) => {
            trace!("ignored method return. LogEnable");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::ContinueInterceptedRequest(_task) => {
            trace!("ignored method return. ContinueInterceptedRequest");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::PageReload(_task) => {
            trace!("ignored method return. PageReload");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::SetLifecycleEventsEnabled(_task) => {
            trace!("ignored method return. SetLifecycleEventsEnabled");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::GetLayoutMetrics(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::GetLayoutMetrics(task)),
        }),
        TargetCallMethodTask::DispatchMouseEvent(_task) => {
            trace!("ignored method return. DispatchMouseEvent");
            Ok(PageResponseWrapper::default())
        }
        TargetCallMethodTask::BringToFront(task) => {
            debug_session.bring_to_front_responded(maybe_target_id.clone())?;
            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::BringToFront(task)),
            })
        }
        TargetCallMethodTask::GetResponseBodyForInterception(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(
                MethodCallDone::GetResponseBodyForInterception(task),
            ),
        }),
        TargetCallMethodTask::CanEmulate(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::CanEmulate(task)),
        }),
        TargetCallMethodTask::SetDeviceMetricsOverride(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::SetDeviceMetricsOverride(
                task,
            )),
        }),
    }
}

// impl HasSessionId for TargetCallMethodTask {
//     fn set_session_id(&mut self, session_id: target::SessionID) {
//         match self {
//             TargetCallMethodTask::NavigateTo(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::QuerySelector(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::DescribeNode(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::PrintToPDF(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::GetBoxModel(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::GetContentQuads(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::GetDocument(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::PageEnable(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::RuntimeEnable(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::CaptureScreenshot(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::Evaluate(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::GetProperties(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::RuntimeCallFunctionOn(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::NetworkEnable(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::SetRequestInterception(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::ContinueInterceptedRequest(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::GetResponseBodyForInterception(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::PageReload(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::GetLayoutMetrics(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::BringToFront(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::PageClose(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::DispatchMouseEvent(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::CanEmulate(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::SetDeviceMetricsOverride(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::SetLifecycleEventsEnabled(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//             TargetCallMethodTask::LogEnable(task) => {task.get_common_fields_mut().session_id.replace(session_id);}
//         }
//     }
// }

// impl HasCallId for TargetCallMethodTask {
//     fn get_call_id(&self) -> usize {
//         match self {
//             TargetCallMethodTask::NavigateTo(task) => task.get_call_id(),
//             TargetCallMethodTask::QuerySelector(task) => task.get_call_id(),
//             TargetCallMethodTask::DescribeNode(task) => task.get_call_id(),
//             TargetCallMethodTask::PrintToPDF(task) => task.get_call_id(),
//             TargetCallMethodTask::GetBoxModel(task) => task.get_call_id(),
//             TargetCallMethodTask::GetContentQuads(task) => task.get_call_id(),
//             TargetCallMethodTask::GetDocument(task) => task.get_call_id(),
//             TargetCallMethodTask::PageEnable(task) => task.get_call_id(),
//             TargetCallMethodTask::RuntimeEnable(task) => task.get_call_id(),
//             TargetCallMethodTask::CaptureScreenshot(task) => task.get_call_id(),
//             TargetCallMethodTask::Evaluate(task) => task.get_call_id(),
//             TargetCallMethodTask::GetProperties(task) => task.get_call_id(),
//             TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_call_id(),
//             TargetCallMethodTask::NetworkEnable(task) => task.get_call_id(),
//             TargetCallMethodTask::SetRequestInterception(task) => task.get_call_id(),
//             TargetCallMethodTask::ContinueInterceptedRequest(task) => task.get_call_id(),
//             TargetCallMethodTask::GetResponseBodyForInterception(task) => task.get_call_id(),
//             TargetCallMethodTask::PageReload(task) => task.get_call_id(),
//             TargetCallMethodTask::GetLayoutMetrics(task) => task.get_call_id(),
//             TargetCallMethodTask::BringToFront(task) => task.get_call_id(),
//             TargetCallMethodTask::PageClose(task) => task.get_call_id(),
//             TargetCallMethodTask::DispatchMouseEvent(task) => task.get_call_id(),
//             TargetCallMethodTask::CanEmulate(task) => task.get_call_id(),
//             TargetCallMethodTask::SetDeviceMetricsOverride(task) => task.get_call_id(),
//             TargetCallMethodTask::SetLifecycleEventsEnabled(task) => task.get_call_id(),
//             TargetCallMethodTask::LogEnable(task) => task.get_call_id(),
//         }
//     }
//     fn renew_call_id(&mut self) {
//         match self {
//             TargetCallMethodTask::NavigateTo(task) => task.renew_call_id(),
//             TargetCallMethodTask::QuerySelector(task) => task.renew_call_id(),
//             TargetCallMethodTask::DescribeNode(task) => task.renew_call_id(),
//             TargetCallMethodTask::PrintToPDF(task) => task.renew_call_id(),
//             TargetCallMethodTask::GetBoxModel(task) => task.renew_call_id(),
//             TargetCallMethodTask::GetContentQuads(task) => task.renew_call_id(),
//             TargetCallMethodTask::GetDocument(task) => task.renew_call_id(),
//             TargetCallMethodTask::PageEnable(task) => task.renew_call_id(),
//             TargetCallMethodTask::RuntimeEnable(task) => task.renew_call_id(),
//             TargetCallMethodTask::CaptureScreenshot(task) => task.renew_call_id(),
//             TargetCallMethodTask::Evaluate(task) => task.renew_call_id(),
//             TargetCallMethodTask::GetProperties(task) => task.renew_call_id(),
//             TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.renew_call_id(),
//             TargetCallMethodTask::NetworkEnable(task) => task.renew_call_id(),
//             TargetCallMethodTask::SetRequestInterception(task) => task.renew_call_id(),
//             TargetCallMethodTask::ContinueInterceptedRequest(task) => task.renew_call_id(),
//             TargetCallMethodTask::GetResponseBodyForInterception(task) => task.renew_call_id(),
//             TargetCallMethodTask::PageReload(task) => task.renew_call_id(),
//             TargetCallMethodTask::GetLayoutMetrics(task) => task.renew_call_id(),
//             TargetCallMethodTask::BringToFront(task) => task.renew_call_id(),
//             TargetCallMethodTask::PageClose(task) => task.renew_call_id(),
//             TargetCallMethodTask::DispatchMouseEvent(task) => task.renew_call_id(),
//             TargetCallMethodTask::CanEmulate(task) => task.renew_call_id(),
//             TargetCallMethodTask::SetDeviceMetricsOverride(task) => task.renew_call_id(),
//             TargetCallMethodTask::SetLifecycleEventsEnabled(task) => task.renew_call_id(),
//             TargetCallMethodTask::LogEnable(task) => task.renew_call_id(),
//         }
//     }
// }