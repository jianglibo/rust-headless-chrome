// use headless_chrome::browser_async::WaitingForPageAttachTaskName;

use headless_chrome::browser_async::page_message::{PageResponse, ReceivedEvent};
use headless_chrome::protocol::target;
use log::*;

use super::{GetContentInIframe, PageState, HOME_URL};


impl GetContentInIframe {
    pub fn waiting_for_qrcode_scan(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        // let expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').length"##;
        let shenbian_gandong_task_name = "shenbian-gandong";
        if let PageResponse::ReceivedEvent(received_event) = page_response {
            match received_event {
                // be carefull, this PageCreated event may not fire.
                ReceivedEvent::PageCreated => {
                    // let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                    // let tasks = vec![
                    //     WaitingForPageAttachTaskName::PageEnable,
                    //     WaitingForPageAttachTaskName::RuntimeEnable,
                    //     WaitingForPageAttachTaskName::NetworkEnable,
                    // ];
                    // tab.attach_to_page_and_then(tasks);
                }
                // ReceivedEvent::LoadEventFired(_t) => {
                ReceivedEvent::LifeCycle => {
                    let tab = self
                        .get_tab(maybe_target_id)
                        .expect("tab should exists. LoadEventFired");
                    if tab.is_at_url(HOME_URL) {
                        if tab.last_life_cycle_event().is_network_almost_idle() {
                            tab.network_enable();
                            tab.activated_at.replace(std::time::Instant::now());
                            tab.move_mouse_random_after_secs(10);
                            // tab.set_move_mouse_random_interval(8, 20);
                            tab.display_full_page_after_secs(16);
                            self.state = PageState::HomePageFullDisplayed;
                        } else {
                            info!("got lifecycleEvent: {:?}", tab.last_life_cycle_event());
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
