#![warn(clippy::all)]

extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::{debug_session::DebugSession, EventName, Tab};

use headless_chrome::browser_async::page_message::{MethodCallDone, PageResponse, ReceivedEvent};
use headless_chrome::browser_async::task_describe::{
    runtime_tasks, HasTaskId, RuntimeEvaluateTask, RuntimeEvaluateTaskBuilder, TaskDescribe,
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
    TargetPageDisplayed,
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
    task_100_called: bool,
    task_101_called: bool,
    ddlogin_frame_stopped_loading: bool,
    state: PageState,
}


impl GetContentInIframe {

    fn get_tab(&mut self, target_id: Option<&target::TargetId>) -> Option<&mut Tab> {
        self.debug_session.get_tab_by_id_mut(target_id).ok()
    }

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
                    self.state = PageState::TargetPageDisplayed;
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


impl GetContentInIframe {
    fn page_displayed(
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
                                tb.expression(r#"document.querySelector("div#qrcode.login_qrcode_content img")"#).context_id(context.id);
                                let t1 = tab.evaluate_task_named(tb, "get-img-object");
                                tab.execute_one_task(t1);
                                self.ddlogin_frame_stopped_loading = true;
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
                    // self.handle_expand_node(target_id.cloned(), task);
                }
            }
            _ => {}
        }
    }
}

