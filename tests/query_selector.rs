extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{dom};

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageResponse};
use headless_chrome::browser_async::debug_session::{DebugSession};
use headless_chrome::browser_async::page_message::{ChangingFrame};
use tokio;
use std::default::Default;

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
        assert_eq!(self.call_count, 3);
        assert!(self.task_id_100_called);
        assert!(tab.temporary_node_holder.len() >= 7);
        info!("all nodes: {:?}", tab.temporary_node_holder);
        // tab.temporary_node_holder.values().for_each(|v| v.iter().for_each(|nd| assert_eq!(nd.node_name, "IFRAME")));
        assert!(self.found_node_id.is_some());
    }
}

impl Future for QuerySelector {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, task_id, value)) = try_ready!(self.debug_session.poll()) {
                let tab = if let Some(tid) = &tab_id {
                    self.debug_session.get_tab_by_id_mut(tid)
                } else {
                    None
                };
                match value {
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        assert!(tab.is_some());
                        let tab = tab.unwrap();
                        tab.navigate_to(self.url);
                    },
                    PageResponse::FrameNavigated(changing_frame) => {
                        info!("got frame: {:?}", changing_frame);
                        if let ChangingFrame::Navigated(frame) = changing_frame {
                            if frame.name == Some("ddlogin-iframe".into()) {
                                if let Some(tab) = self.debug_session.main_tab_mut() {
                                    tab.dom_query_selector_by_selector(self.selector, Some(100));
                                    tab.dom_query_selector_by_selector("#not-existed", Some(102));
                                    tab.dom_query_selector_by_selector(self.selector, Some(101));
                                }
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
                        } else {
                            assert_eq!(task_id, Some(101));
                            let tab = tab.unwrap();
                            self.found_node_id = node_id;
                            let node = tab.find_node_by_id(node_id.unwrap());
                            assert!(node.is_some());
                        }
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            self.assert_result();
                            break Ok(().into())
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
fn t_dom_query_selector() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,query_selector=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let selector = "#ddlogin-iframe";
    let my_page = QuerySelector {
        debug_session: Default::default(),
        url,
        selector,
        found_node_id: None,
        call_count: 0,
        task_id_100_called: false,
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    if let Err(err) = runtime.block_on(my_page.into_future()) {
        error!("err: {:?}", err);
    }
}