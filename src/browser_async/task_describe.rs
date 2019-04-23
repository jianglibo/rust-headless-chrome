use crate::protocol::{dom, target, page};
use super::id_type as ids;
use super::page_message::{PageEventName, ChangingFrame};
use super::dev_tools_method_util::{SessionId};
// use super::inner_event::{InnerEvent, inner_events};
use crate::browser::tab::element::BoxModel;
use log::*;
use failure;
use crate::browser_async::dev_tools_method_util::{MethodUtil, ChromePageError, MethodBeforSendResult, MethodTuple};

#[derive(Debug)]
pub enum TaskDescribe {
    QuerySelector(QuerySelector),
    DescribeNode(DescribeNode),
    ResolveNode(ResolveNode),
    GetBoxModel(GetBoxModel),
    SetChildNodes(target::TargetId, dom::NodeId, Vec<dom::Node>),
    GetDocument(GetDocument),
    PageEnable(ids::Task, target::TargetId, SessionId),
    Interval,
    PageEvent(PageEventName),
    FrameNavigated(target::TargetId, ChangingFrame),
    LoadEventFired(target::TargetId, f32),
    TargetInfoChanged(target::TargetInfo),
    PageCreated(target::TargetInfo, Option<&'static str>),
    PageAttached(target::TargetInfo, SessionId),
    ScreenShot(ScreenShot),
    Fail,
}

impl std::convert::TryFrom<&TaskDescribe> for MethodTuple {
        type Error = failure::Error;

        fn try_from(task_describe: &TaskDescribe) -> Result<Self, Self::Error> {
                match task_describe {
                        TaskDescribe::QuerySelector(QuerySelector {
                                node_id: Some(node_id_value),
                                session_id,
                                selector,
                                ..
                                }) => {
                                        MethodUtil::create_msg_to_send_with_session_id(
                                                dom::methods::QuerySelector {
                                                node_id: *node_id_value,
                                                selector,
                                                },
                                                &session_id,
                                        )
                                }
                        TaskDescribe::DescribeNode(DescribeNode {
                                node_id,
                                backend_node_id,
                                depth,
                                session_id,
                                ..
                                }) => {
                                    MethodUtil::create_msg_to_send_with_session_id(
                                        dom::methods::DescribeNode {
                                        node_id: *node_id,
                                        backend_node_id: *backend_node_id,
                                        depth: *depth,
                                        },
                                        &session_id,
                                        )
                                }
                        TaskDescribe::GetDocument(GetDocument{
                                depth,
                                pierce,
                                session_id,
                                ..
                        }) => {
                                MethodUtil::create_msg_to_send_with_session_id(
                                dom::methods::GetDocument {
                                        depth: depth.or(Some(1)),
                                        pierce: Some(*pierce),
                                },
                                &session_id,
                                )
                        }
                        _ => {  
                                error!("task describe to string failed. {:?}", task_describe);
                                Err(ChromePageError::TaskDescribeConvert.into())
                        }
                }
        }
}

#[derive(Debug, Clone)]
pub struct ScreenShot {
        pub task_id: ids::Task,
        pub target_id: target::TargetId,
        pub session_id: Option<SessionId>,
        pub is_manual: bool,
        pub selector: Option<&'static str>,
        pub format: page::ScreenshotFormat,
        pub clip: Option<page::Viewport>,
        pub from_surface: bool,
        pub base64: Option<String>,
}

#[derive(Debug)]
pub struct GetBoxModel {
        pub task_id: usize,
        pub target_id: target::TargetId,
        pub session_id: Option<SessionId>,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub selector: Option<&'static str>,
        pub backend_node_id: Option<dom::NodeId>,
        pub object_id: Option<ids::RemoteObject>,
        pub found_box: Option<BoxModel>
}

#[derive(Debug)]
pub struct ResolveNode {
        pub task_id: usize,
        pub target_id: target::TargetId,
        pub session_id: Option<SessionId>,
        pub is_manual: bool,
        pub selector: Option<&'static str>,
        pub node_id: Option<dom::NodeId>,
        pub backend_node_id: Option<dom::NodeId>,
        pub object_group: Option<String>,
        pub execution_context_id: Option<String>,
}

#[derive(Debug)]
pub struct QuerySelector {
        pub task_id: usize,
        pub target_id: target::TargetId,
        pub session_id: Option<SessionId>,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub found_node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}

#[derive(Debug)]
pub struct GetDocument {
        pub task_id: usize,
        pub target_id: target::TargetId,
        pub session_id: Option<SessionId>,
        pub is_manual: bool,
        pub depth: Option<u8>,
        pub pierce: bool,
        pub root_node: Option<dom::Node>,
}

#[derive(Debug)]
pub struct DescribeNode {
        pub task_id: usize,
        pub session_id: Option<SessionId>,
        pub target_id: target::TargetId,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub backend_node_id: Option<dom::NodeId>,
        pub found_node: Option<dom::Node>,
        pub selector: Option<&'static str>,
        pub depth: Option<i8>,
        pub object_id: Option<ids::RemoteObject>,
        pub pierce: bool,
}