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

#[derive(Debug, Builder, Default)]
#[builder(setter(into))]
pub struct GetBoxModel {
        pub common_fields: CommonDescribeFields,
        pub node_id: Option<dom::NodeId>,
        pub selector: Option<&'static str>,
        pub backend_node_id: Option<dom::NodeId>,
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

fn default_common_fields(
        target_id: target::TargetId,
        session_id: Option<SessionId>,
) -> CommonDescribeFields {
        CommonDescribeFields {
                target_id,
                task_id: 0,
                is_manual: false,
                session_id,
                call_id: next_call_id(),
        }
}

#[derive(Debug)]
pub struct PageEnableTaskBuilder {
        common_fields: CommonDescribeFields,
}

impl PageEnableTaskBuilder {
        pub fn new(target_id: target::TargetId, session_id: Option<SessionId>) -> Self {
                Self {
                        common_fields: default_common_fields(target_id, session_id),
                }
        }

        pub fn build(mut self) -> TaskDescribe {
                if self.common_fields.task_id == 0 {
                        self.common_fields.task_id = unique_number::create_one();
                }
                TaskDescribe::PageEnable(self.common_fields)
        }
}

#[derive(Debug, Default)]
pub struct QuerySelectorTaskBuilder {
        common_fields: CommonDescribeFields,
        node_id: Option<dom::NodeId>,
        found_node_id: Option<dom::NodeId>,
        selector: &'static str,
}

impl QuerySelectorTaskBuilder {
        pub fn new(
                target_id: target::TargetId,
                session_id: Option<SessionId>,
                selector: &'static str,
        ) -> Self {
                Self {
                        common_fields: default_common_fields(target_id, session_id),
                        selector,
                        ..Default::default()
                }
        }

        pub fn task_id_opt(mut self, task_id: Option<ids::Task>) -> Self {
                self.common_fields.task_id = task_id.map_or_else(unique_number::create_one, |v| v);
                self
        }

        pub fn build(mut self) -> TaskDescribe {
                if self.common_fields.task_id == 0 {
                        self.common_fields.task_id = unique_number::create_one();
                }
                let qs = QuerySelector {
                        common_fields: self.common_fields,
                        selector: self.selector,
                        node_id: self.node_id,
                        found_node_id: self.found_node_id,
                };
                TaskDescribe::QuerySelector(qs)
        }
}

#[derive(Debug, Default)]
pub struct GetDocumentTaskBuilder {
        common_fields: CommonDescribeFields,
        depth: Option<u8>,
        pierce: bool,
        root_node: Option<dom::Node>,
}

impl GetDocumentTaskBuilder {
        pub fn new(target_id: target::TargetId, session_id: Option<SessionId>) -> Self {
                Self {
                        common_fields: default_common_fields(target_id, session_id),
                        ..Default::default()
                }
        }

        pub fn task_id_opt(mut self, task_id: Option<ids::Task>) -> Self {
                self.common_fields.task_id = task_id.map_or_else(unique_number::create_one, |v| v);
                self
        }

        pub fn depth_opt(mut self, depth: Option<u8>) -> Self {
                self.depth = depth;
                self
        }

        pub fn depth(mut self, depth: u8) -> Self {
                self.depth = Some(depth);
                self
        }

        pub fn pierce(mut self, pierce: bool) -> Self {
                self.pierce = pierce;
                self
        }

        pub fn build(mut self) -> TaskDescribe {
                if self.common_fields.task_id == 0 {
                        self.common_fields.task_id = unique_number::create_one();
                }
                let gd = GetDocument {
                        common_fields: self.common_fields,
                        depth: self.depth.or(Some(0)),
                        pierce: self.pierce,
                        root_node: self.root_node,
                };
                TaskDescribe::GetDocument(gd)
        }
}

#[derive(Debug, Default)]
pub struct DescribeNodeTaskBuilder {
        common_fields: CommonDescribeFields,
        node_id: Option<dom::NodeId>,
        selector: &'static str,
        found_node: Option<dom::Node>,
        pub backend_node_id: Option<dom::NodeId>,
        pub depth: Option<i8>,
        pub object_id: Option<ids::RemoteObject>,
        pub pierce: bool,
}

impl DescribeNodeTaskBuilder {
        pub fn new(
                target_id: target::TargetId,
                session_id: Option<SessionId>,
                selector: &'static str,
        ) -> Self {
                Self {
                        common_fields: default_common_fields(target_id, session_id),
                        selector,
                        ..Default::default()
                }
        }

        pub fn task_id_opt(mut self, task_id: Option<ids::Task>) -> Self {
                self.common_fields.task_id = task_id.map_or_else(unique_number::create_one, |v| v);
                self
        }

        pub fn depth_opt(mut self, depth: Option<i8>) -> Self {
                self.depth = depth;
                self
        }

        pub fn build(mut self) -> TaskDescribe {
                if self.common_fields.task_id == 0 {
                        self.common_fields.task_id = unique_number::create_one();
                }
                let qs = DescribeNode {
                        common_fields: self.common_fields,
                        selector: Some(self.selector),
                        depth: self.depth.or(Some(0)),
                        ..Default::default()
                };
                TaskDescribe::DescribeNode(qs)
        }
}

#[derive(Debug, Default)]
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

#[derive(Debug, Clone, Default, Builder)]
#[builder(setter(into))]
pub struct CommonDescribeFields {
        pub task_id: ids::Task,
        pub target_id: target::TargetId,
        pub is_manual: bool,
        pub session_id: Option<SessionId>,
        #[builder(default="next_call_id()")]
        pub call_id: usize,
}

pub fn get_common_fields_builder(target_id: target::TargetId, session_id: Option<SessionId>, task_id: Option<ids::Task>) -> CommonDescribeFieldsBuilder {
        let mut builder = CommonDescribeFieldsBuilder::default();
        let t_id = task_id.map_or_else(unique_number::create_one, |v|v);
        builder.target_id(target_id).session_id(session_id).task_id(t_id);
        builder
}