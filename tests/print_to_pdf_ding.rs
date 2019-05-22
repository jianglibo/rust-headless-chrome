#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::{write_base64_str_to, PageResponse};
use log::*;
use std::default::Default;
use std::fs;
use std::path::Path;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct PrintToPdfDing {
    debug_session: DebugSession,
    url: &'static str,
    base64_data: Option<String>,
    load_event_fired_count: u8,
}

impl PrintToPdfDing {
    fn assert_result(&self) {
        assert!(self.base64_data.is_some());
        assert_eq!(self.load_event_fired_count, 1);
    }
}

impl Future for PrintToPdfDing {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                error!("got page_response_wrapper: {:?}", page_response_wrapper);
                let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
                match page_response_wrapper.page_response {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageCreated(page_idx) => {
                        let tab = tab.expect("tab should exists.");
                        tab.attach_to_page();
                    }
                    PageResponse::PageAttached(_page_info, _session_id) => {
                        let tab = tab.expect("tab should exists. PageAttached");
                        tab.page_enable();
                        tab.navigate_to(self.url, None);
                    }
                    PageResponse::PageEnabled => {}
                    PageResponse::LoadEventFired(_monotonic_time) => {
                        let tab = tab.expect("tab should exists. LoadEventFired");
                        error!("load_event_fired: {:?}", tab);
                        if tab.is_chromewebdata() {
                            self.load_event_fired_count += 1;
                            tab.print_to_pdf(Some(101), None);
                        }
                    }
                    PageResponse::PrintToPdfDone(base64_data) => {
                        let file_name = "target/print_to_pdf.pdf";
                        let path = Path::new(file_name);
                        if path.exists() && path.is_file() {
                            fs::remove_file(file_name).unwrap();
                        }
                        write_base64_str_to(file_name, base64_data.as_ref())
                            .map(|_| {
                                self.base64_data = base64_data;
                            })
                            .expect("write_base64_str_to failed.");

                        assert!(path.exists());
                        // break Ok(().into());
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 90 {
                            self.assert_result();
                            break Ok(().into());
                        }
                    }
                    _ => {
                        error!("got unused page message {:?}", page_response_wrapper);
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn test_print_pdf_ding() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,query_selector=trace");
    env_logger::init();
    let url = "https://59.202.58.131";

    let my_page = PrintToPdfDing {
        url,
        ..PrintToPdfDing::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).expect("tokio should success.");
}
