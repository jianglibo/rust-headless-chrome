use super::target_task::create_target::CreateTargetTask;
use super::security_task::set_ignore_certificate_errors::SetIgnoreCertificateErrorsTask;
use super::target_task::set_discover_target_task::SetDiscoverTargetsTask;
use super::{HasCallId, HasTaskId};
use super::super::page_message::{PageResponseWrapper, PageResponse};
use crate::protocol::target;
use failure;
use log::*;

#[derive(Debug)]
pub enum BrowserCallMethodTask {
    CreateTarget(CreateTargetTask),
    SetDiscoverTargets(SetDiscoverTargetsTask),
    SetIgnoreCertificateErrors(SetIgnoreCertificateErrorsTask),
}

impl HasCallId for BrowserCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            BrowserCallMethodTask::CreateTarget(task) => task.get_call_id(),
            BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_call_id(),
            BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_call_id(),
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
                trace!("TargetSetDiscoverTargets returned. {:?}", task);
            }
            BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => {
                return Ok(PageResponseWrapper{
                    target_id: maybe_target_id,
                    task_id: Some(task.get_task_id()),
                    page_response: PageResponse::SetIgnoreCertificateErrorsDone(task.ignore),
                });
            }
        }
        Ok(PageResponseWrapper::default())
    }
