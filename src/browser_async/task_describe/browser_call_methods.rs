use super::target_task::create_target::CreateTargetTask;
use super::target_task::set_discover_target_task::SetDiscoverTargetsTask;
use super::{HasCallId};

#[derive(Debug)]
pub enum BrowserCallMethodTask {
    CreateTarget(CreateTargetTask),
    SetDiscoverTargets(SetDiscoverTargetsTask),
}

impl HasCallId for BrowserCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            BrowserCallMethodTask::CreateTarget(task) => task.get_call_id(),
            BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_call_id(),
        }
    }
}
