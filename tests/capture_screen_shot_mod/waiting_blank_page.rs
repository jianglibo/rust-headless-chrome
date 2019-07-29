use headless_chrome::browser_async::{Tab};

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
                if let ReceivedEvent::PageCreated = received_event {
                    let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                    tab.network_enable();
                    tab.page_enable();
                    tab.runtime_enable();
                    tab.log_enable();
                    tab.lifecycle_events_enable();
                    tab.attach_to_page();
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
