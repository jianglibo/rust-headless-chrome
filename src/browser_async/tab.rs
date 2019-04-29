use super::chrome_debug_session::ChromeDebugSession;
use super::dev_tools_method_util::{next_call_id, MethodDestination, MethodUtil, SessionId};
use super::id_type as ids;
use super::page_message::ChangingFrame;
use super::task_describe::{self as tasks, TaskDescribe};
use crate::protocol::{self, dom, page, target};
use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub target_info: protocol::target::TargetInfo,
    pub session_id: Option<SessionId>,
    pub root_node: Option<dom::Node>,
    pub changing_frames: HashMap<String, ChangingFrame>,
    pub temporary_node_holder: HashMap<dom::NodeId, Vec<dom::Node>>,
}

impl Tab {
    pub fn new(
        target_info: protocol::target::TargetInfo,
        chrome_session: Arc<Mutex<ChromeDebugSession>>,
    ) -> Self {
        Self {
            target_info,
            chrome_session,
            session_id: None,
            root_node: None,
            changing_frames: HashMap::new(),
            temporary_node_holder: HashMap::new(),
        }
    }
    pub fn navigate_to(&mut self, url: &str) {
        let method_str = MethodUtil::create_msg_to_send_with_session_id(
            page::methods::Navigate { url },
            &self.session_id,
            next_call_id(),
        );
        self.chrome_session
            .lock()
            .unwrap()
            .send_message_direct(method_str);
    }

