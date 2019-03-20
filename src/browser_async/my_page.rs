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

#[derive(Debug)]
enum MyPageState {
    Start,
    WaitingNode,
    Consuming,
}

pub struct MyPage {
    chrome_page: OnePage,
    state: MyPageState,
}


impl Future for MyPage {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<(), Self::Error> {
        loop {
            if let Some(value) = try_ready!(self.chrome_page.poll()) {
                    match &mut self.state {
                        MyPageState::Start => {
                            if let PageMessage::DocumentAvailable = value {
                                self.state = MyPageState::WaitingNode;
                                // self.chrome_page.find_node("#ddlogin");
                                self.chrome_page.find_node("#mw-content-text > div > table.infobox.vevent");
                            }
                        }
                        MyPageState::WaitingNode => {
                            info!("waiting node.");
                            if let PageMessage::FindNode(maybe_selector, nd) = value {
                                // if Some("#ddlogin".to_string()) == maybe_selector {
                                if Some("#mw-content-text > div > table.infobox.vevent".to_string()) == maybe_selector {
                                    info!("got node {:?}", nd);
                                    self.state = MyPageState::Consuming;
                                }
                            }
                        }
                        _ => {
                            trace!("receive message: {:?}", value);
                        },
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
    // #mw-content-text > div > table.infobox.vevent



    #[test]
    fn t_by_enum() {
        ::std::env::set_var("RUST_LOG", "headless_chrome=trace,browser_async=debug");
        env_logger::init();

        let browser = ChromeBrowser::new();
        let page = OnePage::new(browser, ENTERY);
        let my_page = MyPage {
            chrome_page: page,
            state: MyPageState::Start
        };

        tokio::run(my_page.map_err(|_| ()));
    }
}
