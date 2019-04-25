use super::dev_tools_method_util::SessionId;
use super::id_type as ids;
use super::page_message::{ChangingFrame, PageEventName};
use crate::browser::tab::element::BoxModel;
use crate::browser_async::dev_tools_method_util::{ChromePageError, MethodUtil};
use crate::protocol::{dom, page, target};
use failure;
use log::*;

#[derive(Debug)]
pub enum TaskDescribe {
        QuerySelector(QuerySelector),
        DescribeNode(DescribeNode),
        ResolveNode(ResolveNode),
        GetBoxModel(GetBoxModel),
        SetChildNodes(target::TargetId, dom::NodeId, Vec<dom::Node>),
        GetDocument(GetDocument),
        PageEnable(CommonDescribeFields),
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

impl TaskDescribe {
        pub fn get_common_fields(&self) -> Option<&CommonDescribeFields> {
                match &self {
                        TaskDescribe::QuerySelector(query_selector) => {
                                Some(&query_selector.common_fields)
                        }
                        TaskDescribe::DescribeNode(describe_node) => {
                                Some(&describe_node.common_fields)
                        }
                        TaskDescribe::GetDocument(get_document) => {
                                Some(&get_document.common_fields)
                        }
                        TaskDescribe::PageEnable(common_fields) => Some(&common_fields),
                        _ => None,
                }
        }
}

impl std::convert::TryFrom<&TaskDescribe> for String {
        type Error = failure::Error;

        fn try_from(task_describe: &TaskDescribe) -> Result<Self, Self::Error> {
                match task_describe {
                        TaskDescribe::QuerySelector(QuerySelector {
                                node_id: Some(node_id_value),
                                common_fields,
                                selector,
                                ..
                        }) => Ok(MethodUtil::create_msg_to_send_with_session_id(
                                dom::methods::QuerySelector {
                                        node_id: *node_id_value,
                                        selector,
                                },
                                &common_fields.session_id,
                                common_fields.call_id,
                        )),
                        TaskDescribe::DescribeNode(DescribeNode {
                                node_id,
                                backend_node_id,
                                depth,
                                common_fields,
                                ..
                        }) => Ok(MethodUtil::create_msg_to_send_with_session_id(
                                dom::methods::DescribeNode {
                                        node_id: *node_id,
                                        backend_node_id: *backend_node_id,
                                        depth: *depth,
                                },
                                &common_fields.session_id,
                                common_fields.call_id,
                        )),
                        TaskDescribe::GetDocument(GetDocument {
                                depth,
                                pierce,
                                common_fields,
                                ..
                        }) => Ok(MethodUtil::create_msg_to_send_with_session_id(
                                dom::methods::GetDocument {
                                        depth: depth.or(Some(1)),
                                        pierce: Some(*pierce),
                                },
                                &common_fields.session_id,
                                common_fields.call_id,
                        )),
                        TaskDescribe::PageEnable(CommonDescribeFields {
                                session_id,
                                call_id,
                                ..
                        }) => Ok(MethodUtil::create_msg_to_send_with_session_id(
                                page::methods::Enable {},
                                session_id,
                                *call_id,
                        )),
                        _ => {
                                error!("task describe to string failed. {:?}", task_describe);
                                Err(ChromePageError::TaskDescribeConvert.into())
                        }
                }
        }
}

#[derive(Debug, Clone)]
pub struct ScreenShot {
        pub common_fields: CommonDescribeFields,
        pub selector: Option<&'static str>,
        pub format: page::ScreenshotFormat,
        pub clip: Option<page::Viewport>,
        pub from_surface: bool,
        pub base64: Option<String>,
}

#[derive(Debug)]
pub struct GetBoxModel {
        pub common_fields: CommonDescribeFields,
        pub node_id: Option<dom::NodeId>,
        pub selector: Option<&'static str>,
        pub backend_node_id: Option<dom::NodeId>,
        pub object_id: Option<ids::RemoteObject>,
        pub found_box: Option<BoxModel>,
}

#[derive(Debug)]
pub struct ResolveNode {
        pub common_fields: CommonDescribeFields,
        pub selector: Option<&'static str>,
        pub node_id: Option<dom::NodeId>,
        pub backend_node_id: Option<dom::NodeId>,
        pub object_group: Option<String>,
        pub execution_context_id: Option<String>,
}

#[derive(Debug)]
pub struct QuerySelector {
        pub common_fields: CommonDescribeFields,
        pub node_id: Option<dom::NodeId>,
        pub found_node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}

#[derive(Debug)]
pub struct GetDocument {
        pub common_fields: CommonDescribeFields,
        pub depth: Option<u8>,
        pub pierce: bool,
        pub root_node: Option<dom::Node>,
}

#[derive(Debug)]
pub struct DescribeNode {
        pub common_fields: CommonDescribeFields,
        pub node_id: Option<dom::NodeId>,
        pub backend_node_id: Option<dom::NodeId>,
        pub found_node: Option<dom::Node>,
        pub selector: Option<&'static str>,
        pub depth: Option<i8>,
        pub object_id: Option<ids::RemoteObject>,
        pub pierce: bool,
}

#[derive(Debug, Clone)]
pub struct CommonDescribeFields {
        pub task_id: ids::Task,
        pub target_id: target::TargetId,
        pub is_manual: bool,
        pub session_id: Option<SessionId>,
        pub call_id: usize,
}
