use headless_chrome::browser_async::{Tab, WaitingForPageAttachTaskName};

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::protocol::target;

use super::{GetContentInIframe, PageState, LOGIN_URL};

impl GetContentInIframe {

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
                    ];
                    tab.attach_to_page_and_then(tasks);
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                if let MethodCallDone::PageEnabled(_task) = method_call_done {
                    self.state = PageState::LoginPageDisplayed;
                    let tab = self
                        .get_tab(maybe_target_id)
                        .expect("tab should exists. RequestIntercepted");
                    tab.navigate_to(LOGIN_URL);
                }
            }
            _ => {}
        }
    }
}
