// use headless_chrome::browser_async::WaitingForPageAttachTaskName;

use headless_chrome::browser_async::page_message::{PageResponse, ReceivedEvent};
use headless_chrome::protocol::target;
use log::*;

use super::{GetContentInIframe, PageState, HOME_URL, PAGE_STATE};


impl GetContentInIframe {
    pub fn waiting_for_qrcode_scan(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        // let expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').length"##;
        // let shenbian_gandong_task_name = "shenbian-gandong";
        if let PageResponse::ReceivedEvent(received_event) = page_response {
            match received_event {
                ReceivedEvent::LifeCycle => {
                    let tab = self
                        .get_tab(maybe_target_id)
                        .expect("tab should exists. LoadEventFired");
                    info!("got lifecycleEvent: {:?}", tab.life_cycles.last_life_cycle_event());
                    if tab.is_at_url(HOME_URL) {
                        if tab.life_cycles.last_life_cycle_event().is_network_almost_idle() {
                            // self.switch_to_home_page_displayed();
                            // *PAGE_STATE.lock().expect("PAGE_STATE") = PageState::HomePageFullDisplayed;
                            tab.explicitly_close = true;
                            tab.network_enable();
                            tab.activated_at.replace(std::time::Instant::now());
                            tab.set_move_mouse_random_interval(8, 20);
                            tab.display_full_page_after_secs(2);
                            self.state = PageState::HomePageFullDisplayed;
                        } 
                    } else {
                        info!("at state: {:?}", PageState::WaitingForQrcodeScan);
                    }
                }
                ReceivedEvent::ResponseReceived(_event) => {}
                _ => {
                    // info!("got unused page event {:?}", received_event);
                }
            }
        }
    }
}
