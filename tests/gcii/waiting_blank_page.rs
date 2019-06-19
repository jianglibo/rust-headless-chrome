use headless_chrome::browser_async::{Tab, WaitingForPageAttachTaskName};

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::protocol::target;

use super::{GetContentInIframe, PageState, LOGIN_URL};

impl GetContentInIframe {

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
                match received_event {
                    ReceivedEvent::PageCreated(_page_idx) => {
                        let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                        let tasks = vec![
                            WaitingForPageAttachTaskName::PageEnable,
                            WaitingForPageAttachTaskName::RuntimeEnable,
                            WaitingForPageAttachTaskName::NetworkEnable
                        ];
                        tab.attach_to_page_and_then(tasks);
                    }
                    ReceivedEvent::PageAttached(_page_info, _session_id) => {
                        // let tab = self
                        //     .get_tab(target_id)
                        //     .expect("tab should exists. PageAttached");
                        // tab.runtime_enable();
                        // tab.page_enable();
                    }
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
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