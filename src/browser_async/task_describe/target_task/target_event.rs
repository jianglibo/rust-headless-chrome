use crate::protocol::{page, network, target};
use super::super::{TaskDescribe, TargetEvent};

#[derive(Debug)]
pub struct ReceivedMessageFromTarget {}

#[derive(Debug)]
pub struct TargetCreated {}

#[derive(Debug)]
pub struct TargetCrashed {}

#[derive(Debug)]
pub struct TargetInfoChanged {
    pub target_info: target::TargetInfo,
    pub session_id: Option<target::SessionID>,
    pub target_id: Option<target::TargetId>,
}

impl std::convert::From<TargetInfoChanged> for TaskDescribe {
    fn from(event: TargetInfoChanged) -> Self {
        let te = TargetEvent::TargetInfoChanged(event);
        TaskDescribe::TargetEvent(te)
    }
}