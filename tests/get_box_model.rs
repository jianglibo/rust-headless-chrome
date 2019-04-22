extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

// use headless_chrome::protocol::{dom};
use headless_chrome::browser::tab::element::{BoxModel};

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageResponse};
use headless_chrome::browser_async::debug_session::{DebugSession};
use headless_chrome::browser_async::page_message::{ChangingFrame};
use std::default::Default;
use tokio;


struct GetBoxModelTest {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    box_model: Option<BoxModel>,
}

impl Future for GetBoxModelTest {
    type Item = ();
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some((tab_id, _task_id, value)) = try_ready!(self.debug_session.poll()) {
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
                        if seconds > 19 {
                            assert!(self.box_model.is_some());
                        }
                    }
                    PageResponse::FrameNavigated(changing_frame) => {
                        info!("got frame: {:?}", changing_frame);
                        if let ChangingFrame::Navigated(frame) = changing_frame {
                            if frame.name == Some("ddlogin-iframe".into()) {
                                if let Some(tab) = self.debug_session.main_tab_mut() {
                                    tab.get_box_model_by_selector(self.selector, Some(100));
                                }
                            }
                        }
                    }
                    PageResponse::GetBoxModel(selector, box_model) => {
                        info!("got box model: {:?}", box_model);
                        assert!(box_model.is_some());
                        assert_eq!(selector, Some(self.selector));
                        self.box_model  = box_model;
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

#[test]
fn t_get_model_box() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,get_box_model=info");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let mut selector = "#ddlogin-iframe #qrcode";
    let _my_page = GetBoxModelTest {
        debug_session: Default::default(),
        url,
        selector,
        box_model: None,
    };

    selector = "#ddlogin-iframe";
    let my_page = GetBoxModelTest {
        debug_session: Default::default(),
        url,
        selector,
        box_model: None,
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}