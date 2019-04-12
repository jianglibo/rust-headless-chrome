extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::protocol::{self, page, dom};
use headless_chrome::browser_async::chrome_browser::{ChromeBrowser};
use headless_chrome::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};

use websocket::futures::{Async, Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::one_page::{OnePage};
use headless_chrome::browser_async::page_message::{PageMessage, PageEventName, ChangingFrameTree, ChangingFrame};
use headless_chrome::browser_async::interval_one_page::{IntervalOnePage};
use std::fs;
use std::time::{Duration, Instant};
use tokio::timer::{Interval};
use tokio_timer::*;
use tokio::prelude::stream::Select;
use tokio;
use futures::{task};
use std::collections::HashMap;
use std::collections::HashSet;
use std::borrow::Borrow;


struct FindNode {
    chrome_page: IntervalOnePage,
    url: &'static str,
    selector: &'static str,
    root_node_id: Option<dom::NodeId>,
}

impl Future for FindNode {
    type Item = Option<dom::NodeId>;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            info!("my page loop ****************************");
            if let Some(value) = try_ready!(self.chrome_page.poll()) {
                match value {
                    PageMessage::EnablePageDone => {
                        info!("page enabled.");
                        self.chrome_page.one_page.navigate_to(self.url);
                    },
                    PageMessage::SecondsElapsed(seconds) => {
                        info!("seconds elipsed: {}, page stuck in: {:?} ", seconds, self.chrome_page.one_page.state);
                        if seconds > 39 {
                            error!("time out {}", seconds);
                            panic!("time out 40 seconds.");
                        }
                    }
                    // PageMessage::PageEvent(PageEventName::loadEventFired) => {
                    PageMessage::PageEvent(_) => {
                        if let Some(_) = self.chrome_page.one_page.is_frame_navigated("ddlogin-iframe") {
                            self.chrome_page.one_page.dom_query_selector_by_selector(self.selector, Some(5));
                        }
                    }
                    PageMessage::NodeIdComing(node_id, task) => {
                        info!("task done: {:?}", task);
                        break Ok(Some(node_id).into());
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

fn run_one<F>(f: F) -> Result<F::Item, F::Error>
where
    F: IntoFuture,
    F::Future: Send + 'static,
    F::Item: Send + 'static,
    F::Error: Send + 'static,
{
        let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
        runtime.block_on(f.into_future())
}

fn get_fixture_page() -> IntervalOnePage {
    let browser = ChromeBrowser::new();
    let page = OnePage::new(browser);
    IntervalOnePage::new(page)
}


#[test]
fn t_dom_query_selector() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,browser_async=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let mut selector = "#ddlogin-iframe #qrcode";
    let my_page = FindNode {
        chrome_page: get_fixture_page(),
        url,
        selector,
        root_node_id: None,
    };

    selector = "#ddlogin-iframe";
    let my_page = FindNode {
        chrome_page: get_fixture_page(),
        url,
        selector,
        root_node_id: None,
    };
    let result = run_one(my_page).unwrap();
    assert!(result.is_some());
}