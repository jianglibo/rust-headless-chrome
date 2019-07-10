extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::{EventName, Tab};
use headless_chrome::browser_async::page_message::{response_object, PageResponse};
use log::*;
use std::default::Default;
use std::fs;
use std::path::Path;
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
                    if let Some(popup_count) =
                        self.debug_session.loaded_by_this_tab_name_count(HOME_URL)
                    {
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
                    match self.state {
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
        // loop {
        //     if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
        //         let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
        //         match page_response_wrapper.page_response {
        //             PageResponse::ChromeConnected => {
        //                 self.debug_session.set_discover_targets(true);
        //             }
        //             PageResponse::PageEnabled => {
        //                 info!("page enabled.");
        //                 tab.unwrap().navigate_to(self.url, None);
        //             }
        //             PageResponse::SecondsElapsed(seconds) => {
        //                 trace!("seconds elapsed: {} ", seconds);
        //                 if seconds == 19 {
        //                     if let Some(tab) = self.debug_session.first_page_mut() {
        //                         tab.capture_screenshot_by_selector(
        //                             self.selector,
        //                             page::ScreenshotFormat::PNG,
        //                             true,
        //                             Some(100),
        //                         );
        //                     }
        //                 }
        //                 if seconds > 25 {
        //                     assert!(self.ro.is_some());

        //                     let file_name = "target/abc.png";
        //                     let path = Path::new(file_name);
        //                     if path.exists() && path.is_file() {
        //                         fs::remove_file(file_name).unwrap();
        //                     }
        //                     self.ro.as_ref().unwrap().write_to(file_name).unwrap();
        //                     assert!(path.exists());
        //                     break Ok(().into());
        //                 }
        //             }
        //             PageResponse::FrameNavigated(frame_id) => {
        //                 let tab = tab.unwrap();
        //                 let frame = tab.find_frame_by_id(&frame_id).unwrap();
        //                 info!("got frame: {:?}", frame_id);
        //                 if frame.name == Some("ddlogin-iframe".into()) {
        //                     if let Some(tab) = self.debug_session.first_page_mut() {
        //                         tab.capture_screenshot_by_selector(
        //                             self.selector,
        //                             page::ScreenshotFormat::PNG,
        //                             true,
        //                             Some(100),
        //                         );
        //                     }
        //                 }
        //             }
        //             PageResponse::CaptureScreenshotDone(capture_screen_shot) => {
        //                 info!("got screen shot: {:?}", capture_screen_shot.base64);
        //                 assert!(capture_screen_shot.base64.is_some());
        //                 assert_eq!(capture_screen_shot.selector, Some(self.selector));
        //                 self.ro = Some(capture_screen_shot);
        //             }
        //             _ => {
        //                 trace!("got unused page message {:?}", page_response_wrapper);
        //             }
        //         }
        //     } else {
        //         error!("got None, was stream ended?");
        //     }
        // }
    }
}

#[test]
fn t_take_screen_shot() {
    tutil::setup_logger().expect("fern log should work.");

    let my_page = CaptureScreenShotTest::default();

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");

}
