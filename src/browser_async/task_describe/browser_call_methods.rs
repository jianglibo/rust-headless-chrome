use super::{target_tasks, security_tasks, page_tasks, HasCallId, HasTaskId, TaskDescribe};
use super::super::page_message::{PageResponseWrapper, PageResponse, MethodCallDone,};
use super::super::protocol::target;
use failure;
use log::*;

#[derive(Debug, Clone)]
pub enum BrowserCallMethodTask {
    ActivateTarget(target_tasks::ActivateTargetTask),
    CreateTarget(target_tasks::CreateTargetTask),
    SetDiscoverTargets(target_tasks::SetDiscoverTargetsTask),
    SetIgnoreCertificateErrors(security_tasks::SetIgnoreCertificateErrorsTask),
    SecurityEnable(security_tasks::SecurityEnableTask),
    CloseTarget(target_tasks::CloseTargetTask),
    AttachedToTarget(page_tasks::AttachToTargetTask),
}

impl HasCallId for BrowserCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            BrowserCallMethodTask::CreateTarget(task) => task.get_call_id(),
            BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_call_id(),
            BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_call_id(),
            BrowserCallMethodTask::SecurityEnable(task) => task.get_call_id(),
            BrowserCallMethodTask::AttachedToTarget(task) => task.get_call_id(),
            BrowserCallMethodTask::CloseTarget(task) => task.get_call_id(),
            BrowserCallMethodTask::ActivateTarget(task) => task.get_call_id(),
        }
    }
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
            BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => {
                Ok(PageResponseWrapper{
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::MethodCallDone(MethodCallDone::SetIgnoreCertificateErrors(task.ignore)),
                })
            }
            BrowserCallMethodTask::SecurityEnable(task) => {
                trace!("SecurityEnable returned. {:?}", task);
                Ok(PageResponseWrapper::default())
            }
            BrowserCallMethodTask::ActivateTarget(task) => {
                trace!("ActivateTarget returned. {:?}", task);
                Ok(PageResponseWrapper::default())
            }
            BrowserCallMethodTask::AttachedToTarget(task) => {
                Ok(PageResponseWrapper{
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::MethodCallDone(MethodCallDone::TargetAttached(task)),
                })
            }
            BrowserCallMethodTask::CloseTarget(task) => {
                if let Some(r) = task.task_result {
                    if r {
                        info!("tab close method call returned. close successfully.");
                    } else {
                        error!("tab close method call returned. close failed.");
                    }
                } else {
                    error!("tab close method call returned. close failed. {:?}", task);
                }
                Ok(PageResponseWrapper::default())
            }
        }
    }
