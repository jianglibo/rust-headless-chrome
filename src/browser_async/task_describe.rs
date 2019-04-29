use super::dev_tools_method_util::SessionId;
use super::id_type as ids;
use super::page_message::{ChangingFrame, PageEventName};
use super::unique_number;
use crate::browser::tab::element::BoxModel;
use crate::browser_async::dev_tools_method_util::{next_call_id, ChromePageError, MethodUtil};
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
                        TaskDescribe::GetBoxModel(get_box_model) => {
                                Some(&get_box_model.common_fields)
                        }
                        TaskDescribe::ScreenShot(screen_shot) => {
                                Some(&screen_shot.common_fields)
                        }
                        TaskDescribe::PageEnable(common_fields) => Some(&common_fields),
                        _ => {
                                error!("get_common_fields got queried. but it doesn't implement that.");
                                None
                        }
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
                        TaskDescribe::GetBoxModel(GetBoxModel {
                                node_id,
                                backend_node_id,
                                common_fields,
                                object_id,
                                ..
                        }) => {
                                let s = if let Some(sv) = object_id {
                                        Some(sv.as_str())
                                } else {
                                        None
                                };
                                Ok(MethodUtil::create_msg_to_send_with_session_id(
                                        dom::methods::GetBoxModel {
                                                node_id: *node_id,
                                                backend_node_id: *backend_node_id,
                                                object_id: s,
                                        },
                                        &common_fields.session_id,
                                        common_fields.call_id,
                                ))
                        }
                        TaskDescribe::ScreenShot(ScreenShot {
                                clip,
                                format,
                                common_fields,
                                from_surface,
                                ..
                        }) => {
                                let (format, quality) = match format {
                                        page::ScreenshotFormat::JPEG(quality) => {
                                                (page::InternalScreenshotFormat::JPEG, *quality)
                                        }
                                        page::ScreenshotFormat::PNG => {
                                                (page::InternalScreenshotFormat::PNG, None)
                                        }
                                };
                                Ok(MethodUtil::create_msg_to_send_with_session_id(
                                        page::methods::CaptureScreenshot {
                                                clip: clip.as_ref().cloned(),
                                                format,
                                                quality,
                                                from_surface: *from_surface,
                                        },
                                        &common_fields.session_id,
                                        common_fields.call_id,
                                ))
                        }
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

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct ScreenShot {
        pub common_fields: CommonDescribeFields,
        pub selector: Option<&'static str>,
        pub format: page::ScreenshotFormat,
        #[builder(default = "None")]
        pub clip: Option<page::Viewport>,
        #[builder(default = "false")]
        pub from_surface: bool,
        #[builder(default = "None")]
        pub base64: Option<String>,
}

impl From<ScreenShot> for TaskDescribe {
        fn from(screen_shot: ScreenShot) -> Self {
                TaskDescribe::ScreenShot(screen_shot)
        }
}

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct GetBoxModel {
        pub common_fields: CommonDescribeFields,
        #[builder(default = "None")]
        pub node_id: Option<dom::NodeId>,
        #[builder(default = "None")]
        pub selector: Option<&'static str>,
        #[builder(default = "None")]
        pub backend_node_id: Option<dom::NodeId>,
        #[builder(default = "None")]
        pub object_id: Option<ids::RemoteObject>,
        #[builder(setter(skip))]
        pub found_box: Option<BoxModel>,
}

impl From<GetBoxModel> for TaskDescribe {
        fn from(get_box_model: GetBoxModel) -> Self {
                TaskDescribe::GetBoxModel(get_box_model)
        }
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

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct QuerySelector {
        pub common_fields: CommonDescribeFields,
        #[builder(default = "None")]
        pub node_id: Option<dom::NodeId>,
        #[builder(default = "None")]
        pub found_node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}

impl From<QuerySelector> for TaskDescribe {
        fn from(query_selector: QuerySelector) -> Self {
                TaskDescribe::QuerySelector(query_selector)
        }
}

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct GetDocument {
        pub common_fields: CommonDescribeFields,
        #[builder(default = "Some(0)")]
        pub depth: Option<u8>,
        #[builder(default = "false")]
        pub pierce: bool,
        #[builder(setter(skip))]
        pub root_node: Option<dom::Node>,
}

impl From<GetDocument> for TaskDescribe {
        fn from(get_document: GetDocument) -> Self {
                TaskDescribe::GetDocument(get_document)
        }
}

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct DescribeNode {
        pub common_fields: CommonDescribeFields,
        #[builder(default = "None")]
        pub node_id: Option<dom::NodeId>,
        #[builder(default = "None")]
        pub backend_node_id: Option<dom::NodeId>,
        #[builder(default = "None")]
        pub found_node: Option<dom::Node>,
        pub selector: Option<&'static str>,
        #[builder(default = "Some(0)")]
        pub depth: Option<i8>,
        #[builder(default = "None")]
        pub object_id: Option<ids::RemoteObject>,
        #[builder(default = "false")]
        pub pierce: bool,
}

impl From<DescribeNode> for TaskDescribe {
        fn from(describe_node: DescribeNode) -> Self {
                TaskDescribe::DescribeNode(describe_node)
        }
}

#[derive(Debug, Clone, Default, Builder)]
#[builder(setter(into))]
pub struct CommonDescribeFields {
        pub task_id: ids::Task,
        pub target_id: target::TargetId,
        pub session_id: Option<SessionId>,
        #[builder(default = "next_call_id()")]
        pub call_id: usize,
}

pub fn get_common_fields_builder(
        target_id: target::TargetId,
        session_id: Option<SessionId>,
        task_id: Option<ids::Task>,
) -> CommonDescribeFieldsBuilder {
        let mut builder = CommonDescribeFieldsBuilder::default();
        let t_id = task_id.map_or_else(unique_number::create_one, |v| v);
        builder.target_id(target_id)
                .session_id(session_id)
                .task_id(t_id);
        builder
}
