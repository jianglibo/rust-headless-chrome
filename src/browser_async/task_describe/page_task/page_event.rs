use crate::protocol::{page, network, target};
use super::super::{TaskDescribe, PageEvent};

#[derive(Debug)]
pub struct DomContentEventFired {
    pub timestamp: network::types::MonotonicTime,
}

#[derive(Debug)]
pub struct FrameAttached {
    pub params: page::events::FrameAttachedParams,
}

#[derive(Debug)]
pub struct FrameDetached {
    frame_id: page::types::FrameId,
}

#[derive(Debug)]
pub struct FrameNavigated {
    pub frame: page::Frame,
    pub session_id: Option<target::SessionID>,
    pub target_id: Option<target::TargetId>,
}

impl std::convert::From<FrameNavigated> for TaskDescribe {
    fn from(event: FrameNavigated) -> Self {
        let pe = PageEvent::FrameNavigated(event);
        TaskDescribe::PageEvent(pe)
    }
}

#[derive(Debug)]
pub struct FrameStartedLoading {

}

#[derive(Debug)]
pub struct FrameStoppedLoading {

}

#[derive(Debug)]
pub struct LoadEventFired {
    
}

#[derive(Debug)]
pub struct PageCreated {
    target_info: target::TargetInfo,
}

impl std::convert::From<PageCreated> for TaskDescribe {
    fn from(event:PageCreated) -> Self {
        let pe = PageEvent::PageCreated(event);
        TaskDescribe::PageEvent(pe)
    }
}

//     FrameAttached(page::events::FrameAttachedParams, CommonDescribeFields),
//     FrameDetached(page::types::FrameId, CommonDescribeFields),
//     FrameStartedLoading(String, CommonDescribeFields),
//     FrameNavigated(Box<page::Frame>, CommonDescribeFields),
//     FrameStoppedLoading(String, CommonDescribeFields),
//     LoadEventFired(target::TargetId, f32)