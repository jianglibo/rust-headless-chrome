use super::{target_tasks, security_tasks, page_tasks};
use super::{HasCallId, HasTaskId};
use super::super::page_message::{PageResponseWrapper, PageResponse, MethodCallDone,};
use crate::protocol::target;
use failure;
use log::*;

#[derive(Debug)]
pub enum BrowserCallMethodTask {
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
        }
    }
}

pub fn handle_browser_method_call(
        browser_call_method_task: BrowserCallMethodTask,
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match browser_call_method_task {
            BrowserCallMethodTask::SetDiscoverTargets(task) => {
                trace!("TargetSetDiscoverTargets returned. {:?}", task);
            }
            BrowserCallMethodTask::CreateTarget(task) => {
                trace!("CreateTarget returned. {:?}", task);
            }
            BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => {
                return Ok(PageResponseWrapper{
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::MethodCallDone(MethodCallDone::SetIgnoreCertificateErrors(task.ignore)),
                });
            }
            BrowserCallMethodTask::SecurityEnable(task) => {
                trace!("SecurityEnable returned. {:?}", task);
            }
            BrowserCallMethodTask::AttachedToTarget(task) => {
                return Ok(PageResponseWrapper{
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::MethodCallDone(MethodCallDone::TargetAttached(task)),
                });
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
            }
        }
        Ok(PageResponseWrapper::default())
    }
