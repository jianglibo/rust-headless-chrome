#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::browser_async::task_describe::{TaskDescribe};
use headless_chrome::protocol::network::{InterceptionStage, ResourceType};
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct FromTimer {
    debug_session: DebugSession,
    url: &'static str,
    load_event_fired_count: u8,
    ff: bool,
}

impl FromTimer {
    fn assert_result(&self) {
        assert_eq!(self.load_event_fired_count, 2);
    }
}

impl Future for FromTimer {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let tab = self
                    .debug_session
                    .get_tab_by_resp_mut(&page_response_wrapper)
                    .ok();
                let task_id = page_response_wrapper.task_id;
                match page_response_wrapper.page_response {
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds == 10 {
                            self.debug_session.set_discover_targets(true);
                            self.debug_session.security_enable();
                            self.debug_session.set_ignore_certificate_errors(true);
                        } else if seconds == 15 {
                            let tab = self.debug_session.first_page_mut().expect("tab should exists.");
                            tab.attach_to_page();
                        } else if seconds == 17 {
                            let tab = self.debug_session.first_page_mut().expect("tab should exists.");
                            tab.navigate_to("http://www.163.com");
                        }
                        if seconds > 40 {
                            break Ok(().into());
                        }
                    }
                    PageResponse::ChromeConnected => {
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                    }
                    PageResponse::ReceivedEvent(event) => {}
                    PageResponse::MethodCallDone(method_call_done) => {}
                    PageResponse::Fail => {
                        info!("got fail.");
                    }

                }
            } else {
                warn!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn test_from_timer() {
    ::std::env::set_var(
        "RUST_LOG",
        "headless_chrome=trace,from_timer=trace,derive_builder=trace",
    );
    env_logger::init();
    // let url = "https://59.202.58.131";
    let url = "https://59.202.58.131/orgstructure/orgstructure-manage?orgId=100016626";
    let my_page = FromTimer {
        url,
        ..FromTimer::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}
