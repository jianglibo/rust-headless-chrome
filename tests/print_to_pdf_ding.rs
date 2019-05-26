#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::{PageResponse};
use headless_chrome::browser_async::embedded_events;
use headless_chrome::protocol::network::{ResourceType, InterceptionStage};
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct PrintToPdfDing {
    debug_session: DebugSession,
    url: &'static str,
    load_event_fired_count: u8,
}

impl PrintToPdfDing {
    fn assert_result(&self) {
        assert_eq!(self.load_event_fired_count, 1);
    }
}

impl Future for PrintToPdfDing {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
                let task_id = page_response_wrapper.task_id;
                match page_response_wrapper.page_response {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                        self.debug_session.security_enable();
                    }
                    PageResponse::PageCreated(_page_idx) => {
                        let tab = tab.expect("tab should exists.");
                        tab.attach_to_page();
                    }
                    PageResponse::PageAttached(_page_info, _session_id) => {
                        let tab = tab.expect("tab should exists. PageAttached");
                        tab.page_enable();
                        tab.network_enable(Some(999));
                        let mut task = tab.create_set_request_interception_task(Some(1000));
                        task.add_request_pattern(None, Some(ResourceType::XHR), Some(InterceptionStage::Request));
                        tab.execute_one_task(task.into());
                        tab.navigate_to(self.url, None);
                    }
                    PageResponse::PageEnabled => {}
                    PageResponse::LoadEventFired(_monotonic_time) => {
                        self.load_event_fired_count += 1;
                        let tab = tab.expect("tab should exists. LoadEventFired");
                        error!("load_event_fired: {:?}", tab);
                        if tab.is_chrome_error_chromewebdata() {
                            // tab.runtime_evaluate_expression("document.getElementById('proceed-link').click();", Some(200));
                            tab.runtime_evaluate_expression("document.getElementById('details-button')", Some(200));
                        } else {
                            let url = tab.get_url();
                            if url.contains("8888/login") {
                                tab.runtime_evaluate_expression("document.getElementsByClassName('login-tab').item(1).click(); document.getElementById('login-by').value='13777272378';document.getElementById('password').value='00000132abc';document.getElementById('btn-submit-login').click();", Some(201));
                            } else if url.contains("131/") {
                                let _ajax_fn = r##"
                                let tmp_ajax_function = function(URL) {
                                    return new Promise(function (resolve, reject) {
                                        var req = new XMLHttpRequest(); 
                                        req.open('GET', URL, true);
                                        req.onload = function () {
                                        if (req.status === 200) { 
                                                resolve(req.responseText);
                                            } else {
                                                reject(new Error(req.statusText));
                                            } 
                                        };
                                        req.onerror = function () {
                                            reject(new Error(req.statusText));
                                        };
                                        req.send(); 
                                    });
                                };

                                "##;
                            } else {
                                error!("unknown page loaded: {:?}", url);
                            }
                        }
                    }
                    PageResponse::ResponseReceived(response_params) => {
                        info!("got response_params: {:?}", response_params);
                    }
                    PageResponse::EvaluateDone(evaluate_return_object) => {
                        if task_id == Some(200) {
                            let tab = self.debug_session.first_page_mut().expect("main tab should exists.");
                            info!("evaluate_return_object: {:?}", evaluate_return_object);
                            let task2 = tab.navigate_to_task(self.url, None);
                            let task1 = self.debug_session.set_ignore_certificate_errors_task(true);
                            self.debug_session.execute_tasks(vec![task1, task2]);
                        } 
                    }
                    // PageResponse::SetIgnoreCertificateErrorsDone(_ignore) => {
                    //     // let tab = tab.expect("tab should exists. SetIgnoreCertificateErrorsDone");
                    //     let tab = self.debug_session.first_page_mut().expect("main tab should exists.");
                    //     tab.navigate_to(self.url, None); // this time should success.
                    // }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 20 {
                            self.assert_result();
                            break Ok(().into());
                        }
                    }
                    _ => {
                        info!("got unused page message {:?}", page_response_wrapper);
                    }
                }
            } else {
                warn!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn test_print_pdf_ding() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,print_to_pdf_ding=trace,derive_builder=trace");
    env_logger::init();
    let url = "https://59.202.58.131";

    let my_page = PrintToPdfDing {
        url,
        ..PrintToPdfDing::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).expect("tokio should success.");
}
