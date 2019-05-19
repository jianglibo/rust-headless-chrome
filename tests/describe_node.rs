extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::dom;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::PageResponse;
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct DescribeNode {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    node_id: Option<dom::NodeId>,
    node: Option<dom::Node>,
}

impl DescribeNode {
    fn assert_result(&self) {
        assert!(self.node_id.is_some());
        assert!(self.node.is_some());
        info!("describe node return: {:?}", self.node);
    }
}

impl Future for DescribeNode {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
                let task_id = page_response_wrapper.task_id;
                match page_response_wrapper.page_response {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        tab.unwrap().navigate_to(self.url, None);
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            self.assert_result();
                            break Ok(().into());
                        }
                    }
                    PageResponse::FrameNavigated(frame_id) => {
                        let tab = tab.unwrap();
                        let frame = tab.find_frame_by_id(&frame_id).unwrap();
                        info!("got frame: {:?}", frame_id);
                        if frame.name == Some("ddlogin-iframe".into()) {
                            if let Some(tab) = self.debug_session.first_page_mut() {
                                tab.describe_node_by_selector(self.selector, Some(2), Some(100));
                            }
                        }
                    }
                    PageResponse::DescribeNodeDone(selector, node_id) => {
                        if task_id == Some(101) {
                            assert!(node_id.is_none());
                            info!("{:?}", selector);
                        } else {
                            assert!(task_id == Some(100));
                            assert!(node_id.is_some());
                            assert_eq!(selector, Some(self.selector.into()));
                            self.node_id = node_id;
                            self.node = tab.unwrap().find_node_by_id(node_id).cloned();
                            info!(
                                "content document: {:?}",
                                self.node.as_ref().unwrap().content_document
                            );
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
fn t_dom_describe_node() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,describe_node=info");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let selector = "#ddlogin-iframe";
    let my_page = DescribeNode {
        url,
        selector,
        ..Default::default()
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    if let Err(error) = runtime.block_on(my_page.into_future()) {
        error!("{:?}", error)
    }
}
