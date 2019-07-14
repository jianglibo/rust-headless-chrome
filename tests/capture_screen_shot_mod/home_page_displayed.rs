use headless_chrome::browser_async::page_message::{
    write_base64_str_to, MethodCallDone, PageResponse, ReceivedEvent,
};

use headless_chrome::protocol::{target, page};
use log::*;

use std::fs;
use std::path::Path;

use super::{CaptureScreenShotTest, PageState};

impl CaptureScreenShotTest {
    pub fn home_page_displayed(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    // ReceivedEvent::LifeCycle => {
                    //     info!("life cycle happend........................");
                    //     let tab = self
                    //         .get_tab(maybe_target_id)
                    //         .expect("tab should exists. LoadEventFired");
                    //     if tab.life_cycle_event_count("load") == 2 {
                    //         tab.display_full_page();
                    //     }
                    // }
                    ReceivedEvent::LoadEventFired(_t) => {
                        info!("page loaded........................");
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. LoadEventFired");
                        // tab.capture_screenshot_view_jpeg();
                        // tab.get_layout_metrics();
                        // let task = tab.capture_screenshot_jpeg_task(Some(100), Some(false));
                        // let tasks = tab.capture_screenshot_by_selector_jpeg_task("body", Some(100), None, None);
                        // tab.task_queue.add_delayed_many(tasks, 3);
                        // let set_metrics = tab.set_device_metrics_override_simple_task(1000, 20000);
                        // tab.execute_one_task(set_metrics);
                        let tasks = tab.display_full_page_task();
                        tab.task_queue.add_delayed_many(tasks, 6);
                    }
                    ReceivedEvent::FrameStoppedLoading(frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        let _frame = tab.find_frame_by_id(&frame_id).expect("frame should be found by frame_id.");
                        // if frame.
                    }
                    ReceivedEvent::ResponseReceived(_event) => {}
                    ReceivedEvent::RequestWillBeSent(_event) => {}
                    ReceivedEvent::DataReceived(_event) => {}
                    _ => {
                        // info!("got unused page event {:?}", received_event);
                    }
                }
            }
            PageResponse::MethodCallDone(method_call_done) => {
                match  method_call_done {
                    MethodCallDone::CaptureScreenshot(capture_screen_shot) => {
                        // info!("got screen shot: {:?}", capture_screen_shot.task_result);
                        info!("got screen shot: >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>");
                        assert!(capture_screen_shot.task_result.is_some());

                        let file_name = if let page::ScreenshotFormat::PNG = capture_screen_shot.format {
                            "target/abc.png"
                        } else {
                            "target/abc.jpeg"
                        };
                        let path = Path::new(file_name);
                        if path.exists() && path.is_file() {
                            fs::remove_file(file_name).unwrap();
                        }
                        write_base64_str_to(file_name, capture_screen_shot.task_result).expect("write success.");
                        assert!(path.exists());
                    }
                    MethodCallDone::GetLayoutMetrics(task) => {
                        info!("got get_layout_metrics: {:?}", task);
                    }
                    MethodCallDone::CanEmulate(task) => {
                        info!("got can_emulate answer: {:?}", task);
                    }
                    MethodCallDone::SetDeviceMetricsOverride(task) => {
                        info!("got set_device_metrics_override {:?}", task);
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        tab.capture_screenshot_surface_jpeg(Some(100));
                    }
                    _ => {
                        error!("got other method_call_done. {:?}", method_call_done);
                    }
                };
            }
            _ => {}
        }
    }
}