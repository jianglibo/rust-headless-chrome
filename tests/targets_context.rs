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
struct TargetsContext {
    debug_session: DebugSession,
    url: &'static str,
    page_enabled_be_called: u8,
}

impl TargetsContext {
    fn assert_result(&self) {
        assert_eq!(self.page_enabled_be_called, 1);
    }
}

impl Future for TargetsContext {
    type Item = ();
    type Error = failure::Error;

    #[allow(clippy::cognitive_complexity)]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, _task_id, value)) = try_ready!(self.debug_session.poll()) {
                let tab = self.debug_session.get_tab_by_id_mut(tab_id.as_ref()).ok();
                match value {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageCreated(page_idx) => {
                        let tab = tab.unwrap();
                        tab.attach_to_page();
                        if page_idx == 1 {
                            tab.name_the_page("abc");
                            self.debug_session.create_new_tab("https://pc.xuexi.cn");
                        }
                    }
                    PageResponse::PageAttached(_page_info, _session_id) => {
                        let tab = tab.unwrap();
                        tab.page_enable();
                        if tab.page_name == Some("abc") {
                            tab.navigate_to(self.url, None);
                        }
                    }
                    PageResponse::PageEnable => {
                        self.page_enabled_be_called += 1;
                        info!("page enabled.");
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
                    PageResponse::RuntimeEvaluate(_result, _exception_details) => {},
                    PageResponse::RuntimeGetProperties(return_object) => {
                        let get_properties_return_object =
                            return_object.expect("should return get_properties_return_object");
                        info!(
                            "property count: {:?}",
                            get_properties_return_object.result.len()
                        );
                    }
                    PageResponse::RuntimeExecutionContextCreated(frame_id) => {
                        info!(
                            "execution context created, frame_id: <<<<<<<<{:?}",
                            frame_id
                        );
                    }
                    PageResponse::FrameStoppedLoading(_frame_id) => {
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 35 {
                            self.assert_result();
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
fn t_target_context() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,runtime_evaluate=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn";

    let my_page = TargetsContext {
        url,
        ..Default::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
