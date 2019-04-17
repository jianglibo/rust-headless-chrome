extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{dom};

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageResponse};
use headless_chrome::browser_async::debug_session::{DebugSession};
use std::default::Default;
use tokio;


struct FindNode {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
}

impl Future for FindNode {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, value)) = try_ready!(self.debug_session.poll()) {
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
                        if seconds > 39 {
                            error!("time out {}", seconds);
                            panic!("time out 40 seconds.");
                        }
                    }
                    // PageResponse::PageEvent(PageEventName::loadEventFired) => {
                    PageResponse::PageEvent(_) => {
                        // if let Some(_) = self.debug_session.chrome_debug_session.is_frame_navigated("ddlogin-iframe") {
                        //     self.debug_session.chrome_debug_session.dom_describe_node_by_selector(self.selector, Some(5));
                        // }
                    }
                    PageResponse::DescribeNode(_task_id, node, task) => {
                        info!("got node:: {:?}", node);
                        info!("task done: {:?}", task);
                        break Ok(().into());
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
    runtime.block_on(my_page.into_future()).unwrap();
}