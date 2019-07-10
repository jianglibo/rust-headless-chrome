use headless_chrome::browser_async::page_message::{
    write_base64_str_to, MethodCallDone, PageResponse, ReceivedEvent,
};
use headless_chrome::browser_async::task_describe::RuntimeEvaluateTaskBuilder;
use headless_chrome::protocol::{target, page};
use log::*;
use std::default::Default;

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
                    ReceivedEvent::LoadEventFired(_t) => {
                        info!("page loaded........................");
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. LoadEventFired");
                        // tab.capture_screenshot_view_jpeg();
                        tab.get_layout_metrics();
                    }
                    ReceivedEvent::FrameStoppedLoading(frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        let frame = tab.find_frame_by_id(&frame_id).expect("frame should be found by frame_id.");
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
                        info!("got screen shot: {:?}", capture_screen_shot.task_result);
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
                    _ => {
                        error!("got other method_call_done. {:?}", method_call_done);
                    }
                };
                // if let MethodCallDone::Evaluate(task) = method_call_done {
                    // let file_name = "target/qrcode.png";
                    // let path = Path::new(file_name);
                    // if path.exists() && path.is_file() {
                    //     fs::remove_file(file_name).unwrap();
                    // }

                    // let base64_data = task.get_string_result().and_then(|v| {
                    //     let mut ss = v.splitn(2, ',').fuse();
                    //     ss.next();
                    //     ss.next()
                    // });

                    // write_base64_str_to(file_name, base64_data)
                    //     .expect("write_base64_str_to success.");
                    // assert!(path.exists());
                    // self.state = PageState::WaitingForQrcodeScan;
                    // let exe = std::fs::canonicalize("./target/qrcode.png").expect("exists.");
                    // error!("{:?}", exe);
                    // std::process::Command::new("cmd")
                    //     .args(&[
                    //         "/C",
                    //         "C:/Program Files/internet explorer/iexplore.exe",
                    //         exe.to_str().expect("should convert to string."),
                    //     ])
                    //     .spawn()
                    //     .expect("failed to execute process");
                // }
            }
            _ => {}
        }
    }
}