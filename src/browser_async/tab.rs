use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::{self, page, dom};
use super::chrome_debug_session::{ChromeDebugSession};
use super::dev_tools_method_util::{MethodUtil, SessionId};
use super::id_type as ids;
use super::task_describe as tasks;
use super::unique_number::{create_if_no_manual_input};

#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub target_info: protocol::target::TargetInfo,
    pub session_id: Option<SessionId>,
    root_node: Option<dom::Node>,
}

impl Tab {
    pub fn new(target_info: protocol::target::TargetInfo, chrome_session: Arc<Mutex<ChromeDebugSession>>) -> Self {
        Self {
            target_info,
            chrome_session,
            session_id: None,
            root_node: None,
        }
    }
    pub fn navigate_to(&mut self, url: &str) {
        let (_, method_str, _) = MethodUtil::create_msg_to_send_with_session_id(page::methods::Navigate { url }, &self.session_id)
            .unwrap();
        self.chrome_session.lock().unwrap().send_message(method_str);
    }

    
    pub fn get_document(
        &mut self,
        manual_task_id: Option<ids::Task>,
    ) -> (Option<ids::Task>, Option<dom::NodeId>) {
        if let Some(root_node) = &self.root_node {
            (None, Some(root_node.node_id))
        } else {
            let (this_id, _) = create_if_no_manual_input(manual_task_id);
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(dom::methods::GetDocument {
                    depth: Some(0),
                    pierce: Some(false),
                }, &self.session_id)
                .unwrap();
            self.chrome_session.lock().unwrap().add_task_and_method_map(mid.unwrap(), this_id, tasks::TaskDescribe::GetDocument(this_id));
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
            node_id: None,
            task_id: this_task_id,
        };
        match self.get_document(None) {
            (Some(task_id), _) => {
                // if root node is not ready, will return a task_id.
                self.chrome_session.lock().unwrap().add_task(qs.task_id, tasks::TaskDescribe::QuerySelector(qs));
                self.chrome_session.lock().unwrap().add_waiting_task(this_task_id, task_id);
            }
            (_, Some(node_id)) => {
                // self.dom_query_selector_extra(node_id, t_id);
                qs.node_id = Some(node_id);
                self.dom_query_selector(tasks::TaskDescribe::QuerySelector(qs));
            }
            _ => {
                error!("get_document return impossible value combination.");
            }
        }
        (Some(this_task_id), None)
    }

    fn dom_query_selector(&mut self, task: tasks::TaskDescribe) {
        if let tasks::TaskDescribe::QuerySelector(tasks::QuerySelector {
            task_id,
            is_manual,
            node_id: Some(node_id_value),
            selector,
        }) = task
        {
            let (_, method_str, mid) = MethodUtil::create_msg_to_send_with_session_id(dom::methods::QuerySelector {
                    node_id: node_id_value,
                    selector: selector,
                }, &self.session_id)
                .unwrap();
            self.chrome_session.lock().unwrap().add_task_and_method_map(task_id, task_id, task);
            self.chrome_session.lock().unwrap().send_message(method_str);
        } else {
            error!("it's not a query selector task.");
            panic!("it's not a query selector task.");
        }
    }
}