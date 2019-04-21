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
use headless_chrome::browser_async::tab::{Tab};
use tokio;
use std::default::Default;
// use serde_json;


fn assert_result(tab: &Tab, found_node_id: Option<dom::NodeId>) {
    assert_eq!(tab.temporary_node_holder.len() , 7);
    info!("all nodes: {:?}", tab.temporary_node_holder);
    // tab.temporary_node_holder.values().for_each(|v| v.iter().for_each(|nd| assert_eq!(nd.node_name, "IFRAME")));
    assert!(found_node_id.is_some());
}


struct QuerySelector {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    found_node_id: Option<dom::NodeId>,
}

impl Future for QuerySelector {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, _task_id, value)) = try_ready!(self.debug_session.poll()) {
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
                                }
                            }
                        }
                    }
                    PageResponse::QuerySelector(_selector, node_id) => {
                        let tab = tab.unwrap();
                        self.found_node_id = node_id;
                        let node = tab.find_node_by_id(node_id.unwrap());
                        assert!(node.is_some());
                    }
                    PageResponse::LoadEventFired(_timestamp) => {
                        let tab = tab.unwrap();
                        assert_result(&tab, self.found_node_id);
                        break Ok(().into())                        
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            let tab = self.debug_session.main_tab_mut().unwrap();
                            assert_result(&tab, self.found_node_id);
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
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,query_selector=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let mut selector = "#ddlogin-iframe #qrcode";
    let _my_page = QuerySelector {
        debug_session: Default::default(),
        url,
        selector,
        found_node_id: None,
    };

    selector = "#ddlogin-iframe";
    let my_page = QuerySelector {
        debug_session: Default::default(),
        url,
        selector,
        found_node_id: None,
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}