    pub fn main_frame(&self) -> Option<&page::Frame> {
        self.changing_frames.values().find_map(|cf| match cf {
            ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm)
                if fm.parent_id.is_none() =>
            {
                Some(fm)
            }
            _ => None,
        })
    }

    pub fn _frame_navigated(&mut self, changing_frame: ChangingFrame) {
        if let ChangingFrame::Navigated(frame) = &changing_frame {
            let frame_id = frame.id.clone();
            self.changing_frames.insert(frame_id, changing_frame);
        }
    }

    pub fn node_arrived(&mut self, parent_node_id: dom::NodeId, mut nodes: Vec<dom::Node>) {
        self.temporary_node_holder
            .entry(parent_node_id)
            .or_insert_with(|| vec![])
            .append(&mut nodes);
    }

    pub fn node_returned(&mut self, node: Option<dom::Node>) {
        if let Some(nd) = node {
            if let Some(parent_id) = nd.parent_id {
                self.temporary_node_holder
                    .entry(parent_id)
                    .or_insert_with(|| vec![])
                    .push(nd);
            } else {
                warn!("node_returned has no parent_id. treat as 0.");
                self.temporary_node_holder
                    .entry(0_u16)
                    .or_insert_with(|| vec![])
                    .push(nd);
            }
        } else {
            error!("return None Node.");
        }
    }

    pub fn find_node_by_id(&self, node_id: dom::NodeId) -> Option<&dom::Node> {
        self.temporary_node_holder
            .values()
            .flatten()
            .find(|nd| nd.node_id == node_id)
    }

    pub fn find_navigated_frame<F>(&self, mut filter: F) -> Option<&page::Frame>
    where
        F: FnMut(&page::Frame) -> bool,
    {
        self.changing_frames
            .values()
            .filter_map(|cf| match cf {
                ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm) => Some(fm),
                _ => None,
            })
            .find(|frame| filter(frame))
    }

    pub fn get_document(&mut self, depth: Option<u8>, manual_task_id: Option<ids::Task>) {
        let task = tasks::GetDocumentBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .depth(depth)
            .build()
            .unwrap();
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![task.into()]);
    }

    pub fn dom_query_selector_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<usize>,
    ) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(self.get_query_selector(selector, manual_task_id));
    }

    pub fn describe_node_by_selector(
        &mut self,
        selector: &'static str,
        depth: Option<i8>,
        manual_task_id: Option<ids::Task>,
    ) {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let describe_node = tasks::DescribeNodeBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .depth(depth)
            .build()
            .unwrap();
        pre_tasks.push(describe_node.into());
        self.chrome_session.lock().unwrap().execute_task(pre_tasks);
    }

    fn get_query_selector(
        &self,
        selector: &'static str,
        manual_task_id: Option<ids::Task>,
    ) -> Vec<TaskDescribe> {
        let get_document = tasks::GetDocumentBuilder::default()
            .common_fields(self.get_c_f(None))
            .build()
            .unwrap();
        let query_select = tasks::QuerySelectorBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .build()
            .unwrap();
        vec![get_document.into(), query_select.into()]
    }

    fn get_box_model(
        &self,
        selector: &'static str,
        manual_task_id: Option<ids::Task>,
    ) -> Vec<TaskDescribe> {
        let mut pre_tasks = self.get_query_selector(selector, None);
        let get_box_model = tasks::GetBoxModelBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .build()
            .unwrap();
        pre_tasks.push(get_box_model.into());
        pre_tasks
    }

    pub fn get_box_model_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<ids::Task>,
    ) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(self.get_box_model(selector, manual_task_id));
    }
    pub fn capture_screenshot_by_selector(
        &mut self,
        selector: &'static str,
        format: page::ScreenshotFormat,
        from_surface: bool,
        manual_task_id: Option<ids::Task>,
    ) {
        let screen_shot = tasks::ScreenShotBuilder::default()
            .common_fields(self.get_c_f(manual_task_id))
            .selector(selector)
            .format(format)
            .from_surface(from_surface)
            .build()
            .unwrap();
        let mut pre_tasks = self.get_box_model(selector, None);
        pre_tasks.push(screen_shot.into());
        self.chrome_session.lock().unwrap().execute_task(pre_tasks);
    }

    fn get_c_f(&self, task_id: Option<ids::Task>) -> tasks::CommonDescribeFields {
        tasks::get_common_fields_builder(
            self.target_info.target_id.clone(),
            self.session_id.clone(),
            task_id,
        )
        .build()
        .unwrap()
    }

    pub fn page_enable(&mut self) {
        self.chrome_session
            .lock()
            .unwrap()
            .execute_task(vec![TaskDescribe::PageEnable(self.get_c_f(None))]);
    }

    // pub fn capture_screenshot(
    //     &mut self,
    //     format: page::ScreenshotFormat,
    //     clip: Option<page::Viewport>,
    //     from_surface: bool,
    //     manual_task_id: Option<ids::Task>
    // ) {
    //     let (format, quality) = match format {
    //         page::ScreenshotFormat::JPEG(quality) => {
    //             (page::InternalScreenshotFormat::JPEG, quality)
    //         }
    //         page::ScreenshotFormat::PNG => (page::InternalScreenshotFormat::PNG, None),
    //     };
    //     let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //     let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(page::methods::CaptureScreenshot {
    //             format,
    //             clip,
    //             quality,
    //             from_surface,
    //         }, &self.session_id)
    //         .unwrap();
    //         let dn = tasks::ScreenShot {
    //             task_id: this_task_id,
    //             target_id: self.target_info.target_id.clone(),
    //             session_id: self.session_id.clone(),
    //             is_manual,
    //             format,
    //             clip,
    //             quality,
    //             from_surface,
    //             base64: None,
    //         };
    //         self.chrome_session.lock().unwrap().add_task_and_method_map(
    //             mid.unwrap(),
    //             this_task_id,
    //             tasks::TaskDescribe::ScreenShot(dn),
    //         );
    //     self.chrome_session.lock().unwrap().send_message(method_str);
    // }
    pub fn attach_to_page(&mut self) {
        let method_str = MethodUtil::create_msg_to_send(
            target::methods::AttachToTarget {
                target_id: &(self.target_info.target_id),
                flatten: None,
            },
            MethodDestination::Browser,
            next_call_id(),
        );
        self.chrome_session
            .lock()
            .unwrap()
            .send_message_direct(method_str);
    }
}
