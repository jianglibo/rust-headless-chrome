
use super::chrome_debug_session::ChromeDebugSession;
use super::dev_tools_method_util::{MethodDestination, MethodUtil, SessionId};
use super::id_type as ids;
use crate::browser_async::unique_number::{self, create_if_no_manual_input};
use super::page_message::{ChangingFrame, ChangingFrameTree};
use super::task_describe as tasks;
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
    pub changing_frame_tree: ChangingFrameTree,
    pub temporary_node_holder: Vec<dom::Node>,
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
            changing_frame_tree: Default::default(),
            temporary_node_holder: vec![],
        }
    }
    pub fn navigate_to(&mut self, url: &str) {
        let (_, method_str, _) = MethodUtil::create_msg_to_send_with_session_id(
            page::methods::Navigate { url },
            &self.session_id,
        )
        .unwrap();
        self.chrome_session.lock().unwrap().send_message(method_str);
    }

    pub fn frame_tree(&self) -> &ChangingFrameTree {
        &self.changing_frame_tree
    }

    pub fn main_frame(&self) -> Option<&ChangingFrame> {
        self.changing_frame_tree.changing_frame.as_ref()
    }

    pub fn _frame_navigated(&mut self, changing_frame: ChangingFrame) {
        if let ChangingFrame::Navigated(frame) = &changing_frame {
            let frame_id = frame.id.clone();
            let parent_id = frame.parent_id.clone();
            if parent_id.is_none() {
                self.changing_frame_tree
                    .changing_frame
                    .replace(changing_frame);
            } else {
                self.changing_frame_tree
                    .child_changing_frames
                    .insert(frame_id, changing_frame);
            }
        }
    }

    pub fn node_arrived(&mut self, parent_node_id: dom::NodeId, nodes: Vec<dom::Node>) {

    }

    pub fn find_node_by_id(&self, node_id: dom::NodeId) -> Option<&dom::Node> {
        if let Some(node) = self.temporary_node_holder.iter().find(|n| n.node_id == node_id) {
            Some(node)
        } else {
            if let Some(root_node) = &self.root_node {
                root_node.find(|n| n.node_id == node_id)
            } else {
                error!("tab's root node is None.");
                None
            }
        }
    }

    pub fn is_main_frame_navigated(&self) -> bool {
        if let Some(ChangingFrame::Navigated(_)) = &self.changing_frame_tree.changing_frame {
            true
        } else {
            false
        }
    }

    pub fn is_frame_navigated(&self, frame_name: &'static str) -> Option<&page::Frame> {
        let op = self
            .changing_frame_tree
            .child_changing_frames
            .values()
            .find(|cv| {
                if let ChangingFrame::Navigated(frame) = cv {
                    frame.name == Some(frame_name.into())
                } else {
                    false
                }
            });
        if let Some(ChangingFrame::Navigated(frame)) = op {
            Some(frame)
        } else {
            None
        }
    }


    pub fn get_document(
        &mut self,
        manual_task_id: Option<ids::Task>,
    ) -> (Option<ids::Task>, Option<dom::NodeId>) {
        if let Some(root_node) = &self.root_node {
            (None, Some(root_node.node_id))
        } else {
            let (this_id, _) = create_if_no_manual_input(manual_task_id);
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(
                dom::methods::GetDocument {
                    depth: Some(0),
                    pierce: Some(false),
                },
                &self.session_id,
            )
            .unwrap();
            self.chrome_session.lock().unwrap().add_task_and_method_map(
                mid.unwrap(),
                this_id,
                tasks::TaskDescribe::GetDocument(this_id, None, None),
            );
            self.chrome_session.lock().unwrap().send_message(method_str);
            (Some(this_id), None)
        }
    }

    pub fn dom_query_selector_by_selector(
        &mut self,
        selector: &'static str,
        manual_task_id: Option<usize>,
    ) -> (Option<ids::Task>, Option<dom::NodeId>) {
        let (this_task_id, is_manual) = create_if_no_manual_input(manual_task_id);
        let mut qs = tasks::QuerySelector {
            selector,
            is_manual,
            session_id: self.session_id.clone(),
            target_id: self.target_info.target_id.clone(),
            node_id: None,
            found_node_id: None,
            task_id: this_task_id,
        };
        match self.get_document(None) {
            (Some(get_document_task_id), _) => {
                self.chrome_session
                    .lock()
                    .unwrap()
                    .add_task(qs.task_id, tasks::TaskDescribe::QuerySelector(qs));
                self.chrome_session
                    .lock()
                    .unwrap()
                    .add_waiting_task(get_document_task_id, this_task_id);
            }
            (_, Some(node_id)) => {
                qs.node_id = Some(node_id);
                self.chrome_session
                    .lock()
                    .unwrap()
                    .dom_query_selector(tasks::TaskDescribe::QuerySelector(qs));
            }
            _ => {
                error!("get_document return impossible value combination.");
            }
        }
        (Some(this_task_id), None)
    }

    pub fn attach_to_page(&mut self) {
        let (_, method_str, _) = MethodUtil::create_msg_to_send(
            target::methods::AttachToTarget {
                target_id: &(self.target_info.target_id),
                flatten: None,
            },
            MethodDestination::Browser,
            None,
        )
        .unwrap();
        self.chrome_session.lock().unwrap().send_message(method_str);
    }

    pub fn page_enable(&mut self) {
        let this_task_id = unique_number::create_one();
        let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(
            page::methods::Enable {},
            &self.session_id,
        )
        .unwrap();
        self.chrome_session.lock().unwrap().add_task_and_method_map(
            mid.unwrap(),
            this_task_id,
            tasks::TaskDescribe::PageEnable(this_task_id, self.target_info.target_id.clone(), self.session_id.clone().unwrap()),
        );
        self.chrome_session.lock().unwrap().send_message(method_str);
    }
}