// place holder.
pub mod print_to_pdf;
pub mod navigate_to;
pub mod capture_screenshot;
pub mod page_enable;
pub mod page_events;

pub use print_to_pdf::{PrintToPdfTask, PrintToPdfTaskBuilder};
pub use navigate_to::{NavigateToTask, NavigateToTaskBuilder};
pub use capture_screenshot::{CaptureScreenshotTask, CaptureScreenshotTaskBuilder};
pub use page_enable::{PageEnableTask, PageEnableTaskBuilder};

#[derive(Debug)]
pub enum PageEvent {
    DomContentEventFired(page_events::DomContentEventFired),
    FrameAttached(page_events::FrameAttached),
    FrameDetached(page_events::FrameDetached),
    FrameNavigated(page_events::FrameNavigated),
    FrameStartedLoading(page_events::FrameStartedLoading),
    FrameStoppedLoading(page_events::FrameStoppedLoading),
    LoadEventFired(page_events::LoadEventFired),
}