
use super::chrome_debug_session::ChromeDebugSession;
use super::dev_tools_method_util::{MethodDestination, MethodUtil, SessionId, MethodTuple, next_call_id};
use super::id_type as ids;
use crate::browser_async::unique_number::{self, create_if_no_manual_input};
use super::page_message::{ChangingFrame};
use super::task_describe::{self as tasks, TaskDescribe, CommonDescribeFields};
use crate::protocol::{self, dom, page, target};
use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::convert::TryFrom;


#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub target_info: protocol::target::TargetInfo,
    pub session_id: Option<SessionId>,
    pub root_node: Option<dom::Node>,
    pub changing_frames: HashMap<String, ChangingFrame>,
    pub temporary_node_holder: HashMap<dom::NodeId, Vec<dom::Node>>,
    get_document_task_id: Option<ids::Task>,
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
            get_document_task_id: None,
        }
    }
    pub fn navigate_to(&mut self, url: &str) {
        let method_str = MethodUtil::create_msg_to_send_with_session_id(
            page::methods::Navigate { url },
            &self.session_id,
            next_call_id()
        );
        self.chrome_session.lock().unwrap().send_message_direct(method_str);
    }

    pub fn main_frame(&self) -> Option<&page::Frame> {
        self.changing_frames.values().find_map(|cf| match cf {
            ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm)  if fm.parent_id.is_none() => Some(fm),
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
        self.temporary_node_holder.entry(parent_node_id).or_insert_with(||vec![]).append(&mut nodes);
    }

    pub fn node_returned(&mut self, node: Option<dom::Node>) {
        if let Some(nd) = node {
            if let Some(parent_id) = nd.parent_id {
                self.temporary_node_holder.entry(parent_id).or_insert_with(||vec![]).push(nd);
            } else {
                warn!("node_returned has no parent_id. treat as 0.");
                self.temporary_node_holder.entry(0_u16).or_insert_with(||vec![]).push(nd);
            }
        } else {
            error!("return None Node.");
        }
    }

    pub fn find_node_by_id(&self, node_id: dom::NodeId) -> Option<&dom::Node> {
        self.temporary_node_holder.values().flatten().find(|nd|nd.node_id == node_id)
    }

    pub fn find_navigated_frame<F>(&self, mut filter: F) -> Option<&page::Frame> 
        where F: FnMut(&page::Frame) -> bool {
        self.changing_frames.values().filter_map(|cf| match cf {
            ChangingFrame::Navigated(fm) | ChangingFrame::StoppedLoading(fm) => Some(fm),
            _ => None,
        }).find(|frame| filter(frame))
    }

    fn t_s_id(&self) -> (target::TargetId, Option<SessionId>) {
        (self.target_info.target_id.clone(), self.session_id.clone())
    }

    pub fn get_document(&mut self, depth: Option<u8>, manual_task_id: Option<ids::Task>) {
        let (t_id, s_id) = self.t_s_id();
        let task = tasks::GetDocumentTaskBuilder::new(t_id, s_id).task_id_opt(manual_task_id).depth_opt(depth).build();
        self.chrome_session.lock().unwrap().execute_task(vec![task]);
    }

    pub fn dom_query_selector_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<usize>,
    ) {
        let (t_id, s_id) = self.t_s_id();
        let get_document_task = tasks::GetDocumentTaskBuilder::new(t_id.clone(), s_id.clone()).build();
        let query_select_task = tasks::QuerySelectorTaskBuilder::new(t_id, s_id, selector).task_id_opt(manual_task_id).build();
        self.chrome_session.lock().unwrap().execute_task(vec![get_document_task, query_select_task]);
    }

    pub fn describe_node_by_selector(&mut self, selector: &'static str, depth: Option<i8>, manual_task_id: Option<ids::Task>) {
        let (t_id, s_id) = self.t_s_id();
        let get_document_task = tasks::GetDocumentTaskBuilder::new(t_id.clone(), s_id.clone()).build();
        let query_select_task = tasks::QuerySelectorTaskBuilder::new(t_id.clone(), s_id.clone(), selector).build();
        let describe_node_task = tasks::DescribeNodeTaskBuilder::new(t_id, s_id, selector).task_id_opt(manual_task_id).depth_opt(depth).build();
        self.chrome_session.lock().unwrap().execute_task(vec![get_document_task, query_select_task, describe_node_task]);
    }

    pub fn get_box_model_by_selector(&mut self, selector: &'static str, manual_task_id: Option<ids::Task>) {
        let (t_id, s_id) = self.t_s_id();
        let get_document_task = tasks::GetDocumentTaskBuilder::new(t_id.clone(), s_id.clone()).build();
        let query_select_task = tasks::QuerySelectorTaskBuilder::new(t_id.clone(), s_id.clone(), selector).build();
        let common_fields = tasks::get_common_fields_builder(t_id.clone(), s_id.clone(), manual_task_id).build().unwrap();
        let get_box_model = tasks::GetBoxModelBuilder::default().common_fields(common_fields).selector(selector).build().unwrap();
        self.chrome_session.lock().unwrap().execute_task(vec![get_document_task, query_select_task, get_box_model.into()]);
    }

    pub fn page_enable(&mut self) {
        let (t_id, s_id) = self.t_s_id();
        let page_enable_task = tasks::PageEnableTaskBuilder::new(t_id, s_id).build();
        self.chrome_session.lock().unwrap().execute_task(vec![page_enable_task]);
    }

    // pub fn get_document(
    //     &mut self,
    //     depth: Option<u8>,
    //     manual_task_id: Option<ids::Task>,
    // ) -> (Option<ids::Task>, Option<dom::NodeId>) {
    //     if let Some(root_node) = &self.root_node {
    //         (None, Some(root_node.node_id))
    //     } else if self.get_document_task_id.is_none() {
    //             let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //             let call_id = next_call_id();
    //             let common_fields = CommonDescribeFields {
    //                 task_id: this_task_id,
    //                 call_id,
    //                 target_id: self.target_info.target_id.clone(),
    //                 session_id: self.session_id.clone(),
    //                 is_manual,
    //             };
    //             let dc = tasks::GetDocument {
    //                 common_fields,
    //                 depth,
    //                 pierce: false,
    //                 root_node: None,
    //             };
    //             let task = TaskDescribe::GetDocument(dc); 
    //             self.chrome_session.lock().unwrap().send_message_and_save_task(task);
    //             self.get_document_task_id = Some(this_task_id);
    //             (Some(this_task_id), None)
    //         } else {
    //             (self.get_document_task_id, None)
    //         }
    // }

    // pub fn get_document_raw(
    //     &mut self,
    //     depth: Option<u8>,
    //     manual_task_id: Option<ids::Task>,
    // ) -> (Option<ids::Task>, Option<dom::NodeId>) {
    //         let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //         let call_id = next_call_id();
    //         let common_fields = CommonDescribeFields {
    //             task_id: this_task_id,
    //             call_id,
    //             target_id: self.target_info.target_id.clone(),
    //             session_id: self.session_id.clone(),
    //             is_manual,
    //         };
    //         let dc = tasks::GetDocument {
    //             common_fields,
    //             depth,
    //             pierce: false,
    //             root_node: None,
    //         };
    //         let task = TaskDescribe::GetDocument(dc);
    //         let method_str = String::try_from(&task).unwrap();
    //         self.chrome_session.lock().unwrap().send_message_direct(method_str);
    //         (Some(this_task_id), None)
    // }

    // pub fn capture_screenshot_by_selector (
    //     &mut self,
    //     selector: &'static str,
    //     format: page::ScreenshotFormat,
    //     from_surface: bool,
    //     manual_task_id: Option<ids::Task>
    // ) {
    //     let get_box_model_task_id = self.get_box_model_by_selector(selector, None);
            
    //     let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //     let sh = tasks::ScreenShot {
    //         task_id: this_task_id,
    //         target_id: self.target_info.target_id.clone(),
    //         session_id: self.session_id.clone(),
    //         selector: Some(selector),
    //         is_manual,
    //         format,
    //         clip: None,
    //         from_surface,
    //         base64: None,
    //     };
    //     self.chrome_session.lock().unwrap().add_task(sh.task_id, tasks::TaskDescribe::ScreenShot(sh));
    // }

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



    // #[allow(clippy::too_many_arguments)]
    // pub fn describe_node(&mut self, manual_task_id: Option<ids::Task>, node_id: Option<dom::NodeId>, backend_node_id: Option<dom::NodeId>,
    //     object_id: Option<ids::RemoteObject>, depth: Option<i8>, pierce: bool, selector: Option<&'static str>) {
    //     let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //     let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(dom::methods::DescribeNode {
    //             node_id,
    //             backend_node_id,
    //             depth,
    //         }, &self.session_id)
    //         .unwrap();
    //         let dn = tasks::DescribeNode {
    //             task_id: this_task_id,
    //             target_id: self.target_info.target_id.clone(),
    //             session_id: self.session_id.clone(),
    //             is_manual,
    //             node_id,
    //             backend_node_id,
    //             object_id,
    //             depth,
    //             pierce,
    //             selector,
    //             found_node: None,
    //         };
    //         // self.chrome_session.lock().unwrap().add_task_and_method_map(
    //         //     mid.unwrap(),
    //         //     this_task_id,
    //         //     tasks::TaskDescribe::DescribeNode(dn),
    //         // );
    //         let method_str_id = Some((method_str, mid.unwrap()));
    //     self.chrome_session.lock().unwrap().send_message_and_save_task(method_str_id, this_task_id, TaskDescribe::DescribeNode(dn));

    // }


    // pub fn get_box_model_by_node_id(&mut self, node_id: Option<dom::NodeId>, manual_task_id: Option<ids::Task>) -> ids::Task {
    //     self.get_box_model(manual_task_id, None, node_id, None, None)
    // }

    // pub fn get_box_model_by_backend_node_id(&mut self, backend_node_id: Option<dom::NodeId>, manual_task_id: Option<ids::Task>) -> ids::Task {
    //     self.get_box_model(manual_task_id, backend_node_id, None, None, None)
    // }

    // pub fn get_box_model(&mut self, manual_task_id: Option<ids::Task>, backend_node_id: Option<dom::NodeId>, node_id: Option<dom::NodeId>, object_id: Option<ids::RemoteObject>, selector: Option<&'static str>) -> ids::Task {
    //     let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //     let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(dom::methods::GetBoxModel {
    //             node_id,
    //             backend_node_id,
    //             object_id: None,
    //         }, &self.session_id)
    //         .unwrap();
    //         let task = tasks::GetBoxModel {
    //             task_id: this_task_id,
    //             target_id: self.target_info.target_id.clone(),
    //             session_id: self.session_id.clone(),
    //             is_manual,
    //             node_id,
    //             backend_node_id,
    //             object_id,
    //             selector,
    //             found_box: None,
    //         };
    //     let method_str_id = Some((method_str, mid.unwrap()));
    //     self.chrome_session.lock().unwrap().send_message_and_save_task(method_str_id, this_task_id, TaskDescribe::GetBoxModel(task));
    //     this_task_id
    // }

    // pub fn dom_query_selector_by_selector(
    //     &mut self,
    //     selector: &'static str,
    //     manual_task_id: Option<usize>,
    // ) -> (Option<ids::Task>, Option<dom::NodeId>) {
    //     let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
    //     let common_fields = CommonDescribeFields {
    //         is_manual,
    //         session_id: self.session_id.clone(),
    //         target_id: self.target_info.target_id.clone(),
    //         task_id: this_task_id,
    //         call_id: next_call_id(),
    //     };
    //     let mut qs = tasks::QuerySelector {
    //         common_fields,
    //         selector,
    //         node_id: None,
    //         found_node_id: None,
    //     };
    //     match self.get_document(None, None) {
    //         (Some(_get_document_task_id), _) => {
    //             self.chrome_session.lock().unwrap().send_message_and_save_task(TaskDescribe::QuerySelector(qs));
    //         }
    //         (_, Some(node_id)) => {
    //             qs.node_id = Some(node_id);
    //             let task = TaskDescribe::QuerySelector(qs);
    //             self.chrome_session.lock().unwrap().send_message_and_save_task(task);
    //         }
    //         _ => {
    //             error!("get_document return impossible value combination.");
    //         }
    //     }
    //     (Some(this_task_id), None)
    // }

    pub fn attach_to_page(&mut self) {
        let method_str = MethodUtil::create_msg_to_send(
            target::methods::AttachToTarget {
                target_id: &(self.target_info.target_id),
                flatten: None,
            },
            MethodDestination::Browser,
            next_call_id(),
            None,
        );
        self.chrome_session.lock().unwrap().send_message_direct(method_str);
    }

}