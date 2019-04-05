use crate::protocol;
use crate::protocol::page;
use crate::protocol::dom;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};
use crate::protocol::page::methods::{Navigate};
use crate::protocol::target;
use websocket::futures::{Async, Future, Poll, Stream, IntoFuture};
use log::*;
use super::one_page::{OnePage};
use super::page_message::{PageMessage};
use super::interval_one_page::{IntervalOnePage};
use std::fs;
use std::time::{Duration, Instant};
use tokio::timer::{Interval};
use tokio_timer::*;
use tokio::prelude::stream::Select;


pub struct FindNode {
    chrome_page: IntervalOnePage,
    url: &'static str,
    selector: &'static str,
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
                    }
                    PageMessage::FrameNavigatedEvent(session_id, target_id, frame_navigated_event) => {
                        info!("frame event: {:?}", frame_navigated_event);
                        if let Some(frame_name) = frame_navigated_event.params.frame.name {
                            if frame_name == "ddlogin-iframe" {
                                self.chrome_page.one_page.find_node(self.selector);
                            }
                        }
                    }
                    PageMessage::DomQuerySelector(selector, node_id_op) => {
                        break Ok(node_id_op.into());
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

// impl Future for FindNode {
//     type Item = ();
//     type Error = failure::Error;

//     fn poll(&mut self) -> Poll<(), Self::Error> {
//         loop {
//             info!("my page loop ****************************");
//             if let Some(value) = try_ready!(self.chrome_page.poll()) {
//                 match value {
//                     PageMessage::EnablePageDone => {
//                         info!("page enabled.");
//                         self.chrome_page.one_page.navigate_to(self.url);
//                         // self.chrome_page.sleep(Duration::from_secs(10));
//                     },
//                     PageMessage::DocumentAvailable => {
                        
//                     }
//                     PageMessage::Screenshot(selector,_, _, jpeg_data) => {
//                         fs::write("screenshot.jpg", &jpeg_data.unwrap()).unwrap();
//                     },
//                     PageMessage::SecondsElapsed(seconds) => {
//                         info!("seconds elipsed: {}, page stuck in: {:?} ", seconds, self.chrome_page.one_page.state);
//                     }
//                     PageMessage::FrameNavigatedEvent(session_id, target_id, frame_navigated_event) => {
//                         info!("frame event: {:?}", frame_navigated_event);
//                         if let Some(frame_name) = frame_navigated_event.params.frame.name {
//                             info!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxx frame name {}", frame_name);
//                             if frame_name == "ddlogin-iframe" {
//                                 self.chrome_page.one_page.capture_screenshot_by_selector(self.selector, page::ScreenshotFormat::JPEG(Some(100)), true);
//                             }
                            
//                         }
//                     }
//                     _ => {
//                         info!("got unused page message {:?}", value);
//                     }
//                 }
//             } else {
//                 error!("got None, was stream ended?");
//             }
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use websocket::futures::{Future};
    use futures::{task};

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

    fn init_log() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=info,browser_async=debug");
        env_logger::init();
    }

    fn get_fixture_page() -> IntervalOnePage {
        let browser = ChromeBrowser::new();
        let page = OnePage::new(browser);
        IntervalOnePage::new(page)
    }

    #[test]
    fn t_get_node_id() {
        init_log();
        let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
        let mut selector = "#ddlogin-iframe #qrcode";
        let my_page = FindNode {
            chrome_page: get_fixture_page(),
            url,
            selector,
        };
        // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
        let result = run_one(my_page).unwrap();
        assert!(result.is_none());

        selector = "#ddlogin-iframe";
        let my_page = FindNode {
            chrome_page: get_fixture_page(),
            url,
            selector,
        };
        // tokio::run(my_page.map_err(|e| error!("{:?}", e)));
        let result = run_one(my_page).unwrap();
        assert!(result.is_some());

    }
}
