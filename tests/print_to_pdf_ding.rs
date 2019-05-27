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
struct PrintToPdfDing {
    debug_session: DebugSession,
    url: &'static str,
    load_event_fired_count: u8,
}

impl PrintToPdfDing {
    fn assert_result(&self) {
        assert_eq!(self.load_event_fired_count, 2);
    }
}

impl Future for PrintToPdfDing {
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
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                        self.debug_session.security_enable();
                        self.debug_session.set_ignore_certificate_errors(true);
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 20 {
                            self.assert_result();
                            break Ok(().into());
                        }
                    }
                    PageResponse::ReceivedEvent(event) => match event {
                        ReceivedEvent::PageCreated(_page_idx) => {
                            let tab = tab.expect("tab should exists.");
                            tab.attach_to_page();
                        }
                        ReceivedEvent::PageAttached(_page_info, _session_id) => {
                            let tab = tab.expect("tab should exists. PageAttached");
                            let t1 = tab.page_enable_task();
                            let t2 = tab.network_enable_task(None);
                            
                            let mut task = tab.create_set_request_interception_task(Some("1000".into()));
                            task.add_request_pattern(
                                None,
                                Some(ResourceType::XHR),
                                Some(InterceptionStage::HeadersReceived),
                            );

                            let t3: TaskDescribe = task.into();
                            let t4 = tab.navigate_to_task(self.url, None);
                            tab.execute_tasks(vec![t1, t2, t3, t4]);
                        }
                        ReceivedEvent::LoadEventFired(_monotonic_time) => {
                            self.load_event_fired_count += 1;
                            let tab = tab.expect("tab should exists. LoadEventFired");
                            error!("load_event_fired: {:?}", tab);
                            let url = tab.get_url();
                            if url.contains("8888/login") {
                                tab.evaluate_expression_named("document.getElementsByClassName('login-tab').item(1).click(); document.getElementById('login-by').value='13777272378';document.getElementById('password').value='00000132abc';document.getElementById('btn-submit-login').click();",
                                 "login");
                            } else if url.contains("131/") {
                                tab.evaluate_expression_named("document.getElementById('100016626')", "get-root");
                            } else {
                                error!("unknown page loaded: {:?}", url);
                            }
                            // if tab.is_chrome_error_chromewebdata() {
                            //     tab.runtime_evaluate_expression(
                            //         "document.getElementById('details-button')",
                            //         Some(200),
                            //     );
                            // } else {
                            // }
                        }
                        ReceivedEvent::RequestIntercepted(interception_event) => {
                            let tab = tab.expect("tab should exists. RequestIntercepted");
                            tab.get_response_body_for_interception(
                                interception_event.get_interception_id(),
                                None,
                            );
                        }
                        ReceivedEvent::ResponseReceived(response_params) => {
                            info!("got response_params: {:?}", response_params);
                        }
                        _ => {
                            info!("got unused page event {:?}", event);
                        }
                    },
                    PageResponse::MethodCallDone(method_call_done) => match method_call_done {
                        MethodCallDone::GetResponseBodyForInterception(task) => {
                            error!("data: {:?}", task.get_body_string());
                            let tab = tab.expect("tab should exists. RequestIntercepted");
                            tab.continue_intercepted_request(task.interception_id);
                        }
                        MethodCallDone::Evaluate(task) => {
                            if task_id == Some("200".into()) {
                                let tab = self
                                    .debug_session
                                    .first_page_mut()
                                    .expect("main tab should exists.");
                                info!("evaluate_return_object: {:?}", task);
                            }
                        }
                        _ => {
                            info!("got unused method return: {:?}", method_call_done);
                        }
                    },
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
fn test_print_pdf_ding() {
    ::std::env::set_var(
        "RUST_LOG",
        "headless_chrome=trace,print_to_pdf_ding=trace,derive_builder=trace",
    );
    env_logger::init();
    // let url = "https://59.202.58.131";
    let url = "https://59.202.58.131/orgstructure/orgstructure-manage?orgId=100016626";
    let my_page = PrintToPdfDing {
        url,
        ..PrintToPdfDing::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}
