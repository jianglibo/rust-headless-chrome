use crate::protocol;
use crate::protocol::page;
use crate::protocol::dom;
use crate::browser_async::chrome_browser::{ChromeBrowser};
use crate::browser_async::dev_tools_method_util::{
    MethodUtil,MethodDestination, MethodBeforSendResult, ChromePageError,
};
use crate::protocol::page::methods::{Navigate};
use crate::protocol::target;
use websocket::futures::{Async, Future, Poll, Stream};
use log::*;
use crate::browser_async::one_page::{OnePage, PageMessage};
use std::fs;

// #[derive(Debug)]
// enum MyPageState {
//     Start,
//     // WaitingNode,
//     // WaitElement,
//     // WaitModelBox,
//     // WaitingScreenshot,
//     Consuming,
// }

pub struct MyPage {
    chrome_page: OnePage,
    // state: MyPageState,
    node_id: &'static str,
}


impl Future for MyPage {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            info!("mypage loop ****************************");
            if let Some(value) = try_ready!(self.chrome_page.poll()) {
                match value {
                        PageMessage::DocumentAvailable => {
                            self.chrome_page.find_node(self.node_id);
                        }
                        PageMessage::FindNode(maybe_selector, nd) => {
                            if Some(self.node_id.to_string()) == maybe_selector {
                                info!("got node {:?}", nd);
                            }
                        }
                        PageMessage::FindElement(selector, element) => {
                            if self.node_id == &selector {
                                info!("got element {:?}", element);
                            }
                        }
                        PageMessage::GetBoxModel(backend_node_id, box_model) => {
                            info!("box model: {:?}", box_model);
                            self.chrome_page.capture_screenshot(page::ScreenshotFormat::JPEG(Some(100)),
                                Some(box_model.content_viewport()),
                                true
                            );
                        }
                        PageMessage::Screenshot(jpeg_data) => {
                            fs::write("screenshot.jpg", &jpeg_data).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use websocket::futures::{Future};

    // const ENTERY: &'static str = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    const ENTERY: &'static str = "https://en.wikipedia.org/wiki/WebKit";
    // https://en.wikipedia.org/wiki/WebKit
    // "#mw-content-text > div > table.infobox.vevent"

    #[test]
    fn t_by_enum() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();
        // let entry_url = "https://en.wikipedia.org/wiki/WebKit";
        let entry_url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

        let browser = ChromeBrowser::new();
        let page = OnePage::new(browser, entry_url);
        let my_page = MyPage {
            chrome_page: page,
            // state: MyPageState::Start,
            node_id: "#ddlogin",
        };

        tokio::run(my_page.map_err(|_| ()));
    }
}