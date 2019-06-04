#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::browser_async::task_describe::{HasTaskId, TaskDescribe};
use headless_chrome::protocol::network::{InterceptionStage, ResourceType};
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct PrintToPdfDing {
    debug_session: DebugSession,
    url: &'static str,
    intercept_counter: u16,
    intercepted_requests: Vec<String>,
    // waiting_load_event_fired: std::collections::VecDeque<Vec<TaskDescribe>>,
}

impl PrintToPdfDing {
    fn assert_result(&self) {
        let tab = self
            .debug_session
            .first_page()
            .expect("first tab should exists.");
        assert_eq!(tab.load_event_fired_count, 2);
        // assert_eq!(self.intercepted_requests, vec![""]);
        assert_eq!(self.intercept_counter, 7);
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
                        // self.debug_session.security_enable();
                        self.debug_session.set_ignore_certificate_errors(true);
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 20 {
                            self.assert_result();
                            break Ok(().into());
                        }
                    }
                    PageResponse::ReceivedEvent(received_event) => {
                        match received_event {
                            ReceivedEvent::PageCreated(_page_idx) => {
                                let tab = tab.expect("tab should exists.");
                                tab.attach_to_page();
                            }
                            ReceivedEvent::PageAttached(_page_info, _session_id) => {
                                let tab = tab.expect("tab should exists. PageAttached");
                                let t1 = tab.page_enable_task();
                                let t2 = tab.network_enable_task(None);
                                tab.execute_tasks(vec![t1, t2]);
                            }
                            ReceivedEvent::FrameNavigated(_event) => {}
                            // Page navigating, page reload will cause this event to be fired.
                            // But interception may stop this event from firing. So we wait this page to be steady. when it fired 2 times, we enable interception and reload the page.
                            // But reload will cause this event to fire again, so be careful to distinct from each fire.
                            ReceivedEvent::LoadEventFired(_monotonic_time) => {
                                let tab = tab.expect("tab should exists. LoadEventFired");
                                // error!("load_event_fired: {:?}", tab);
                                let url = tab.get_url();
                                info!("url >>>>>>>>>>>>>>>>>>>>>>>>>>>>>  {:?}", url);
                                if tab.load_event_fired_count == 1 {
                                    let expression = r##"document.getElementsByClassName('login-tab').item(1).click();
                                        document.getElementById('login-by').value='13777272378';
                                        document.getElementById('password').value='00000132abc';
                                        document.getElementById('btn-submit-login').click();"##;
                                    let t1 =
                                        tab.evaluate_expression_task_named(expression, "login");
                                    let mut tk = tab.set_request_interception_task_named("a");
                                    // vec!["*/api/user/pagingValidUsers*", "*/api/user/getUserInfo*", "*/api/org/getTreeOrg*"].iter().for_each(|url| {
                                    vec!["*/api/*"].iter().for_each(|url| {
                                        tk.add_request_pattern(
                                            Some(url),
                                            Some(ResourceType::XHR),
                                            Some(InterceptionStage::HeadersReceived),
                                        );
                                    });
                                    let t3: TaskDescribe = tk.into();
                                    // let t4 = tab.reload_task(true, None);
                                    tab.execute_tasks(vec![t3, t1]);
                                } else if tab.load_event_fired_count == 2 {
                                    info!("second event fired.");
                                } else {
                                    info!("******************************************************");
                                }
                            }
                            ReceivedEvent::RequestIntercepted(interception_id) => {
                                self.intercept_counter += 1;
                                let tab = tab.expect("tab should exists. RequestIntercepted");
                                // tab.continue_intercepted_request(task.get_interception_id());
                                let intercepted_reqeust = tab
                                    .request_intercepted
                                    .get(&interception_id)
                                    .expect("RequestIntercepted should exists.");
                                info!("----------------------------------------------------------------------{:?}", intercepted_reqeust);
                                self.intercepted_requests.push(
                                    intercepted_reqeust.get_raw_parameters().request.url.clone(),
                                );
                                let request_id =
                                    intercepted_reqeust.get_raw_parameters().request_id.clone();
                                tab.get_response_body_for_interception(interception_id, request_id);
                            }
                            ReceivedEvent::ResponseReceived(_event) => {}
                            ReceivedEvent::RequestWillBeSent(_event) => {}
                            ReceivedEvent::DataReceived(_event) => {}
                            _ => {
                                info!("got unused page event {:?}", received_event);
                            }
                        }
                    }
                    PageResponse::MethodCallDone(method_call_done) => match method_call_done {
                        MethodCallDone::GetResponseBodyForInterception(task) => {
                            let tab = tab.expect("tab should exists. RequestIntercepted");
                            let readable = task.get_body_string();
                            info!("************* body string: {:?}", readable);
                            let decoded_body_string = task.get_body_string();
                            let intercepted = tab
                                .request_intercepted
                                .remove(&task.interception_id)
                                .expect("should find intercepted request.");
                            let saved_response = task
                                .request_id
                                .and_then(|rid| tab.response_received.remove(&rid));

                            let raw_response = intercepted.construct_raw_response_from_response(
                                saved_response.as_ref(),
                                decoded_body_string.as_ref().map(String::as_str),
                            );
                            tab.continue_intercepted_request_with_raw_response(
                                task.interception_id,
                                Some(raw_response),
                            );
                        }
                        MethodCallDone::Evaluate(task) => {
                            if task.task_id_equal("get-root") {
                                if task
                                    .task_result
                                    .as_ref()
                                    .expect("task should exists")
                                    .result
                                    .object_id
                                    .is_none()
                                {
                                    let tab = tab.expect("tab should exists. Evaluate");
                                    tab.execute_one_task(task.into());
                                } else {
                                    error!("****** result: {:?}", task.task_result);
                                }
                            }
                        }
                        MethodCallDone::PageEnabled(_task) => {
                            let tab = tab.expect("tab should exists. RequestIntercepted");
                            tab.navigate_to(self.url);
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
        "headless_chrome=info,print_to_pdf_ding=trace,derive_builder=trace",
    );
    // "headless_chrome=info,headless_chrome::browser_async::chrome_browser=trace,print_to_pdf_ding=trace,derive_builder=trace",
    env_logger::init();
    // let url = "https://59.202.58.131";
    let url = "https://59.202.58.131/orgstructure/orgstructure-manage?orgId=100016626";
    let my_page = PrintToPdfDing {
        url,
        // waiting_load_event_fired: std::collections::VecDeque::new(),
        intercepted_requests: Vec::new(),
        ..PrintToPdfDing::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}
