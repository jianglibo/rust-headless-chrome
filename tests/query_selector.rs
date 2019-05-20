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
        let tab = self.debug_session.main_tab().expect("main tab should exists.");
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
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let mut tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
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
                    PageResponse::FrameNavigated(frame_id) => {
                        // error!("got frame_id: {:?}", frame_id);
                        let tab = tab.expect("tab should exists. FrameNavigated");
                        let frame = tab.find_frame_by_id(&frame_id)
                            .filter(|f| f.name == Some("ddlogin-iframe".into()));
                        // error!("got frame: {:?}", frame);
                        if frame.is_some() {
                            let tt = self.debug_session.first_page_mut().expect("first_page_mut should exists.");

                            tt.dom_query_selector_by_selector(self.selector, Some(100));
                            tt.dom_query_selector_by_selector("#not-existed", Some(102));
                            tt.dom_query_selector_by_selector(self.selector, Some(101));
                            
                        }
                    }
                    PageResponse::QuerySelectorDone(selector, node_id) => {
                        self.call_count += 1;
                        let tab = tab.expect("tab should exists. QuerySelectorDone");
                        if task_id == Some(102) {
                            assert_eq!(selector, "#not-existed");
                            assert_eq!(node_id, None);
                        } else if task_id == Some(100) {
                            self.task_id_100_called = true;
                        } else if task_id == Some(101) { // should got node_id, but not node.

                            let mut task_builder = tasks::DescribeNodeTaskBuilder::default();
                            task_builder.node_id(node_id).depth(10);
                            tab.describe_node(task_builder, Some(105));
                        } else {
                            assert!(node_id.is_none());
                        }
                    }
                    PageResponse::DescribeNodeDone(_selector, node_id) => {
                        let tab = tab.expect("tab should exists. DescribeNodeDone.");
                        if task_id == Some(105) {
                            let content_document = tab.find_node_by_id(node_id)
                                .and_then(|n| n.content_document.as_ref());

                            failure::ensure!(content_document.is_some(), "content_document should be some.");
                            let content_document_id = content_document.map(|cd| cd.node_id);
                            let mut task_builder = tasks::QuerySelectorTaskBuilder::default();
                            task_builder
                                .node_id(content_document_id)
                                .selector("#qrcode img"); // got nothing.
                            tab.query_selector(task_builder, Some(106));
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
fn t_dom_query_selector() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,query_selector=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let selector = "#ddlogin-iframe";
    let my_page = QuerySelector {
        url,
        selector,
        ..Default::default()
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).expect("tokio should success.")
}
