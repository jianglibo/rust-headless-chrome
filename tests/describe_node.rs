extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{dom};

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageMessage};
use headless_chrome::browser_async::debug_session::{DebugSession};
use std::default::Default;
use tokio;


struct FindNode {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
}

impl Future for FindNode {
    type Item = Option<dom::Node>;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            info!("my page loop ****************************");
            if let Some(value) = try_ready!(self.debug_session.poll()) {
                match value {
                    PageMessage::EnablePageDone(target_id) => {
                        info!("page enabled.");
                        let tab = self.debug_session.get_tab_by_id(target_id);
                        tab.unwrap().navigate_to(self.url);
                    },
                    PageMessage::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {}, page stuck in: {:?} ", seconds, self.debug_session.session_state());
                        if seconds > 39 {
                            error!("time out {}", seconds);
                            panic!("time out 40 seconds.");
                        }
                    }
                    // PageMessage::PageEvent(PageEventName::loadEventFired) => {
                    PageMessage::PageEvent(_) => {
                        // if let Some(_) = self.debug_session.chrome_debug_session.is_frame_navigated("ddlogin-iframe") {
                        //     self.debug_session.chrome_debug_session.dom_describe_node_by_selector(self.selector, Some(5));
                        // }
                    }
                    PageMessage::NodeComing(node, task) => {
                        info!("got node:: {:?}", node);
                        info!("task done: {:?}", task);
                        break Ok(Some(node).into());
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
    let _my_page = FindNode {
        debug_session: Default::default(),
        url,
        selector,
    };

    selector = "#ddlogin-iframe";
    let my_page = FindNode {
        debug_session: Default::default(),
        url,
        selector,
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    let result = runtime.block_on(my_page.into_future()).unwrap();
    assert!(result.is_some());
}