use crate::protocol::{self, dom, page, target};
use super::id_type as ids;
use super::page_message::{PageEventName, ChangingFrame};
use super::dev_tools_method_util::{SessionId};

#[derive(Debug)]
pub enum TaskDescribe {
    QuerySelector(QuerySelector),
    DescribeNode(DescribeNode),
    SetChildNodes(target::TargetId, dom::NodeId, Vec<dom::Node>),
    GetDocument(ids::Task, Option<target::TargetId>, Option<dom::Node>),
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
pub struct QuerySelector {
        pub task_id: usize,
        pub session_id: Option<SessionId>,
        pub target_id: target::TargetId,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub found_node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}

#[derive(Debug)]
pub struct DescribeNode {
        pub task_id: usize,
        pub session_id: Option<SessionId>,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub backend_node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}