impl GetContentInIframe {
    fn assert_result(&self) {
        assert!(self.task_100_called);
        assert!(self.task_101_called);
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
                    if seconds > 20 {
                        let tab = self
                            .debug_session
                            .first_page_mut()
                            .expect("tab should exists.");
                        assert_eq!(
                            tab.event_statistics
                                .happened_count(EventName::ExecutionContextCreated),
                            7
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
                        PageState::TargetPageDisplayed => {
                            self.page_displayed(
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
fn t_get_content_in_iframe() {
    ::std::env::set_var(
        "RUST_LOG",
        "headless_chrome=info,get_content_in_iframe=trace",
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

//     if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
//         let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
//         let target_id = page_response_wrapper.target_id.clone();
//         let task_id = page_response_wrapper.task_id;
//         match page_response_wrapper.page_response {
//             PageResponse::ChromeConnected => {
//                 self.debug_session.set_discover_targets(true);
//             }
//             PageResponse::PageCreated(page_idx) => {
//                 let tab = tab.expect("tab should exists.");
//                 tab.attach_to_page();
//             }
//             PageResponse::PageAttached(_page_info, _session_id) => {
//                 let tab = tab.expect("tab should exists. PageAttached");
//                 tab.page_enable();
//                 tab.navigate_to(self.url, None);
//             }
//             PageResponse::PageEnabled => {}
//             PageResponse::FrameNavigated(frame_id) => {
//                 let tab = tab.expect("tab should exists. FrameNavigated");
//                 let frame =
//                     tab.find_frame_by_id(&frame_id)
//                         .and_then(|frame| {
//                             if frame.name == Some("ddlogin-iframe".into()) {
//                                 Some(frame)
//                             } else {
//                                 None
//                             }
//                         });

//                 if frame.is_some() {
//                     let mtab = self.debug_session.first_page_mut().expect("main tab should exists.");
//                     mtab.runtime_evaluate_expression("3+3", Some(100));
//                     mtab.runtime_evaluate_expression("3::0", Some(101));
//                     mtab.runtime_enable(Some(102));
//                     mtab.runtime_evaluate_expression(r#"var iframe = document.getElementById("ddlogin-iframe");iframe.contentDocument;"#, Some(103));
//                 }
//             }
//             PageResponse::CallFunctionOnDone(result) => {
//                 info!("got call result: {:?}", result);
//                 let file_name = "target/qrcode.png";
//                 let path = Path::new(file_name);
//                 if path.exists() && path.is_file() {
//                     fs::remove_file(file_name).unwrap();
//                 }
//                 let base64_data = result
//                     .as_ref()
//                     .map(|rn| &rn.result)
//                     .map(|ro| &ro.value)
//                     .and_then(Option::as_ref)
//                     .and_then(serde_json::value::Value::as_str)
//                     .and_then(|v| {
//                         let mut ss = v.splitn(2, ',').fuse();
//                         ss.next();
//                         ss.next()
//                     });

//                 write_base64_str_to(file_name, base64_data).expect("write_base64_str_to success.");
//                 assert!(path.exists());
//             }
//             PageResponse::EvaluateDone(evaluate_result) => match task_id {
//                 Some(100) => {
//                     self.task_100_called = true;
//                     let evaluate_result = evaluate_result.expect("evaluate_result should exists.");
//                     let ro = evaluate_result.result;
//                     assert_eq!(ro.object_type, "number");
//                     assert_eq!(ro.value, Some(6.into()));
//                     assert_eq!(ro.description, Some("6".into()));
//                     assert!(evaluate_result.exception_details.is_none());
//                 }
//                 Some(101) => {
//                     self.task_101_called = true;
//                     let evaluate_result = evaluate_result.expect("evaluate_result should exists. 101");
//                     let ro = evaluate_result.result;
//                     assert_eq!(ro.object_type, "object");
//                     assert_eq!(ro.subtype, Some("error".into()));
//                     assert_eq!(ro.class_name, Some("SyntaxError".into()));
//                     assert_eq!(
//                         ro.description,
//                         Some("SyntaxError: Unexpected token :".into())
//                     );
//                     assert!(evaluate_result.exception_details.is_some());
//                 }
//                 Some(102) | Some(103) => {
//                     info!("task id:{:?}, {:?}", task_id, evaluate_result);
//                 }
//                 Some(110) => {
//                     info!("task id: {:?}, {:?}", task_id, evaluate_result);
//                     let result = evaluate_result.expect("evaluate_result should exists. 110").result;
//                     let tab = tab.expect("tab should exists. EvaluateDone");
//                     let object_id = result.object_id.expect("object_id should exists.");
//                     tab.runtime_get_properties_by_object_id(object_id.clone(), Some(111));

//                     let mut task = tasks::RuntimeCallFunctionOnTaskBuilder::default();
//                     let fnd = "function() {return this.getAttribute('src');}";
//                     task.object_id(object_id.clone()).function_declaration(fnd);
//                     tab.runtime_call_function_on(task, Some(112));
//                 }
//                 _ => unreachable!(),
//             },
//             PageResponse::GetPropertiesDone(return_object) => {
//                 let get_properties_return_object =
//                     return_object.expect("should return get_properties_return_object");
//                 info!(
//                     "property count: {:?}",
//                     get_properties_return_object.result.len()
//                 );
//                 // let src: std::collections::HashSet<String> = ["src", "currentSrc", "outerHTML"].iter().map(|&v|v.to_string()).collect();
//                 // get_properties_return_object.result.iter().filter(|pd|src.contains(&pd.name)).for_each(|pd| info!("property name: {:?}, value: {:?}", pd.name, pd.value));
//             }
//             PageResponse::RuntimeExecutionContextCreated(frame_id) => {
//                 info!(
//                     "execution context created, frame_id: <<<<<<<<{:?}",
//                     frame_id
//                 );
//                 self.runtime_execution_context_created_count += 1;
//             }
//             PageResponse::FrameStoppedLoading(frame_id) => {
//                 let tab = tab.expect("tab should exists. FrameStoppedLoading");
//                 let frame = tab.find_frame_by_id(&frame_id)
//                     .filter(|f| f.name == Some("ddlogin-iframe".into()));

//                 if frame.is_some() {
//                     let context = tab.find_execution_context_id_by_frame_name("ddlogin-iframe");
//                     if context.is_some() {
//                         info!("execution_context_description: {:?}", context);
//                         self.ddlogin_frame_stopped_loading = true;
//                         let mut tb = tasks::GetContentInIframeTaskBuilder::default();
//                         tb.expression(r#"document.querySelector("div#qrcode.login_qrcode_content img")"#).context_id(context.unwrap().id);
//                         tab.runtime_evaluate(tb, Some(110));
//                     }
//                 }
//             }
//             PageResponse::SecondsElapsed(seconds) => {
//                 trace!("seconds elapsed: {} ", seconds);
//                 if seconds > 20 {
//                     self.assert_result();
//                     break Ok(().into());
//                 }
//             }
//             _ => {
//                 warn!("got unused page message {:?}", page_response_wrapper);
//             }
//         }
//     } else {
//         error!("got None, was stream ended?");
//     }
// }
