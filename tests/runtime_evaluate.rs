extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::ChangingFrame;
use headless_chrome::browser_async::page_message::PageResponse;
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

struct RuntimeEvaluate {
    debug_session: DebugSession,
    url: &'static str,
    task_100_called: bool,
    task_101_called: bool,
    runtime_execution_context_created_count: u8,
}

impl RuntimeEvaluate {
    fn assert_result(&self) {
        assert!(self.task_100_called);
        assert!(self.task_101_called);
        assert_eq!(self.runtime_execution_context_created_count, 8);
    }
}

impl Future for RuntimeEvaluate {
    type Item = ();
    type Error = failure::Error;

    #[allow(clippy::cognitive_complexity)]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, task_id, value)) = try_ready!(self.debug_session.poll()) {
                let tab = if let Some(tid) = &tab_id {
                    self.debug_session.get_tab_by_id_mut(Some(tid))
                } else {
                    None
                };
                match value {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        assert!(tab.is_some());
                        let tab = tab.unwrap();
                        tab.navigate_to(self.url, None);
                    }
                    PageResponse::FrameNavigated(frame_id) => {
                        let tab = tab.unwrap();
                        let frame = tab.find_frame_by_id(&frame_id).unwrap();
                        info!("got frame: {:?}", frame_id);
                        if frame.name == Some("ddlogin-iframe".into()) {
                            if let Some(tab) = self.debug_session.main_tab_mut() {
                                tab.runtime_evaluate("3+3".into(), Some(100));
                                tab.runtime_evaluate("3::0".into(), Some(101));
                                tab.runtime_enable(Some(102));
                                tab.runtime_evaluate(r#"var iframe = document.getElementById("ddlogin-iframe");iframe.contentDocument;"#.into(), Some(103));
                            }
                        }
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
                        _ => unreachable!(),
                    },
                    PageResponse::RuntimeExecutionContextCreated(
                        runtime_execution_context_created,
                    ) => {
                        info!("{:?}", runtime_execution_context_created);
                        self.runtime_execution_context_created_count += 1;
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            self.assert_result();
                            break Ok(().into());
                        }
                    }
                    _ => {
                        info!("got unused page message {:?}", value);
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

// [2019-04-15T08:12:35Z ERROR headless_chrome::browser_async::chrome_debug_session] unprocessed ReceivedMessageFromTargetEvent { params: ReceivedMessageFromTargetParams { session_id: "B1B0F5851D30241BC39BE00415D4F43A", target_id: "C9CB934D0B4C63978622ED5F01D6B829", message: "{\"method\":\"DOM.setChildNodes\",\"params\":{\"parentId\":1,\"nodes\":[{\"nodeId\":2,\"parentId\":1,\"backendNodeId\":5,\"nodeType\":10,\"nodeName\":\"html\",\"localName\":\"\",\"nodeValue\":\"\",\"publicId\":\"\",\"systemId\":\"\"},{\"nodeId\":3,\"parentId\":1,\"backendNodeId\":6,\"nodeType\":1,\"nodeName\":\"HTML\",\"localName\":\"html\",\"nodeValue\":\"\",\"childNodeCount\":2,\"attributes\":[],\"frameId\":\"C9CB934D0B4C63978622ED5F01D6B829\"}]}}" } }

#[test]
fn t_runtime_evaluate() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,runtime_evaluate=info");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let my_page = RuntimeEvaluate {
        debug_session: Default::default(),
        url,
        task_100_called: false,
        task_101_called: false,
        runtime_execution_context_created_count: 0,
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
