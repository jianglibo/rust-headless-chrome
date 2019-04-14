extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{dom};

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::task_describe::{TaskDescribe};
use headless_chrome::browser_async::debug_session::{DebugSession};
use tokio;
use std::default::Default;


struct FindNode {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
}

impl Future for FindNode {
    type Item = Option<dom::NodeId>;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            info!("my page loop ****************************");
            if let Some(value) = try_ready!(self.debug_session.poll()) {
                match value {
                    TaskDescribe::PageEnable(task_id, target_id) => {
                        info!("page enabled.");
                        // self.debug_session.chrome_debug_session.navigate_to(self.url);
                    },
                    TaskDescribe::SecondsElapsed(seconds) => {
                        info!("seconds elipsed: {} ", seconds);
                        if seconds > 39 {
                            error!("time out {}", seconds);
                            panic!("time out 40 seconds.");
                        }
                    }
                    // TaskDescribe::PageEvent(PageEventName::loadEventFired) => {
                    TaskDescribe::PageEvent(_) => {
                        // if let Some(_) = self.debug_session.chrome_debug_session.is_frame_navigated("ddlogin-iframe") {
                        //     self.debug_session.chrome_debug_session.dom_query_selector_by_selector(self.selector, Some(5));
                        // }
                    }
                    // TaskDescribe::NodeIdComing(node_id, task) => {
                    //     info!("task done: {:?}", task);
                    //     break Ok(Some(node_id).into());
                    // }
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


#[test]
fn t_dom_query_selector() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,query_selector=trace");
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