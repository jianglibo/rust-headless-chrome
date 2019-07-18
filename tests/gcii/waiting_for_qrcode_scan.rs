// use headless_chrome::browser_async::WaitingForPageAttachTaskName;

use headless_chrome::browser_async::page_message::{PageResponse, ReceivedEvent};
use headless_chrome::protocol::target;

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
                ReceivedEvent::PageCreated(_page_idx) => {
                    // let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                    // let tasks = vec![
                    //     WaitingForPageAttachTaskName::PageEnable,
                    //     WaitingForPageAttachTaskName::RuntimeEnable,
                    //     WaitingForPageAttachTaskName::NetworkEnable,
                    // ];
                    // tab.attach_to_page_and_then(tasks);
                }
                ReceivedEvent::LoadEventFired(_t) => {
                    let tab = self
                        .get_tab(maybe_target_id)
                        .expect("tab should exists. LoadEventFired");
                    assert_eq!(tab.get_url(), HOME_URL);
                    tab.network_enable();
                    tab.move_mouse_random_after_secs(4);
                    tab.display_full_page_after_secs(6);
                    self.state = PageState::HomePageFullDisplayed;
                }
                ReceivedEvent::ResponseReceived(_event) => {}
                _ => {
                    // info!("got unused page event {:?}", received_event);
                }
            }
        }
    }
}
