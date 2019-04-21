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
use std::default::Default;
use tokio;


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
            if let Some((tab_id, task_id, value)) = try_ready!(self.debug_session.poll()) {
                let tab = if let Some(tid) = &tab_id {
                    self.debug_session.get_tab_by_id_mut(tid)
                } else {
                    None
                };
                match value {
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        tab.unwrap().navigate_to(self.url);
                    },
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            self.assert_result();
                            break Ok(().into())                        
                        }
                    }
                    // PageResponse::LoadEventFired(_timestamp) => {
                    //     // let tab = tab.unwrap();
                    //     self.assert_result();
                    //     break Ok(().into())
                    // }
                    PageResponse::FrameNavigated(changing_frame) => {
                        info!("got frame: {:?}", changing_frame);
                        if let ChangingFrame::Navigated(frame) = changing_frame {
                            if frame.name == Some("ddlogin-iframe".into()) {
                                if let Some(tab) = self.debug_session.main_tab_mut() {
                                    // tab.describe_node_by_selector(self.selector, Some(2), Some(100));
                                    tab.describe_node_by_selector("#notexistid", Some(2), Some(101));
                                }
                            }
                        }
                    }
                    PageResponse::DescribeNode(selector, node_id) => {
                        assert!(task_id == Some(100) || task_id == Some(101));
                        assert!(node_id.is_some());
                        assert_eq!(selector, Some(self.selector));
                        self.node_id  = node_id;
                        self.node = tab.unwrap().find_node_by_id(node_id.unwrap()).cloned();
                        info!("content document: {:?}", self.node.as_ref().unwrap().content_document);
                        // tab.unwrap().
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

// fn run_one<F>(f: F) -> Result<F::Item, F::Error>
// where
//     F: IntoFuture,
//     F::Future: Send + 'static,
//     F::Item: Send + 'static,
//     F::Error: Send + 'static,
// {
//         let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
//         runtime.block_on(f.into_future())
// }

#[test]
fn t_dom_describe_node() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,describe_node=info");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let mut selector = "#ddlogin-iframe #qrcode";
    let _my_page = DescribeNode {
        debug_session: Default::default(),
        url,
        selector,
        node_id: None,
        node: None,
    };

    selector = "#ddlogin-iframe";
    let my_page = DescribeNode {
        debug_session: Default::default(),
        url,
        selector,
        node_id: None,
        node: None,
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}