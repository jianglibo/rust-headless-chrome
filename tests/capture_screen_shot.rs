extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use websocket::futures::{Future, Poll, Stream, IntoFuture};
use log::*;
use headless_chrome::browser_async::page_message::{PageResponse, response_object};
use headless_chrome::browser_async::debug_session::{DebugSession};
use headless_chrome::browser_async::page_message::{ChangingFrame};
use headless_chrome::protocol::{page};
use std::default::Default;
use std::fs;
use std::path::Path;
use tokio;


struct CaptureScreenShotTest {
    debug_session: DebugSession,
    url: &'static str,
    selector: &'static str,
    ro: Option<response_object::CaptureScreenshot>,
}

impl Future for CaptureScreenShotTest {
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
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    },
                    PageResponse::PageEnable => {
                        info!("page enabled.");
                        tab.unwrap().navigate_to(self.url);
                    },
                    PageResponse::SecondsElapsed(seconds) => {
                        info!("seconds elapsed: {} ", seconds);
                        if seconds == 19 {
                                if let Some(tab) = self.debug_session.main_tab_mut() {
                                    tab.capture_screenshot_by_selector(self.selector, page::ScreenshotFormat::PNG, true, Some(100));
                                }
                        }
                        if seconds > 25 {
                            assert!(self.ro.is_some());
                            
                            let file_name = "target/abc.png";
                            let path = Path::new(file_name);
                            if path.exists() && path.is_file() {
                                fs::remove_file(file_name).unwrap();
                            }
                            self.ro.as_ref().unwrap().write_to(file_name).unwrap();
                            assert!(path.exists());
                            break Ok(().into());
                        }
                    }
                    PageResponse::FrameNavigated(changing_frame) => {
                        trace!("got frame: {:?}", changing_frame);
                        if let ChangingFrame::Navigated(frame) = changing_frame {
                            if frame.name == Some("ddlogin-iframe".into()) {
                                if let Some(tab) = self.debug_session.main_tab_mut() {
                                    tab.capture_screenshot_by_selector(self.selector, page::ScreenshotFormat::PNG, true, Some(100));
                                }
                            }
                        }
                    }
                    PageResponse::Screenshot(capture_screen_shot) => {
                        info!("got screen shot: {:?}", capture_screen_shot.base64);
                        assert!(capture_screen_shot.base64.is_some());
                        assert_eq!(capture_screen_shot.selector, Some(self.selector));
                        self.ro  = Some(capture_screen_shot);
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
fn t_take_screen_shot() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,capture_screen_shot=info");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let selector = "#ddlogin-iframe";
    let my_page = CaptureScreenShotTest {
        debug_session: Default::default(),
        url,
        selector,
        ro: None,
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}