use super::super::{TargetEvent, TaskDescribe};
use crate::protocol::{self, network, page, target};
use crate::browser_async::page_message::{PageResponse, PageResponseWrapper};

#[derive(Debug)]
pub struct ReceivedMessageFromTarget {}

// "{\"method\":\"Target.targetCreated\",\"params\":{\"targetInfo\":{\"targetId\":\"4dd28f26-cfdc-4067-98aa-0b818265bbe6\",\"type\":\"browser\",\"title\":\"\",\"url\":\"\",\"attached\":true}}}"

wrapper_raw_event!(
    TaskDescribe::TargetEvent,
    TargetEvent::TargetCreated,
    TargetCreated,
    target::events::TargetCreatedEvent
);

impl TargetCreated {
    pub fn get_target_type(&self) -> &target::TargetType {
        &self.raw_event.params.target_info.target_type
    }

    pub fn to_target_info(self) -> target::TargetInfo {
        self.raw_event.params.target_info
    }
}

#[derive(Debug)]
pub struct TargetCrashed {}

wrapper_raw_event!(
    TaskDescribe::TargetEvent,
    TargetEvent::AttachedToTarget,
    AttachedToTarget,
    target::events::AttachedToTargetEvent
);

impl AttachedToTarget {
    pub fn page_attached(&self) -> Option<PageResponseWrapper> {
        let target_info = &self.raw_event.params.target_info;
        if let protocol::target::TargetType::Page = target_info.target_type {
            let session_id = self.raw_event.params.session_id.clone();
            PageResponseWrapper {
                target_id: Some(target_info.target_id.clone()),
                task_id: None,
                page_response: PageResponse::PageAttached(target_info.clone(), session_id),
            }.into()
        } else {
            None
        }
    }
}

// "{\"method\":\"Target.targetInfoChanged\",\"params\":{\"targetInfo\":{\"targetId\":\"C2BE62797F9C7651987210D7540B7A01\",\"type\":\"page\",\"title\":\"about:blank\",\"url\":\"about:blank\",\"attached\":false,\"browserContextId\":\"6DFF871590F044BB2D4A888C41D7F1AA\"}}}"
wrapper_raw_event!(
    TaskDescribe::TargetEvent,
    TargetEvent::TargetInfoChanged,
    TargetInfoChanged,
    target::events::TargetInfoChangedEvent
);