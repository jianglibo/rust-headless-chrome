use crate::protocol::{self, dom, page, target};
use super::id_type as ids;

#[derive(Debug)]
pub enum TaskDescribe {
    QuerySelector(QuerySelector),
    DescribeNode(DescribeNode),
    GetDocument(ids::Task)
}

#[derive(Debug)]
pub struct QuerySelector {
        pub task_id: usize,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}

#[derive(Debug)]
pub struct DescribeNode {
        pub task_id: usize,
        pub is_manual: bool,
        pub node_id: Option<dom::NodeId>,
        pub backend_node_id: Option<dom::NodeId>,
        pub selector: &'static str,
}