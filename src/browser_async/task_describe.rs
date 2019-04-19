use crate::protocol::{dom, target};
use super::id_type as ids;
use super::page_message::{PageEventName, ChangingFrame};
use super::dev_tools_method_util::{SessionId};
// use super::element_async::{BoxModel};

#[derive(Debug)]
pub enum TaskDescribe {
    QuerySelector(QuerySelector),
    DescribeNode(DescribeNode),
    ResolveNode(ResolveNode),
    GetBoxModel(GetBoxModel),
    SetChildNodes(target::TargetId, dom::NodeId, Vec<dom::Node>),
    GetDocument(ids::Task, target::TargetId, Option<dom::Node>),
    PageEnable(ids::Task, target::TargetId, SessionId),
    Interval,
    PageEvent(PageEventName),
    FrameNavigated(target::TargetId, ChangingFrame),
    TargetInfoChanged(target::TargetInfo),
    PageCreated(target::TargetInfo, Option<&'static str>),
    PageAttached(target::TargetInfo, SessionId),
    Fail,
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
        pub found_box: Option<dom::methods::BoxModel>
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