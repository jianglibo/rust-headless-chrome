extern crate log;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

use headless_chrome::browser_async::debug_session::DebugSession;
use headless_chrome::browser_async::page_message::{response_object, PageResponse};
use headless_chrome::protocol::page;
use log::*;
use std::default::Default;
use std::fs;
use std::path::Path;
use tokio;
use websocket::futures::{Future, IntoFuture, Poll, Stream};

#[derive(Default)]
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
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let tab = self.debug_session.get_tab_by_resp_mut(&page_response_wrapper).ok();
                match page_response_wrapper.page_response {
                    PageResponse::ChromeConnected => {
                        self.debug_session.set_discover_targets(true);
                    }
                    PageResponse::PageEnabled => {
                        info!("page enabled.");
                        tab.unwrap().navigate_to(self.url, None);
                    }
                    PageResponse::SecondsElapsed(seconds) => {
                        trace!("seconds elapsed: {} ", seconds);
                        if seconds == 19 {
                            if let Some(tab) = self.debug_session.first_page_mut() {
                                tab.capture_screenshot_by_selector(
                                    self.selector,
                                    page::ScreenshotFormat::PNG,
                                    true,
                                    Some(100),
                                );
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
                    PageResponse::FrameNavigated(frame_id) => {
                        let tab = tab.unwrap();
                        let frame = tab.find_frame_by_id(&frame_id).unwrap();
                        info!("got frame: {:?}", frame_id);
                        if frame.name == Some("ddlogin-iframe".into()) {
                            if let Some(tab) = self.debug_session.first_page_mut() {
                                tab.capture_screenshot_by_selector(
                                    self.selector,
                                    page::ScreenshotFormat::PNG,
                                    true,
                                    Some(100),
                                );
                            }
                        }
                    }
                    PageResponse::CaptureScreenshotDone(capture_screen_shot) => {
                        info!("got screen shot: {:?}", capture_screen_shot.base64);
                        assert!(capture_screen_shot.base64.is_some());
                        assert_eq!(capture_screen_shot.selector, Some(self.selector));
                        self.ro = Some(capture_screen_shot);
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
fn t_take_screen_shot() {
    ::std::env::set_var("RUST_LOG", "headless_chrome=info,capture_screen_shot=info");
    env_logger::init();
    let url = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
    let selector = "#ddlogin-iframe";
    let my_page = CaptureScreenShotTest {
        url,
        selector,
        ..Default::default()
    };

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(my_page.into_future()).unwrap();
}
