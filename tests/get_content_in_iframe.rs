#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::{debug_session::DebugSession, EventName, Tab, WaitingForPageAttachTaskName};

use headless_chrome::browser_async::page_message::{
    write_base64_str_to, MethodCallDone, PageResponse, ReceivedEvent,
};
use headless_chrome::browser_async::task_describe::{RuntimeEvaluateTaskBuilder, HasTaskId};
use headless_chrome::protocol::target;
use log::*;
use std::default::Default;

use std::fs;
use std::path::Path;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

enum PageState {
    WaitingBlankPage,
    LoginPageDisplayed,
    WaitingForQrcodeScan,
}


impl Default for PageState {
    fn default() -> Self {
        PageState::WaitingBlankPage
    }
}


#[derive(Default)]
struct GetContentInIframe {
    debug_session: DebugSession,
    url: &'static str,
    ddlogin_frame_stopped_loading: bool,
    state: PageState,
}


impl GetContentInIframe {

    fn get_tab(&mut self, target_id: Option<&target::TargetId>) -> Option<&mut Tab> {
        self.debug_session.get_tab_by_id_mut(target_id).ok()
    }

    fn waiting_blank_page(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
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
                        let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                        let tasks = vec![
                            WaitingForPageAttachTaskName::PageEnable,
                            WaitingForPageAttachTaskName::RuntimeEnable,
                            WaitingForPageAttachTaskName::NetworkEnable];
                        tab.attach_to_page_and_then(tasks);
                    }
                    ReceivedEvent::PageAttached(_page_info, _session_id) => {
                        // let tab = self
                        //     .get_tab(target_id)
                        //     .expect("tab should exists. PageAttached");
                        // tab.runtime_enable();
                        // tab.page_enable();
                    }
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                if let MethodCallDone::PageEnabled(_task) = method_call_done {
                    self.state = PageState::LoginPageDisplayed;
                    let url = self.url;
                    let tab = self
                        .get_tab(maybe_target_id)
                        .expect("tab should exists. RequestIntercepted");
                    tab.navigate_to(url);
                }
            }
            _ => {}
        }
    }
}


impl GetContentInIframe {
    fn login_page_displayed(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::FrameStoppedLoading(frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        if let Some(_frame) = tab
                            .find_frame_by_id(&frame_id)
                            .filter(|f| f.name == Some("ddlogin-iframe".into()))
                        {
                            if let Some(context) =
                                tab.find_execution_context_id_by_frame_name("ddlogin-iframe")
                            {
                                info!("execution_context_description: {:?}", context);
                                let mut tb = RuntimeEvaluateTaskBuilder::default();
                                tb.expression(r#"document.querySelector('div#qrcode.login_qrcode_content img').getAttribute('src')"#).context_id(context.id);
                                let t1 = tab.evaluate_task_named(tb, "get-img-object");
                                tab.execute_one_task(t1);
                                self.ddlogin_frame_stopped_loading = true;
                            } else {
                                error!("cannot find execution_context_description");
                            }
                        }
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
                    let file_name = "target/qrcode.png";
                    let path = Path::new(file_name);
                    if path.exists() && path.is_file() {
                        fs::remove_file(file_name).unwrap();
                    }

                    let base64_data = task.get_string_result().and_then(|v| {
                        let mut ss = v.splitn(2, ',').fuse();
                        ss.next();
                        ss.next()
                    });

                    write_base64_str_to(file_name, base64_data)
                        .expect("write_base64_str_to success.");
                    assert!(path.exists());
                    self.state = PageState::WaitingForQrcodeScan;
                    let exe = std::fs::canonicalize("./target/qrcode.png").expect("exists.");
                    error!("{:?}", exe);
                    std::process::Command::new("cmd")
                        .args(&["/C", "C:/Program Files/internet explorer/iexplore.exe", exe.to_str().expect("should convert to string.")])
                        .spawn()
                        .expect("failed to execute process");
                }
            }
            _ => {}
        }
    }
}

impl GetContentInIframe {
    fn waiting_for_qrcode_scan(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        let expression = r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').length"##;
        let get_children_number = "get-children-number";
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::PageCreated(_page_idx) => {
                        let tab = self.get_tab(maybe_target_id).expect("tab should exists.");
                        let tasks = vec![
                            WaitingForPageAttachTaskName::PageEnable,
                            WaitingForPageAttachTaskName::RuntimeEnable,
                            WaitingForPageAttachTaskName::NetworkEnable];
                        tab.attach_to_page_and_then(tasks);
                    }
                    ReceivedEvent::FrameStoppedLoading(_frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        info!("url current: {:?}", tab.get_url());
                        if tab.get_url() == "https://www.xuexi.cn/" {
                            tab.evaluate_expression_named(expression, get_children_number);
                        }
                    }
                    ReceivedEvent::ResponseReceived(_event) => {}
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
            }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                match method_call_done {
                    MethodCallDone::Evaluate(task) => {
                        info!("{:?}", task);
                        if task.task_id_equal(get_children_number) {
                        if let Some(v) = task.get_u64_result() {
                            if v < 16 {
                                let tab = self
                                    .get_tab(maybe_target_id)
                                    .expect("tab should exists. FrameStoppedLoading");
                                tab.evaluate_expression_named(expression, get_children_number);
                            } else {
                                info!("^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ready.............");
                                let tab = self
                                    .get_tab(maybe_target_id)
                                    .expect("tab should exists. FrameStoppedLoading");
                                let fm = |i: u64| {
                                    format!(r##"document.querySelectorAll('#\\32 31c div.grid-cell span.text').item({}).click()"##, i)
                                };
                                for i in 0..15 {
                                    let exp = fm(i);
                                    let slice = exp.as_str();
                                    let t1 = tab.evaluate_expression_task(slice);
                                    tab.task_queue.add(t1, i + 2);
                                }
                            }
                        } else {
                            panic!("unexpected call return.");
                        }
                    } else {
                        info!("{:?}", task);
                    }
                    } 
                    MethodCallDone::GetResponseBodyForInterception(_task) => {}
                    _ => {}
                }
                // info!("{:?}", method_call_done);
            }
            _ => {}
        }
    }
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
                    trace!("seconds elapsed: {} ", seconds);
                    self.debug_session.activates_next_in_interval(180);
                    if seconds > 12000 {
                        self.debug_session.tabs.iter().for_each(|tb|info!("{:?}", tb));
                        assert_eq!(self.debug_session.tabs.len(), 19);
                        let m = self.debug_session.tabs.iter().filter(|tb|tb.get_url() == "https://www.xuexi.cn/").count();
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
    ::std::env::set_var(
        "RUST_LOG",
        "headless_chrome=trace,get_content_in_iframe=trace",
    );
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let my_page = GetContentInIframe {
        url,
        ..Default::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}
