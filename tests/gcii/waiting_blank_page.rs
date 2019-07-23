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
                if let ReceivedEvent::PageCreated = received_event {
                    let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                    tab.page_enable();
                    tab.runtime_enable();
                    tab.attach_to_page();
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
