#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::{debug_session::DebugSession, Tab};

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::browser_async::task_describe::{
    runtime_tasks, HasTaskId, TaskDescribe,
};
use headless_chrome::protocol::{
    network::{InterceptionStage, ResourceType},
    target,
};
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

enum PageState {
    WaitingBlankPage,
    WaitingLoginPage,
    TargetPageDisplayed,
}

impl Default for PageState {
    fn default() -> Self {
        PageState::WaitingBlankPage
    }
}

#[derive(Default)]
struct PrintToPdfDing {
    debug_session: DebugSession,
    url: &'static str,
    intercept_counter: u16,
    recursive: u8,
    state: PageState,
}

impl PrintToPdfDing {
    fn assert_result(&self) {
        let tab = self
            .debug_session
            .first_page()
            .expect("first tab should exists.");
        assert_eq!(tab.event_statistics.load_event_fired_count(), 2);
        assert!(self.intercept_counter > 7);
    }

    fn handle_expand_node(
        &mut self,
        maybe_target_id: Option<target::TargetId>,
        task: runtime_tasks::evaluate::EvaluateTask,
    ) {
        if task.task_id_equal("get-root") {
            if task
                .task_result
                .as_ref()
                .expect("return object should exists")
                .result
                .object_id
                .is_none()
            {
                let tab = self
                    .get_tab(maybe_target_id.as_ref())
                    .expect("tab should exists. Evaluate");
                tab.execute_one_task(task.into());
            } else {
                let tab = self
                    .get_tab(maybe_target_id.as_ref())
                    .expect("tab should exists. Evaluate");
                let expression = r##"let cc = document.querySelectorAll('#\\31 00016626 i.jstree-ocl');cc.forEach(function(cv){cv.click()});cc.length;"##;
                tab.evaluate_expression_named(expression, "expand-root");
                error!("****** result: {:?}", task.task_result);
            }
        } else if task.task_id_equal("expand-root") {
            let tab = self
                .get_tab(maybe_target_id.as_ref())
                .expect("tab should exists. Evaluate");
            let expression = r##"cc = document.querySelectorAll('#\\31 00016626 .jstree-closed i.jstree-ocl');cc.forEach(function(cv){cv.click()});cc.length;"##;
            let t1 = tab.evaluate_expression_task_prefixed(expression, "expand-node");
            tab.task_queue.add(t1, 1);
        } else if task.task_id_starts_with("expand-node") {
            self.recursive += 1;
            // task.task_result;
            if self.recursive < 3 {
                let tab = self
                    .get_tab(maybe_target_id.as_ref())
                    .expect("tab should exists. Evaluate");
                // https://doc.rust-lang.org/reference/tokens.html#literals
                // \u{001F}
                let expression = r##"cc = document.querySelectorAll('#\\31 00016626 .jstree-closed i.jstree-ocl');cc.forEach(function(cv){cv.click()});cc.length;"##;
                // let expression = "cc = document.querySelectorAll('#u{001F} 00016626 .jstree-closed i.jstree-ocl');cc.forEach(function(cv){cv.click()});cc.length;";
                let t1 = tab.evaluate_expression_task_prefixed(expression, "expand-node");
                tab.task_queue.add(t1, 1);
                info!("expand node result:::::::::::::::::::::::: {:?}", task);
            }
        }
    }

    fn get_tab(&mut self, target_id: Option<&target::TargetId>) -> Option<&mut Tab> {
        self.debug_session.get_tab_by_id_mut(target_id).ok()
    }
}


impl PrintToPdfDing {
    fn waiting_blank_page(
        &mut self,
        target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        match page_response {
            PageResponse::ChromeConnected => {
                self.debug_session.set_discover_targets(true);
                self.debug_session.set_ignore_certificate_errors(true);
            }
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::PageCreated(_page_idx) => {
                        let tab = self.get_tab(target_id).expect("tab should exists.");
                        tab.attach_to_page();
                    }
                    ReceivedEvent::PageAttached(_page_info, _session_id) => {
                        let tab = self
                            .get_tab(target_id)
                            .expect("tab should exists. PageAttached");
                        let t1 = tab.page_enable_task();
                        let t2 = tab.network_enable_task(None);
                        tab.execute_tasks(vec![t1, t2]);
                    }
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                if let MethodCallDone::PageEnabled(_task) = method_call_done {
                    self.state = PageState::WaitingLoginPage;
                    let url = self.url;
                    let tab = self
                        .get_tab(target_id)
                        .expect("tab should exists. RequestIntercepted");
                    tab.navigate_to(url);
                }
            }
            _ => {}
        }
    }
}

