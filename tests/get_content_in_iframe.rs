#![warn(clippy::all)]


extern crate chrono;
extern crate fern;
extern crate log;
use rand::prelude::*;

#[macro_use]
extern crate futures;
extern crate tokio_timer;

mod gcii;
mod tutil;

use headless_chrome::browser_async::{EventName, Tab};

use headless_chrome::browser_async::page_message::PageResponse;
use log::*;
use std::default::Default;

use websocket::futures::{Future, IntoFuture, Poll, Stream};

use gcii::{GetContentInIframe, PageState, HOME_URL, SHENBIAN_GANDONG_URL, DETAIL_PAGE};

impl GetContentInIframe {
    fn assert_result(&self) {
        assert!(self.ddlogin_frame_stopped_loading);
    }
}

impl Future for GetContentInIframe {
    type Item = ();
    type Error = failure::Error;

    #[allow(clippy::cognitive_complexity)]
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            if let Some(page_response_wrapper) = try_ready!(self.debug_session.poll()) {
                let maybe_target_id = page_response_wrapper.target_id.clone();
                if let PageResponse::SecondsElapsed(seconds) = page_response_wrapper.page_response {

                    if seconds % 30 == 0 {
                        self.debug_session.close_tab_by_window_close_old_than(390);
                        if self.debug_session.tab_count() < 2 {
                            info!("************** tab_count: {:?}", self.debug_session.tab_count());
                            self.debug_session.run_manually_tasks();
                        } else {
                            info!("************** tab_count: {:?}", self.debug_session.tab_count());
                        }
                    }
                    if seconds % 30 == 0 {
                        info!("{:?}", self.state);
                        for t in self.debug_session.tabs.iter_mut() {
                            // t.move_mouse_random_after_secs(1);
                            let rs = t.network_statistics.list_request_urls();
                            let pclogs = t.network_statistics.list_request_urls_end_with("/pclog");
                            // if !rs.is_empty() {
                                info!("main frame: {:?}, frame_count: {:?}", t.main_frame(), t.changing_frames.len());
                                info!("{}, context_id: {:?}, target_id: {:?}, session_id: {:?}", t.get_url(), t.target_info.browser_context_id, t.target_info.target_id, t.session_id);
                                info!("requested urls {:?}: {:?}",rs.len(), pclogs);
                                info!("box_model: {:?}", t.box_model.as_ref().and_then(|_|Some("exists.")));
                                info!("task queue {:?}, {:?}: {:?}", t.task_queue.vec_len(), t.task_queue.item_len(), t.task_queue.to_task_names());
                            // }
                        }
                        // self.debug_session.browser_contexts().deduplicate();
                        // self.debug_session.activates_next_in_interval(10);
                        // self.debug_session.activate_last_opened_tab();
                        let  popup_count = self.debug_session.loaded_by_this_tab_name_count(HOME_URL);
                        if popup_count > 0 { //when popup_count > 0, home tab should exist.
                            let run_task_queue_manually = popup_count < 1;
                            let tab = self
                                .debug_session
                                .find_tab_by_name_mut(HOME_URL)
                                .expect("home page should exists.");
                            if run_task_queue_manually {
                                tab.run_task_queue_manually();
                            }
                        }
                        if popup_count > 0 {
                            info!("popup_count is {}", popup_count);
                        }
                    }

                    if let Ok(tab) = self.debug_session.find_tab_by_name_mut(DETAIL_PAGE) {
                        if tab.bring_to_front() {
                            info!("activating {:?}", tab);
                        }

                        // if seconds % 100 == 0 {
                        //     info!("detail page: {:?}", tab);
                        // }
                        // if seconds > 100 && seconds % 20 == 0 && tab.session_id.is_some() {
                        //     // let mut rng = rand::thread_rng();
                        //     // let (x, y): (u64, u64) = (rng.gen_range(0, 300), rng.gen_range(0, 300));
                        //     let t1 = tab.mouse_move_to_xy_task(10.0, 10.0);
                        //     // let (x1, y1): (u64, u64) = (rng.gen_range(0, 300), rng.gen_range(0, 300));
                        //     let t2 = tab.mouse_move_to_xy_task(20.0, 20.0);
                        //     tab.execute_tasks(vec![t1, t2]);
                        // }
                    }
                    // self.debug_session.activates_next_in_interval(3);
                    // if let Some(tab) = self
                    //     .debug_session
                    //     .loaded_by_this_tab_name_mut(HOME_URL)
                    //     .get_mut(0)
                    // {
                    //     // info!("bring to front. {:?}", tab);
                    //     if tab.bring_to_front() {
                    //         info!("bring to front................had sent.");
                    //     }
                    // }
                    // if let Some(popup_count) = self.debug_session.loaded_by_this_tab_name_count(SHENBIAN_GANDONG_URL) {
                    //     let run_task_queue_manually = popup_count < 2;
                    //     let tab = self.debug_session.find_tab_by_name_mut(SHENBIAN_GANDONG_URL).expect("shenbian gandong page should exists.");
                    //     if run_task_queue_manually {
                    //         info!("run_task_queue_manually.");
                    //         tab.run_task_queue_manually();
                    //     }
                    // }

                    // self.debug_session
                    //     .find_tabs_old_than(600)
                    //     .into_iter()
                    //     .filter(|tb| !tb.is_at_url(HOME_URL))
                    //     .for_each(Tab::page_close);
                    if seconds > 12_0000 {
                        self.debug_session
                            .tabs
                            .iter()
                            .for_each(|tb| info!("{:?}", tb));
                        assert_eq!(self.debug_session.tabs.len(), 19);
                        let m = self
                            .debug_session
                            .tabs
                            .iter()
                            .filter(|tb| tb.is_at_url(HOME_URL))
                            .count();
                        assert_eq!(m, 1);

                        let tab = self
                            .debug_session
                            .first_page_mut()
                            .expect("tab should exists.");
                        assert!(
                            tab.event_statistics
                                .happened_count(EventName::ExecutionContextCreated)
                                > 7
                        );
                        self.assert_result();
                        break Ok(().into());
                    }
                } else {
                    match self.state {
                        PageState::WaitingBlankPage => {
                            self.waiting_blank_page(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::LoginPageDisplayed => {
                            self.login_page_displayed(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::WaitingForQrcodeScan => {
                            self.waiting_for_qrcode_scan(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                        PageState::HomePageFullDisplayed => {
                            self.home_page_full_displayed(
                                maybe_target_id.as_ref(),
                                page_response_wrapper.page_response,
                            );
                        }
                    }
                }
            } else {
                warn!("got None, was stream ended?");
            }
        }
    }
}

/**
 * {"logs":"{\"17171904947631584673_lwjqu85vhfb_1560952133562\":{\"url\":\"https://www.xuexi.cn/lgpage/detail/index.html?id=17171904947631584673\",\"pt\":\"Graphic-article\",\"tags\":\"联合国大会\",\"item_id\":\"17171904947631584673\",\"refer\":\"https://www.xuexi.cn/\",\"read_id\":\"17171904947631584673_lwjqu85vhfb_1560952133562\",\"page_uuid\":\"17171904947631584673\",\"duration\":30,\"et\":2,\"track_id\":\"f82c3c70-e3bd-4879-aaa4-c1b02b930920\",\"tm\":1560952235276}}","uid":"8b07a940-3c0c-11e9-ae50-3981e89811ea","token":"15aff8fa71a24fd095011592ea5f87ed","sid":"c6a61d6fd3aa25275febb1584d687448","uaToken":"115#1kHgE11O1TavOfqVTCCY1CsoE51GLJA11g2mOh2/jCCcAo21/drCBftunjs1y5fyUqczvP5GX9PKi/RChaFGaLumakUNsGBU9jfyeKT8ukZQi/yJHZz4OWNcaLBXyrrQASRretT4O6Nj7RMWhEz8/DNDaLpXyrrQOSYseKT8ukNQi/RChaU8OSP2Z6yiHFtRsCuKFtQc1Sit5JFR7afewgyUKv56jD5jwW9YdjkUMz2ifIaScSlGYmr7P6gj2qgzxdy8Yymx2dHypBbJCX6t2jkVcxkbLc5xd/kVDlW1cwCVLjCxCo6P3E5xwQLES3Q/ZU4c02IOmor2sXEQyHc0zQ9F5k6pihHXKiUyuHJabn1rcActZF97UJUHmLJYtkfRyB3YynN5C7vYn5hzh21l9lddFV9vtPM52vuG69Kj+Wg2KMgVymR8DZEQcy6TuWM/JveFVyQHD9Jh1HN8diShO5naR/1w68JYTlMpBuI+C7U84YLUV4Lnar1kYcrp2pc8rKHs6aDw1m/idODAlDM2poSSp+P7ljdOY2eKpPjgmiVP2sgA9QRr9iT/JWYtvSJIw4ZsYLGT0vKJpPWWzRbzX6I33pGNyLyzK5vPdUryG9Fr/tSlTVpiq7mC4CwPL0alP24WPjq3RhIVB/6/FSCPNTHC+6VR+X1SSwiMAkKHJpzK6mCw8oi8mLk5p3VRPc3RhK3vfbKAhq/Azab4lSYnDpPhJQZxmVSGDZ7OQm+jTGdBc2PZDs2CfAHokhZQA8LR1ddP1OfifJowUdHBzgEaPyc4yh2up1BJhodXwuu8g2MTuGqvfAKYgm/cqUoF9tW7l1GxHxDzPysJKh7qRn8wcCiHfOHIpm441I8oR/Qc71fBhV6D31XB/nyDGpOifuaG42gftI7SOgxfK6ShoatWbUDPd2HTmz8ZJEbf8dA1udsXlasXM9PgNSJTBgGprKY00S3NZAz/0hgLfgmXStnkhwurh9gdm1rxHeH/GgbEKIJpqHgwb5sdhN8294liCe0GOJGZyk4AzbTyb9tDhxExRhcHTLUa3i//gcYq4UU2PtTFWGhzQtKPtZAmfUbRFG+2ljwveDGt2dMoukHHrsbUqZgEwiqYpP6y4Y1NXXFO2DckDHVBbJJMaeY7muPwXebhxlhzkUatZbhe0QaOmGqWrR/jDrzoNagV0ibaJLM4KNNHnmIR4XzyxSYq0k9fF/j69AqK05i1JVRLbPdK+8qS+wNG","webUmidToken":"T11BAD007E91D3135A9A7CB211B61B000D592779860EC49419349ED6B95"}
 * {"logs":"{\"index_ialldf9k0me_1560951748972\":{\"ext\":\"{\\\"method\\\":\\\"click\\\",\\\"target\\\":\\\"李克强会见联合国大会主席埃斯皮诺萨\\\",\\\"dataId\\\":\\\"grid-business-title\\\",\\\"componentWrapper\\\":\\\"231c\\\",\\\"slotId\\\":\\\"1566\\\"}\",\"url\":\"https://www.xuexi.cn/\",\"refer\":\"https://pc.xuexi.cn/points/login.html?ref=https%3A%2F%2Fwww.xuexi.cn%2F\",\"read_id\":\"index_ialldf9k0me_1560951748972\",\"page_uuid\":\"index\"}}","uid":"8b07a940-3c0c-11e9-ae50-3981e89811ea","token":"15aff8fa71a24fd095011592ea5f87ed","sid":"79a86277603042feddbdcd5027f13a9b"}
 */
// the tabs were unattached when created by clicking the link. but the browser_context_id are same. the open_id are same too.
// post to: https://iflow-api.xuexi.cn/logflow/api/v1/pclog
#[test]
fn t_get_content_in_iframe() {
    tutil::setup_logger(vec!["browser_async::task_queue", "browser_async::chrome_browser"]).expect("fern log should work.");

    let my_page = GetContentInIframe::default();
    // let my_page = GetContentInIframe::new_visible();

    let mut runtime = tokio::runtime::Runtime::new().expect("Unable to create a runtime");
    runtime
        .block_on(my_page.into_future())
        .expect("tokio should success.");
}