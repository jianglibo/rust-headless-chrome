extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{dom, page};

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::ChangingFrame;
use headless_chrome::browser_async::page_message::PageResponse;
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

struct IframeContent {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    node_id: Option<dom::NodeId>,
    node: Option<dom::Node>,
    ddlogin_frame: Option<page::Frame>,
}

impl IframeContent {
    fn assert_result(&self) {
        let tab = self.debug_session.main_tab().unwrap();
        // assert!(tab.temporary_node_holder.len() >= 7);
        // tab.temporary_node_holder.values().for_each(|n|{
        //     info!("all nodes: {:?}", n);
        // });
        // let message_text = serde_json::to_string(&tab.temporary_node_holder).unwrap();
        // info!("json: {:?}", message_text);
    }
}

impl Future for IframeContent {
    type Item = ();
    type Error = failure::Error;

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
                        let tab = tab.unwrap();
                        tab.navigate_to(self.url, None);
                    }
                    PageResponse::FrameNavigated(frame_id) => {
                        let tab = tab.unwrap();
                        let frame = tab.find_frame_by_id(&frame_id).unwrap();
                        info!("got frame: {:?}", frame_id);
                        self.ddlogin_frame = Some(frame.clone());
                        if frame.name == Some("ddlogin-iframe".into()) {
                            if let Some(tab) = self.debug_session.main_tab_mut() {
                                tab.describe_node_by_selector(self.selector, Some(10), Some(100));
                            }
                        }
                    }
                    PageResponse::DescribeNode(_selector, node_id) => {
                        self.node = tab.unwrap().find_node_by_id(node_id.unwrap()).cloned();
                        assert_eq!(
                            Some(&self.ddlogin_frame.as_ref().unwrap().id),
                            self.node.as_ref().unwrap().frame_id.as_ref()
                        );
                        info!("node content: {:?}", self.node);
                        info!(
                            "content document: {:?}",
                            self.node.as_ref().unwrap().content_document
                        );
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

// [2019-04-15T08:12:35Z ERROR headless_chrome::browser_async::chrome_debug_session] unprocessed ReceivedMessageFromTargetEvent { params: ReceivedMessageFromTargetParams { session_id: "B1B0F5851D30241BC39BE00415D4F43A", target_id: "C9CB934D0B4C63978622ED5F01D6B829", message: "{\"method\":\"DOM.setChildNodes\",\"params\":{\"parentId\":1,\"nodes\":[{\"nodeId\":2,\"parentId\":1,\"backendNodeId\":5,\"nodeType\":10,\"nodeName\":\"html\",\"localName\":\"\",\"nodeValue\":\"\",\"publicId\":\"\",\"systemId\":\"\"},{\"nodeId\":3,\"parentId\":1,\"backendNodeId\":6,\"nodeType\":1,\"nodeName\":\"HTML\",\"localName\":\"html\",\"nodeValue\":\"\",\"childNodeCount\":2,\"attributes\":[],\"frameId\":\"C9CB934D0B4C63978622ED5F01D6B829\"}]}}" } }

#[test]
fn t_iframe_content() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,iframe_content=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

    let selector = "#ddlogin-iframe";
    let my_page = IframeContent {
        debug_session: Default::default(),
        url,
        selector,
        node_id: None,
        node: None,
        ddlogin_frame: None,
    };
    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    if let Err(err) = runtime.block_on(my_page.into_future()) {
        error!("err: {:?}", err);
    }
}
