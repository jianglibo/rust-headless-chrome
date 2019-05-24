use super::TargetEvent;
use super::super::TaskDescribe;
use crate::protocol::{self, target};
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

// {\"method\":\"Target.attachedToTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"targetInfo\":{\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\",\"type\":\"page\",\"title\":\"\",\"url\":\"about:blank\",\"attached\":true,\"browserContextId\":\"6CEFE43CB35F53A22DB4009118D8978C\"},\"waitingForDebugger\":false}}
wrapper_raw_event!(
    TaskDescribe::TargetEvent,
    TargetEvent::AttachedToTarget,
    AttachedToTarget,
    target::events::AttachedToTargetEvent
);

impl AttachedToTarget {
    pub fn get_target_id(&self) -> target::TargetId {
        self.raw_event.params.target_info.target_id.clone()
    }
    pub fn get_session_id(&self) -> target::SessionID {
        self.raw_event.params.session_id.clone()
    }

    pub fn is_page_attached(&self) -> bool {
        if let target::TargetType::Page = &self.raw_event.params.target_info.target_type {
            true
        } else {
            false
        }
    }

    pub fn try_into_page_attached(self) -> Option<PageResponseWrapper> {
        if let protocol::target::TargetType::Page = &self.raw_event.params.target_info.target_type {
            let session_id = self.raw_event.params.session_id;
            let target_info = self.raw_event.params.target_info;
            PageResponseWrapper {
                target_id: Some(target_info.target_id.clone()),
                task_id: None,
                page_response: PageResponse::PageAttached(target_info, session_id),
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

impl TargetInfoChanged {
    pub fn into_target_info(self) -> target::TargetInfo {
        self.raw_event.params.target_info
    }
}