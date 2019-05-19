extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

// use headless_chrome::protocol::{dom};
use headless_chrome::browser::tab::element::BoxModel;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::PageResponse;
use log::*;
use std::default::Default;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
struct GetBoxModelTest {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    box_model: Option<Box<BoxModel>>,
}

impl Future for GetBoxModelTest {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {

                let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
                match page_response_wrapper.page_response {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        tab.unwrap().navigate_to(self.url, None);
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds > 19 {
                            assert!(self.box_model.is_some());
                        }
                    }
                    PageResponse::FrameNavigated(frame_id) => {
                        let tab = tab.unwrap();
                        let frame = tab.find_frame_by_id(&frame_id).unwrap();
                        info!("got frame: {:?}", frame_id);
                        if frame.name == Some("ddlogin-iframe".into()) {
                            if let Some(tab) = self.debug_session.first_page_mut() {
                                tab.get_box_model_by_selector(self.selector, Some(100));
                            }
                        }
                    }
                    PageResponse::GetBoxModelDone(selector, box_model) => {
                        trace!("got box model: {:?}", box_model);
                        assert!(box_model.is_some());
                        assert_eq!(selector, Some(self.selector.into()));
                        self.box_model = box_model;
                        break Ok(().into());
                    }
                    _ => {
                        trace!("got unused page message {:?}", page_response_wrapper);
                    }
                }
            } else {
                error!("got None, was stream ended?");
            }
        }
    }
}

#[test]
fn t_get_model_box() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=trace,get_box_model=trace");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let selector = "#ddlogin-iframe";
    let my_page = GetBoxModelTest {
        url,
        selector,
        ..Default::default()
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
