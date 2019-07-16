use headless_chrome::browser_async::{Tab, WaitingForPageAttachTaskName};

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::protocol::target;

use super::{CaptureScreenShotTest, PageState, HOME_URL};

impl CaptureScreenShotTest {
    pub fn get_tab(&mut self, target_id: Option<&target::TargetId>) -> Option<&mut Tab> {
        self.debug_session.find_tab_by_id_mut(target_id).ok()
    }

    pub fn waiting_blank_page(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        match page_response {
            PageResponse::ChromeConnected => {
                self.debug_session.set_discover_targets(true);
                self.debug_session.set_ignore_certificate_errors(true);
            }
            PageResponse::ReceivedEvent(received_event) => {
                if let ReceivedEvent::PageCreated(_page_idx) = received_event {
                    let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                    let tasks = vec![
                        WaitingForPageAttachTaskName::PageEnable,
                        WaitingForPageAttachTaskName::RuntimeEnable,
                        WaitingForPageAttachTaskName::SetLifecycleEventsEnabled,
                        WaitingForPageAttachTaskName::NetworkEnable
                    ];
                    tab.attach_to_page_and_then(tasks);
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                if let MethodCallDone::PageEnabled(_task) = method_call_done {
                    self.state = PageState::HomePageDisplayed;
                    let tab = self
                        .get_tab(maybe_target_id)
                        .expect("tab should exists. RequestIntercepted");
                    tab.navigate_to(HOME_URL);
                }
            }
            _ => {}
        }
    }
}
