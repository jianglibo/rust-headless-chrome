use super::super::page_message::{MethodCallDone, PageResponse, PageResponseWrapper};
use super::super::protocol::target;
use super::{browser_tasks, page_tasks, security_tasks, target_tasks, HasTaskId, TaskDescribe};
use failure;
use log::*;

#[derive(Debug, Clone)]
pub enum BrowserCallMethodTask {
    ActivateTarget(target_tasks::ActivateTargetTask),
    GetTargets(target_tasks::GetTargetsTask),
    CreateTarget(target_tasks::CreateTargetTask),
    SetDiscoverTargets(target_tasks::SetDiscoverTargetsTask),
    SetIgnoreCertificateErrors(security_tasks::SetIgnoreCertificateErrorsTask),
    SecurityEnable(security_tasks::SecurityEnableTask),
    CloseTarget(target_tasks::CloseTargetTask),
    AttachedToTarget(page_tasks::AttachToTargetTask),
    GetBrowserCommandLine(browser_tasks::GetBrowserCommandLineTask),
}

impl std::convert::From<BrowserCallMethodTask> for TaskDescribe {
    fn from(task: BrowserCallMethodTask) -> Self {
        TaskDescribe::BrowserCallMethod(task)
    }
}

pub fn handle_browser_method_call(
    browser_call_method_task: BrowserCallMethodTask,
    _maybe_session_id: Option<target::SessionID>,
    maybe_target_id: Option<target::TargetId>,
) -> Result<PageResponseWrapper, failure::Error> {
    match browser_call_method_task {
        BrowserCallMethodTask::SetDiscoverTargets(task) => {
            trace!("TargetSetDiscoverTargets returned. {:?}", task);
            Ok(PageResponseWrapper::default())
        }
        BrowserCallMethodTask::CreateTarget(task) => {
            trace!("CreateTarget returned. {:?}", task);
            Ok(PageResponseWrapper::default())
        }
        BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(
                MethodCallDone::SetIgnoreCertificateErrors(task.ignore),
            ),
        }),
        BrowserCallMethodTask::SecurityEnable(task) => {
            trace!("SecurityEnable returned. {:?}", task);
            Ok(PageResponseWrapper::default())
        }
        BrowserCallMethodTask::ActivateTarget(task) => {
            trace!("ActivateTarget returned. {:?}", task);
            Ok(PageResponseWrapper::default())
        }
        BrowserCallMethodTask::AttachedToTarget(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::TargetAttached(task)),
        }),
        BrowserCallMethodTask::GetTargets(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::GetTargets(task)),
        }),
        BrowserCallMethodTask::GetBrowserCommandLine(task) => Ok(PageResponseWrapper {
            target_id: maybe_target_id,
            task_id: Some(task.get_task_id()),
            page_response: PageResponse::MethodCallDone(MethodCallDone::GetBrowserCommandLine(
                task,
            )),
        }),
        BrowserCallMethodTask::CloseTarget(task) => {
            let mut success = false;
            if let Some(r) = task.task_result {
                if r {
                    info!("tab close method call returned. close successfully.");
                    success = true;
                } else {
                    error!("tab close method call returned. close failed.");
                }
            } else {
                error!("tab close method call may not returned. {:?}", task);
            }
            Ok(PageResponseWrapper {
                target_id: maybe_target_id,
                task_id: Some(task.get_task_id()),
                page_response: PageResponse::MethodCallDone(MethodCallDone::PageClosed(success)),
            })
        }
    }
}

// impl HasCallId for BrowserCallMethodTask {
//     fn get_call_id(&self) -> usize {
//         match self {
//             BrowserCallMethodTask::CreateTarget(task) => task.get_call_id(),
//             BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_call_id(),
//             BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_call_id(),
//             BrowserCallMethodTask::SecurityEnable(task) => task.get_call_id(),
//             BrowserCallMethodTask::AttachedToTarget(task) => task.get_call_id(),
//             BrowserCallMethodTask::CloseTarget(task) => task.get_call_id(),
//             BrowserCallMethodTask::ActivateTarget(task) => task.get_call_id(),
//             BrowserCallMethodTask::GetTargets(task) => task.get_call_id(),
//             BrowserCallMethodTask::GetBrowserCommandLine(task) => task.get_call_id(),
//         }
//     }

//     fn renew_call_id(&mut self) {
//         match self {
//             BrowserCallMethodTask::CreateTarget(task) => task.renew_call_id(),
//             BrowserCallMethodTask::SetDiscoverTargets(task) => task.renew_call_id(),
//             BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.renew_call_id(),
//             BrowserCallMethodTask::SecurityEnable(task) => task.renew_call_id(),
//             BrowserCallMethodTask::AttachedToTarget(task) => task.renew_call_id(),
//             BrowserCallMethodTask::CloseTarget(task) => task.renew_call_id(),
//             BrowserCallMethodTask::ActivateTarget(task) => task.renew_call_id(),
//             BrowserCallMethodTask::GetTargets(task) => task.renew_call_id(),
//         }
//     }
// }
