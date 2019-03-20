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
                                self.chrome_page.find_node("#ddlogin");
                            }
                        }
                        MyPageState::WaitingNode => {
                            info!("waiting node.");
                            if let PageMessage::FindNode(maybe_selector, nd) = value {
                                if Some("#ddlogin".to_string()) == maybe_selector {
                                    info!("got node {:?}", nd);
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
    use crate::protocol::page::methods::Navigate;
    use crate::protocol::page::ScreenshotFormat;
    use futures::stream::Stream;
    use tokio;
    use tokio::runtime::Runtime;
    use websocket::futures::{Async, Future, Poll, Sink};
    use websocket::r#async::client::{Client, ClientNew};
    use websocket::r#async::TcpStream;
    use websocket::ClientBuilder;
    use websocket::Message;

    use crate::browser::process::{LaunchOptions, LaunchOptionsBuilder, Process};

    const ENTERY: &'static str = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";

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
