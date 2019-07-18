extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::{EventName, Tab};
use headless_chrome::browser_async::page_message::{PageResponse};
use log::*;
use std::default::Default;
mod tutil;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

mod capture_screen_shot_mod;
use capture_screen_shot_mod::{CaptureScreenShotTest, PageState, HOME_URL};


impl Future for CaptureScreenShotTest {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                info!("{:?}", self.state);
                let maybe_target_id = page_response_wrapper.target_id.clone();
                if let PageResponse::SecondsElapsed(seconds) = page_response_wrapper.page_response {
                    for t in &self.debug_session.tabs {
                        trace!("{}, {:?}", t.get_url(), t.target_info.browser_context_id);
                    }
                    self.debug_session.browser_contexts().deduplicate();
                    if let Ok(tab) = self.debug_session.find_tab_by_name(HOME_URL) {
                        info!("requested urls: {:?}", tab.network_statistics.list_request_urls_end_with("/pclog"));
                    }
                    let popup_count = self.debug_session.loaded_by_this_tab_name_count(HOME_URL);
                    if popup_count > 0 {
                        let run_task_queue_manually = popup_count < 2;
                        let tab = self
                            .debug_session
                            .find_tab_by_name_mut(HOME_URL)
                            .expect("home page should exists.");
                        if run_task_queue_manually {
                            // info!("run_task_queue_manually.");
                            tab.run_task_queue_manually();
                        }
                    }
                    if let Some(tab) = self
                        .debug_session
                        .loaded_by_this_tab_name_mut(HOME_URL)
                        .get_mut(0)
                    {
                        if tab.bring_to_front() {
                            info!("bring to front................had sent.");
                        }
                    }
                    self.debug_session
                        .find_tabs_old_than(600)
                        .into_iter()
                        .filter(|tb| !tb.is_at_url(HOME_URL))
                        .for_each(Tab::page_close);
                    if seconds > 12_0000 {
                        self.debug_session
                            .tabs
                            .iter()
                            .for_each(|tb| info!("{:?}", tb));
                        assert_eq!(self.debug_session.tabs.len(), 19);
                        let m = self
                            .debug_session
                            .tabs
                            .iter()
                            .filter(|tb| tb.is_at_url(HOME_URL))
                            .count();
                        assert_eq!(m, 1);

                        let tab = self
                            .debug_session
                            .first_page_mut()
                            .expect("tab should exists.");
                        assert!(
                            tab.event_statistics
                                .happened_count(EventName::ExecutionContextCreated)
                                > 7
                        );
                        // self.assert_result();
                        break Ok(().into());
                    }
                } else {
                    match &mut self.state {
                        PageState::WaitingBlankPage => {
                            self.waiting_blank_page(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::HomePageDisplayed => {
                            self.home_page_displayed(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                    }
                }
            } else {
                warn!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn t_take_screen_shot() {
    tutil::setup_logger(vec![""]).expect("fern log should work.");

    let my_page = CaptureScreenShotTest::default();

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");

}
