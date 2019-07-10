use headless_chrome::browser_async::DebugSession;

use std::default::Default;

mod home_page_displayed;
mod waiting_blank_page;
// mod waiting_for_qrcode_scan;

pub const HOME_URL: &str = "https://www.xuexi.cn/";
pub const LOGIN_URL: &str = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
pub const SHENBIAN_GANDONG_URL: &str = "https://www.xuexi.cn/2b50c77aa08a2621e69cdc7cb29e7d4b/b87d700beee2c44826a9202c75d18c85.html";

#[derive(Debug)]
pub enum PageState {
    WaitingBlankPage,
    HomePageDisplayed
}


impl Default for PageState {
    fn default() -> Self {
        PageState::WaitingBlankPage
    }
}


#[derive(Default)]
pub struct CaptureScreenShotTest {
    pub debug_session: DebugSession,
    pub ddlogin_frame_stopped_loading: bool,
    pub state: PageState,
}
