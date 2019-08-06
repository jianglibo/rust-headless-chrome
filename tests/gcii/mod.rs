use headless_chrome::browser_async::DebugSession;
use headless_chrome::protocol::{target};
use headless_chrome::browser::process;
use headless_chrome::browser_async::{Tab};

use std::default::Default;

mod login_page_displayed;
mod waiting_blank_page;
mod waiting_for_qrcode_scan;
mod home_page_full_displayed;

pub const HOME_URL: &str = "https://www.xuexi.cn/";
pub const LOGIN_URL: &str = "https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/";
pub const SHENBIAN_GANDONG_URL: &str = "https://www.xuexi.cn/2b50c77aa08a2621e69cdc7cb29e7d4b/b87d700beee2c44826a9202c75d18c85.html";
// pub const DETAIL_PAGE: &str = "detail_page";


use std::sync::Mutex;

lazy_static! {
    pub static ref PAGE_STATE: Mutex<PageState> = Mutex::new(PageState::WaitingBlankPage);
}


#[derive(Debug, Clone)]
pub enum PageState {
    WaitingBlankPage,
    LoginPageDisplayed,
    WaitingForQrcodeScan,
    QrCodeScaned,
    HomePageFullDisplayed,
}


impl Default for PageState {
    fn default() -> Self {
        PageState::WaitingBlankPage
    }
}


#[derive(Default)]
pub struct GetContentInIframe {
    pub debug_session: DebugSession,
    pub ddlogin_frame_stopped_loading: bool,
    pub state: PageState,
}



impl GetContentInIframe {
    pub fn new_visible() -> Self {
        Self {
            debug_session: DebugSession::new_visible(),
            ..Default::default()
        }
    }
    pub fn get_tab(&mut self, target_id: Option<&target::TargetId>) -> Option<&mut Tab> {
        self.debug_session.find_tab_by_id_mut(target_id).ok()
    }

}