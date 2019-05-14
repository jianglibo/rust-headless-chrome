extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::{write_base64_str_to, PageResponse};
use headless_chrome::browser_async::task_describe as tasks;
use headless_chrome::protocol::page;
use log::*;
use serde_json;

use std::fs;
use std::path::Path;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct RuntimeEvaluate {
    debug_session: DebugSession,
    url: &'static str,
    task_100_called: bool,
    task_101_called: bool,
    runtime_execution_context_created_count: u8,
    ddlogin_frame_stopped_loading: bool,
}

impl RuntimeEvaluate {
    fn assert_result(&self) {
        assert!(self.task_100_called);
        assert!(self.task_101_called);
        assert!(self.runtime_execution_context_created_count > 7);
        assert!(self.ddlogin_frame_stopped_loading);
    }
}

impl Future for RuntimeEvaluate {
    type Item = ();
    type Error = failure::Error;

    #[allow(clippy::cognitive_complexity)]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, task_id, value)) = try_ready!(self.debug_session.poll()) {
                let tab = self.debug_session.get_tab_by_id_mut(tab_id.as_ref()).ok();
                match value {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        assert!(tab.is_some());
                        let url = self.url;
                        tab.map(|t| t.navigate_to(url, None));
                    }
                    PageResponse::FrameNavigated(frame_id) => {
                        let frame =
                            tab.and_then(|t| t.find_frame_by_id(&frame_id))
                                .and_then(|frame| {
                                    if frame.name == Some("ddlogin-iframe".into()) {
                                        Some(frame)
                                    } else {
                                        None
                                    }
                                });

                        if frame.is_some() {
                            self.debug_session.main_tab_mut().map(|tab|{
                                tab.runtime_evaluate_expression("3+3".into(), Some(100));
                                tab.runtime_evaluate_expression("3::0".into(), Some(101));
                                tab.runtime_enable(Some(102));
                                tab.runtime_evaluate_expression(r#"var iframe = document.getElementById("ddlogin-iframe");iframe.contentDocument;"#.into(), Some(103));
                                ()
                            });
                        }
                    }
                    PageResponse::RuntimeCallFunctionOn(result) => {
                        info!("got call result: {:?}", result);
                        let file_name = "target/qrcode.png";
                        let path = Path::new(file_name);
                        if path.exists() && path.is_file() {
                            fs::remove_file(file_name).unwrap();
                        }
                        let base64_data = result
                            .as_ref()
                            .map(|rn| &rn.result)
                            .map(|ro| &ro.value)
                            .and_then(Option::as_ref)
                            .and_then(serde_json::value::Value::as_str)
                            .and_then(|v| {
                                let mut ss = v.splitn(2, ',').fuse();
                                ss.next();
                                ss.next()
                            });

                        write_base64_str_to(file_name, base64_data).unwrap();
                        assert!(path.exists());
                    }
                    PageResponse::RuntimeEvaluate(result, exception_details) => match task_id {
                        Some(100) => {
                            self.task_100_called = true;
                            assert!(result.is_some());
                            let ro = result.unwrap();
                            assert_eq!(ro.object_type, "number");
                            assert_eq!(ro.value, Some(6.into()));
                            assert_eq!(ro.description, Some("6".into()));
                            assert!(exception_details.is_none());
                        }
                        Some(101) => {
                            self.task_101_called = true;
                            assert!(result.is_some());
                            let ro = result.unwrap();
                            assert_eq!(ro.object_type, "object");
                            assert_eq!(ro.subtype, Some("error".into()));
                            assert_eq!(ro.class_name, Some("SyntaxError".into()));
                            assert_eq!(
                                ro.description,
                                Some("SyntaxError: Unexpected token :".into())
                            );
                            assert!(exception_details.is_some());
                        }
                        Some(102) | Some(103) => {
                            info!("task id: {:?}, {:?}", task_id, result);
                            info!("task id: {:?}, {:?}", task_id, exception_details);
                        }
                        Some(110) => {
                            info!("task id: {:?}, {:?}", task_id, result);
                            assert!(result.is_some());
                            let result = result.unwrap();
                            let tab = tab.unwrap();
                            let object_id = result.object_id.expect("object_id should exists.");
                            tab.runtime_get_properties_by_object_id(object_id.clone(), Some(111));

                            let mut task = tasks::RuntimeCallFunctionOnTaskBuilder::default();
                            let fnd = "function() {return this.getAttribute('src');}";
                            task.object_id(object_id.clone()).function_declaration(fnd);
                            tab.runtime_call_function_on(task, Some(112));
                        }
                        _ => unreachable!(),
                    },
                    PageResponse::RuntimeGetProperties(return_object) => {
                        let get_properties_return_object =
                            return_object.expect("should return get_properties_return_object");
                        info!(
                            "property count: {:?}",
                            get_properties_return_object.result.len()
                        );
                        // let src: std::collections::HashSet<String> = ["src", "currentSrc", "outerHTML"].iter().map(|&v|v.to_string()).collect();
                        // get_properties_return_object.result.iter().filter(|pd|src.contains(&pd.name)).for_each(|pd| info!("property name: {:?}, value: {:?}", pd.name, pd.value));
                    }
                    PageResponse::RuntimeExecutionContextCreated(frame_id) => {
                        info!(
                            "execution context created, frame_id: <<<<<<<<{:?}",
                            frame_id
                        );
                        self.runtime_execution_context_created_count += 1;
                    }
                    PageResponse::FrameStoppedLoading(frame_id) => {
                        let frame = tab
                            .as_ref()
                            .and_then(|t| t.find_frame_by_id(&frame_id))
                            .filter(|f| f.name == Some("ddlogin-iframe".into()));

                        if frame.is_some() {
                            let context = tab.as_ref().and_then(|t| {
                                t.find_execution_context_id_by_frame_name("ddlogin-iframe")
                            });
                            if context.is_some() {
                                info!("execution_context_description: {:?}", context);
                                self.ddlogin_frame_stopped_loading = true;
                                let mut tb = tasks::RuntimeEvaluateTaskBuilder::default();
                                tb.expression(r#"document.querySelector("div#qrcode.login_qrcode_content img")"#).context_id(context.unwrap().id);
                                tab.map(|t| t.runtime_evaluate(tb, Some(110)));
                            }
                        }
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 35 {
                            self.assert_result();
                            // break Ok(().into());
                        }
                    }
                    _ => {
                        trace!("got unused page message {:?}", value);
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn t_runtime_evaluate() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,runtime_evaluate=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let my_page = RuntimeEvaluate {
        url,
        ..Default::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
