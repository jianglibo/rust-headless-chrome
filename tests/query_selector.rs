extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::dom;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::PageResponse;
use headless_chrome::browser_async::task_describe as tasks;
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct QuerySelector {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    found_node_id: Option<dom::NodeId>,
    call_count: u8,
    task_id_100_called: bool,
}

impl QuerySelector {
    fn assert_result(&self) {
        let tab = self.debug_session.main_tab().unwrap();
        assert_eq!(self.call_count, 4);
        assert!(self.task_id_100_called);
        assert!(tab.temporary_node_holder.len() >= 7);
        assert!(self.found_node_id.is_some());
    }
}

impl Future for QuerySelector {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, task_id, value)) = try_ready!(self.debug_session.poll()) {
                let mut tab = self.debug_session.get_tab_by_id_mut(tab_id.as_ref()).ok();
                match value {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        assert!(tab.is_some());
                        if let Some(t) = tab {
                            t.navigate_to(self.url, None);
                        }
                    }
                    PageResponse::FrameNavigated(frame_id) => {
                        let frame = tab
                            .and_then(|t| t.find_frame_by_id(&frame_id))
                            .filter(|f| f.name == Some("ddlogin-iframe".into()));
                        if frame.is_some() {
                            if let Some(tab) = self.debug_session.main_tab_mut() {
                                tab.dom_query_selector_by_selector(self.selector, Some(100));
                                tab.dom_query_selector_by_selector("#not-existed", Some(102));
                                tab.dom_query_selector_by_selector(self.selector, Some(101));
                            }
                        }
                    }
                    PageResponse::QuerySelector(selector, node_id) => {
                        self.call_count += 1;
                        if task_id == Some(102) {
                            assert_eq!(selector, "#not-existed");
                            assert_eq!(node_id, None);
                        } else if task_id == Some(100) {
                            self.task_id_100_called = true;
                        } else if task_id == Some(101) {
                            assert_eq!(task_id, Some(101));

                            let content_document = tab
                                .as_ref()
                                .and_then(|t| t.find_node_by_id(node_id))
                                .and_then(|n| n.content_document.as_ref());

                            let backend_node_id = content_document.map(|cd| cd.backend_node_id);
                            let content_document_id = content_document.map(|cd| cd.node_id);
                            let mut task_builder = tasks::DescribeNodeTaskBuilder::default();
                            task_builder.backend_node_id(backend_node_id).depth(10);
                            tab.as_mut()
                                .map(|t| t.describe_node(task_builder, Some(105)));
                            let mut task_builder = tasks::QuerySelectorTaskBuilder::default();
                            task_builder
                                .node_id(content_document_id)
                                .selector("#qrcode img"); // got nothing.
                            tab.map(|t| t.query_selector(task_builder, Some(106)));
                        } else {
                            assert_eq!(Some(0), node_id);
                            info!("got node_id in frame: {:?}", node_id);
                        }
                    }
                    PageResponse::DescribeNode(_selector, node_id) => {
                        if task_id == Some(105) {
                            if let Some(t) = tab {
                                info!(
                                    "got node by backend_node_id: {:?}",
                                    t.find_node_by_id(node_id)
                                );
                            }
                        }
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            self.assert_result();
                            break Ok(().into());
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
fn t_dom_query_selector() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,query_selector=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let selector = "#ddlogin-iframe";
    let my_page = QuerySelector {
        url,
        selector,
        ..Default::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
