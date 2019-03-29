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
use super::one_page::{OnePage};
use super::page_message::{PageMessage};
use super::interval_one_page::{IntervalOnePage};
use std::fs;
use std::time::{Duration, Instant};
use tokio::timer::{Interval};
use tokio_timer::*;
use tokio::prelude::stream::Select;

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
    chrome_page: IntervalOnePage,
}


impl Future for MyPage {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            info!("my page loop ****************************");
            if let Some(value) = try_ready!(self.chrome_page.poll()) {
                match value {
                    PageMessage::NavigatingToTarget => {
                        self.chrome_page.sleep(Duration::from_secs(10));
                    },
                    PageMessage::DocumentAvailable => {
                        self.chrome_page.one_page.capture_screenshot_by_selector("#ddlogin", page::ScreenshotFormat::JPEG(Some(100)), true);
                    }
                    PageMessage::Screenshot(selector,_, _, jpeg_data) => {
                        fs::write("screenshot.jpg", &jpeg_data.unwrap()).unwrap();
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

        let itv = Interval::new_interval(Duration::from_secs(10));
        let itv_1 = Interval::new_interval(Duration::from_secs(20));

        // let select2 = Select::new(itv, itv_1);
        tokio::run(ct.map_err(|_| ()));
    }


    #[test]
    fn t_by_enum() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=info,browser_async=debug");
        // ::std::env::set_var("RUST_LOG", "trace");
        env_logger::init();
        // let entry_url = "https://en.wikipedia.org/wiki/WebKit";
        let entry_url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

        let browser = ChromeBrowser::new();
        let page = OnePage::new(browser, entry_url);

        let interval_page = IntervalOnePage::new(Duration::from_secs(3), page);

        let my_page = MyPage {
            chrome_page: interval_page,
        };

        tokio::run(my_page.map_err(|_| ()));

    }
}
