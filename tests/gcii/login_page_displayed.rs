use headless_chrome::browser_async::page_message::{
    write_base64_str_to, MethodCallDone, PageResponse, ReceivedEvent,
};
use headless_chrome::browser_async::task_describe::EvaluateTaskBuilder;
use headless_chrome::protocol::target;
use log::*;
use std::default::Default;

use std::fs;
use std::path::Path;

use super::{GetContentInIframe, PageState, PAGE_STATE};

impl GetContentInIframe {
    pub fn login_page_displayed(
        &mut self,
        maybe_target_id: Option<&target::TargetId>,
        page_response: PageResponse,
    ) {
        match page_response {
            PageResponse::ReceivedEvent(received_event) => {
                match received_event {
                    ReceivedEvent::FrameStoppedLoading(frame_id) => {
                        let tab = self
                            .get_tab(maybe_target_id)
                            .expect("tab should exists. FrameStoppedLoading");
                        tab.lifecycle_events_enable();
                        
                        if let Some(_frame) = tab.changing_frames
                            .find_frame_by_id(&frame_id)
                            .filter(|f| f.name == Some("ddlogin-iframe".into()))
                        {
                            if let Some(context) =
                                tab.find_execution_context_id_by_frame_name("ddlogin-iframe")
                            {
                                info!("execution_context_description: {:?}", context);
                                let mut tb = EvaluateTaskBuilder::default();
                                tb.expression(r#"document.querySelector('div#qrcode.login_qrcode_content img').getAttribute('src')"#).context_id(context.id);
                                let t1 = tab.evaluate_task_named(tb, "get-img-object");
                                tab.execute_one_task(t1);
                                self.ddlogin_frame_stopped_loading = true;
                            } else {
                                panic!("cannot find execution_context_description");
                            }
                        }
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
                if let MethodCallDone::Evaluate(task) = method_call_done {
                    let file_name = "target/qrcode.png";
                    let path = Path::new(file_name);
                    if path.exists() && path.is_file() {
                        fs::remove_file(file_name).unwrap();
                    }

                    let base64_data = task.get_string_result().and_then(|v| {
                        let mut ss = v.splitn(2, ',').fuse();
                        ss.next();
                        ss.next()
                    });

                    write_base64_str_to(file_name, base64_data)
                        .expect("write_base64_str_to success.");
                    assert!(path.exists());
                    self.state = PageState::WaitingForQrcodeScan;
                    // *PAGE_STATE.lock().expect("PAGE_STATE") = PageState::WaitingForQrcodeScan;
                    let exe = std::fs::canonicalize("./target/qrcode.png").expect("exists.");
                    #[cfg(windows)]
                    {
                    std::process::Command::new("cmd")
                        .args(&[
                            "/C",
                            "C:/Program Files/internet explorer/iexplore.exe",
                            exe.to_str().expect("should convert to string."),
                        ])
                        .spawn()
                        .expect("failed to execute process");
                    }
                }
            }
            _ => {}
        }
    }
}