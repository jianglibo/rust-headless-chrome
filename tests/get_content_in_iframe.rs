#![warn(clippy::all)]

extern crate log;
extern crate fern;
extern crate chrono;

use fern::colors::ColoredLevelConfig;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

mod gcii;

use headless_chrome::browser_async::{EventName, Tab};

use headless_chrome::browser_async::page_message::PageResponse;
use log::*;
use std::default::Default;

use websocket::futures::{Future, IntoFuture, Poll, Stream};

use gcii::{GetContentInIframe, PageState, HOME_URL, SHENBIAN_GANDONG_URL};

        // "headless_chrome=trace,get_content_in_iframe=trace",
fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new();
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                // record.level(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("headless_chrome", log::LevelFilter::Trace)
        .level_for("get_content_in_iframe", log::LevelFilter::Trace)
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}

impl GetContentInIframe {
    fn assert_result(&self) {
        assert!(self.ddlogin_frame_stopped_loading);
    }
}

impl Future for GetContentInIframe {
    type Item = ();
    type Error = failure::Error;

    #[allow(clippy::cognitive_complexity)]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let maybe_target_id = page_response_wrapper.target_id.clone();
                if let PageResponse::SecondsElapsed(seconds) = page_response_wrapper.page_response {
                    // trace!(
                    //     "seconds elapsed: {},number: {}  ",
                    //     seconds,
                    //     self.debug_session.tabs.len()
                    // );
                    // for t in &self.debug_session.tabs {
                    //     trace!("{}, {:?}", t.get_url(), t.target_info.browser_context_id);
                    // }
                    self.debug_session.browser_contexts().deduplicate();
                    // self.debug_session.activates_next_in_interval(10);
                    // self.debug_session.activate_last_opened_tab();
                    if let Some(popup_count) = self.debug_session.loaded_by_this_tab_name_count(HOME_URL) {
                        let run_task_queue_manually = popup_count < 2;
                        let tab = self.debug_session.find_tab_by_name_mut(HOME_URL).expect("home page should exists.");
                        if run_task_queue_manually {
                            info!("run_task_queue_manually.");
                            tab.run_task_queue_manually();
                        }
                    }
                    
                    if let Some(popup_count) = self.debug_session.loaded_by_this_tab_name_count(SHENBIAN_GANDONG_URL) {
                        let run_task_queue_manually = popup_count < 2;
                        let tab = self.debug_session.find_tab_by_name_mut(HOME_URL).expect("shenbian gandong page should exists.");
                        if run_task_queue_manually {
                            info!("run_task_queue_manually.");
                            tab.run_task_queue_manually();
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
                        self.assert_result();
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
                        PageState::LoginPageDisplayed => {
                            self.login_page_displayed(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::WaitingForQrcodeScan => {
                            self.waiting_for_qrcode_scan(
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


// the tabs were unattached when created by clicking the link. but the browser_context_id are same. the open_id are same too.

// post to: https://iflow-api.xuexi.cn/logflow/api/v1/pclog
#[test]
fn t_get_content_in_iframe() {
    // ::std::env::set_var(
    //     "RUST_LOG",
    //     "headless_chrome=trace,get_content_in_iframe=trace",
    // );
    // env_logger::init();
    setup_logger().expect("fern log should work.");

    let my_page = GetContentInIframe::default();

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}
