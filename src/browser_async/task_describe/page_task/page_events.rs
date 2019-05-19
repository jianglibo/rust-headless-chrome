use super::super::{PageEvent, TaskDescribe};
use crate::browser_async::embedded_events;
use crate::protocol::{page, target};


wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::DomContentEventFired,
    DomContentEventFired,
    embedded_events::DomContentEventFired
);

wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameAttached,
    FrameAttached,
    page::events::FrameAttachedEvent
);

wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameDetached,
    FrameDetached,
    page::events::FrameDetachedEvent
);

wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameNavigated,
    FrameNavigated,
    page::events::FrameNavigatedEvent
);

#[derive(Debug)]
pub struct FrameStartedLoading {
    pub frame_id: page::types::FrameId,
}

#[derive(Debug)]
pub struct FrameStoppedLoading {
    pub frame_id: page::types::FrameId,
}

wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::LoadEventFired,
    LoadEventFired,
    embedded_events::LoadEventFired
);


// #[derive(Debug)]
// pub struct PageCreated {
//     pub target_info: target::TargetInfo,
// }

// impl_into_task_describe!(TaskDescribe::PageEvent, PageEvent::PageCreated, PageCreated);
impl_into_task_describe!(
    TaskDescribe::PageEvent,
    PageEvent::FrameStoppedLoading,
    FrameStoppedLoading
);
impl_into_task_describe!(
    TaskDescribe::PageEvent,
    PageEvent::FrameStartedLoading,
    FrameStartedLoading
);

//     FrameAttached(page::events::FrameAttachedParams, CommonDescribeFields),
//     FrameDetached(page::types::FrameId, CommonDescribeFields),
//     FrameStartedLoading(String, CommonDescribeFields),
//     FrameNavigated(Box<page::Frame>, CommonDescribeFields),
//     FrameStoppedLoading(String, CommonDescribeFields),
//     LoadEventFired(target::TargetId, f32)


#[cfg(test)]
mod tests {
    use super::*;
    use log::*;

    #[macro_export]
    macro_rules! add {
        {one to $input:expr} => ($input + 1);
        {two to $input:expr} => ($input + 2);
    }

    #[test]
    fn a_macro() {
        println!("Add two: {}", add!(two to 25/5));
    }
}
