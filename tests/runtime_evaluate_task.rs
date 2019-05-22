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
struct RuntimeEvaluateTask {
    debug_session: DebugSession,
    url: &'static str,
}

impl RuntimeEvaluateTask {
}

impl Future for RuntimeEvaluateTask {
    type Item = ();
    type Error = failure::Error;

    #[allow(clippy::cognitive_complexity)]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
                let task_id = page_response_wrapper.task_id;
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
                    PageResponse::EvaluateDone(evaluate_result) => {
                        info!("got result: {:?}", evaluate_result);
                        let tab = tab.expect("tab should exists. EvaluateDone");
                        
                        if let Some(oid) = evaluate_result.and_then(|ro| ro.result.object_id) {
                            tab.runtime_get_properties_by_object_id(oid, Some(111))
                        }
                        
                    }
                    PageResponse::LoadEventFired(_monotonic_time) => {
                        info!("loadEventFired, start invoke method.");
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds == 8 {
                            if let Some(mt) = self.debug_session.first_page_mut() {
                                mt.runtime_evaluate_expression(
                                    r#"document.querySelectorAll("div.text-link-item-title .text-wrap .text")"#,
                                    // r#"document.querySelectorAll("div.text-link-item-title .text-wrap .text").item(0).click()"#,
                                    Some(101),
                                )
                            }
                        }
                        if seconds > 19 {
                            break Ok(().into());
                        }
                    }
                    PageResponse::GetPropertiesDone(return_object) => {
                        if let Some(t) = tab {
                            if task_id == Some(111) {
                                let get_properties_return_object =
                                    return_object.expect("should return get_properties_return_object");
                                get_properties_return_object
                                    .result
                                    .iter()
                                    .take(1)
                                    .map(|pd| pd.value.as_ref().and_then(|v| v.object_id.as_ref()))
                                    .for_each(|oid| {
                                        if let Some(oid) = oid {
                                            info!("oid {:?}", oid);
                                            t.runtime_get_properties_by_object_id(oid.clone(), None);
                                        }
                                    });
                            } else {
                                info!("got item: {:?}", return_object);
                            }
                        }
                    }
                    _ => {
                        trace!("got unused page message {:?}", page_response_wrapper);
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn t_runtime_evaluate_task() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,runtime_evaluate_task=trace");
    env_logger::init();
    let url = "https://www.xuexi.cn/";
    // div.text-link-item-title
    // "span:not(.my-points-red).my-points-points"
    let my_page = RuntimeEvaluateTask {
        url,
        ..Default::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