impl PrintToPdfDing {
    fn waiting_login_page(
        &mut self,
        target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        if let PageResponse::ReceivedEvent(ReceivedEvent::LoadEventFired(_monotonic_time)) =
            page_response
        {
            let tab = self
                .get_tab(target_id)
                .expect("tab should exists. LoadEventFired");
            // error!("load_event_fired: {:?}", tab);
            let url = tab.get_url();
            info!("url >>>>>>>>>>>>>>>>>>>>>>>>>>>>>  {:?}", url);
            let expression = r##"document.getElementsByClassName('login-tab').item(1).click();
                                                document.getElementById('login-by').value='13777272378';
                                                document.getElementById('password').value='00000132abc';
                                                document.getElementById('btn-submit-login').click();"##;
            let t1 = tab.evaluate_expression_task_named(expression, "login");
            let mut tk = tab.set_request_interception_task_named("a");
            vec!["*/api/*"].iter().for_each(|url| {
                tk.add_request_pattern(
                    Some(url),
                    Some(ResourceType::XHR),
                    Some(InterceptionStage::HeadersReceived),
                );
            });
            let t3: TaskDescribe = tk.into();
            tab.execute_tasks(vec![t3, t1]);
            self.state = PageState::TargetPageDisplayed;
        }
    }
}

impl PrintToPdfDing {
    fn page_displayed(
        &mut self,
        target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::LoadEventFired(_monotonic_time) => {
                        let tab = self
                            .get_tab(target_id)
                            .expect("tab should exists. LoadEventFired");
                        let url = tab.get_url();
                        info!("url >>>>>>>>>>>>>>>>>>>>>>>>>>>>>  {:?}", url);
                        let expression = "document.getElementById('100016626')";
                        tab.evaluate_expression_named(expression, "get-root");
                        info!("second event fired.");
                    }
                    ReceivedEvent::SetChildNodesOccurred(node_id) => {
                        error!(".......................{:?}", node_id);
                    }
                    ReceivedEvent::ResponseReceived(_event) => {}
                    ReceivedEvent::RequestWillBeSent(_event) => {}
                    ReceivedEvent::DataReceived(_event) => {}
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                if let MethodCallDone::Evaluate(task) = method_call_done {
                    self.handle_expand_node(target_id.cloned(), task);
                }
            }
            _ => {}
        }
    }
}

impl Future for PrintToPdfDing {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let target_id = page_response_wrapper.target_id.clone();
                if let PageResponse::SecondsElapsed(seconds) = page_response_wrapper.page_response {
                    trace!("seconds elapsed: {} ", seconds);
                    if seconds > 20 {
                        self.assert_result();
                        break Ok(().into());
                    }
                } else if let PageResponse::ReceivedEvent(ReceivedEvent::RequestIntercepted(
                    interception_id,
                )) = page_response_wrapper.page_response
                {
                    self.intercept_counter += 1;
                    let tab = self
                        .get_tab(target_id.as_ref())
                        .expect("tab should exists. RequestIntercepted");
                    let intercepted_request = tab
                        .request_intercepted
                        .get(&interception_id)
                        .expect("RequestIntercepted should exists.");
                    info!("----------------------------------------------------------------------{:?}", intercepted_request);
                    let request_id = intercepted_request.get_raw_parameters().request_id.clone();
                    tab.get_response_body_for_interception(interception_id, request_id);
                } else if let PageResponse::MethodCallDone(
                    MethodCallDone::GetResponseBodyForInterception(task),
                ) = page_response_wrapper.page_response
                {
                    let tab = self
                        .get_tab(target_id.as_ref())
                        .expect("tab should exists. RequestIntercepted");
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
                } else {
                    match self.state {
                        PageState::WaitingBlankPage => {
                            self.waiting_blank_page(
                                target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::WaitingLoginPage => {
                            self.waiting_login_page(
                                target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::TargetPageDisplayed => {
                            self.page_displayed(
                                target_id.as_ref(),
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
        // intercepted_requests: Vec::new(),
        ..PrintToPdfDing::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}
