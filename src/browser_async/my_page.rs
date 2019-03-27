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
use std::time::{Duration, Instant};
use tokio::timer::{Interval};
use tokio_timer::*;

#[derive(Debug)]
struct FutureConsumeInterval {
    interval: Interval
}

impl Future for FutureConsumeInterval {
    type Item = ();
    type Error = tokio_timer::Error;
    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            if let Some(inst) = try_ready!(self.interval.poll()) {
                info!("{:?}", inst);
            } else {
                info!("not ready yet!");
            }
        }
    }
}

pub struct MyPage {
    chrome_page: OnePage,
    node_id: &'static str,
    interval: Interval,
    count: usize,
    last_not_ready: Instant,
    delay: Option<Delay>,
}


impl Future for MyPage {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        let mut loop_count = 0_usize;
        loop {
            loop_count += 1;

            info!("mypage loop ****************************");
            // If the poll result is not ready, then only the arrives of new messages can awake this loop.
            // when after a success poll we invoke a interval polling, the interval will awake this loop again.
            match self.chrome_page.poll() {
                Ok(Async::Ready(Some(value))) => {
                    loop_count = 0_usize;
                    match value {
                            PageMessage::DocumentAvailable => {
                                self.chrome_page.capture_screenshot_by_selector(self.node_id, page::ScreenshotFormat::JPEG(Some(100)), true);
                            }
                            PageMessage::Screenshot(selector,_, _, jpeg_data) => {
                                fs::write("screenshot.jpg", &jpeg_data.unwrap()).unwrap();
                            }
                            _ => {
                                info!("got unused page message {:?}", value);
                            }
                    }
                }
                Ok(Async::NotReady) => {
                    self.count += 1;
                    let elapsed = self.last_not_ready.elapsed();
                    info!("not ready! {}, {:?}, loop_count {}", self.count, elapsed, loop_count);
                    self.last_not_ready = Instant::now();
                    // if elapsed > Duration::from_secs(3) {
                    //     self.delay = Delay::new(Instant::now() + Duration::from_secs)
                    // } else {
                        return Ok(Async::NotReady);
                    // }
                    // If I return NotReady that means I will get polled again only if new message arrives.
                }
                Ok(Async::Ready(None)) => {
                    info!("reach stream end.");
                }
                Err(e) => {
                    error!("{:?}", e);
                }
            }
            // if let Some(value) = try_ready!(self.chrome_page.poll()) {
            //     match value {
            //             PageMessage::DocumentAvailable => {
            //                 self.chrome_page.capture_screenshot_by_selector(self.node_id, page::ScreenshotFormat::JPEG(Some(100)), true);
            //             }
            //             PageMessage::Screenshot(selector,_, _, jpeg_data) => {
            //                 fs::write("screenshot.jpg", &jpeg_data.unwrap()).unwrap();
            //             }
            //             _ => {
            //                 info!("got unused page message {:?}", value);
            //             }
            //         }
            //     info!("try interval.");
            //     if let Some(inst) = try_ready!(self.interval.poll()) {
            //         info!("{:?}", inst);
            //     }
            // } else {
            //     error!("got None, was stream ended?");
            // }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use websocket::futures::{Future};
    use futures::{task};

    // const ENTERY: &'static str = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    const ENTERY: &'static str = "https://en.wikipedia.org/wiki/WebKit";
    // https://en.wikipedia.org/wiki/WebKit
    // "#mw-content-text > div > table.infobox.vevent"


    struct Count {
        remaining: usize,
    }

    impl Future for Count {
        type Item = ();
        type Error = ();

        fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
            info!("got polled: remain {}", self.remaining);
            while self.remaining > 0 {
                self.remaining -= 1;

                // Yield every 10 iterations
                if self.remaining % 10 == 0 {
                    task::current().notify();
                    return Ok(Async::NotReady);
                }
            }
            Ok(Async::Ready(()))
        }
    }

    #[test]
    fn test_yielding() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();
        let ct = Count {remaining: 100};
        tokio::run(ct.map_err(|_| ()));
    }


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
            interval: Interval::new_interval(Duration::from_secs(10)),
            count: 0,
            last_not_ready: Instant::now(),
            delay: None,
        };

        tokio::run(my_page.map_err(|_| ()));

        // let ft = FutureConsumeInterval {
        //     interval: Interval::new_interval(Duration::from_secs(10)),
        // };
        // tokio::run(ft.map_err(|_| ()));
    }
}
