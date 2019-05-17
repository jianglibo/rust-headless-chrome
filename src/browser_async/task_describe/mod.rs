use super::inner_event::inner_events;
use crate::browser_async::{
    create_msg_to_send_with_session_id,create_msg_to_send, MethodDestination, create_unique_usize, next_call_id, TaskId,
};
use crate::protocol::{self, dom, page, runtime, target};
use log::*;

pub mod dom_task;
pub mod other_task;
pub mod page_task;
pub mod runtime_task;
pub mod target_task;

pub use dom_task::{
    DescribeNodeTask, DescribeNodeTaskBuilder, GetBoxModelTask, GetBoxModelTaskBuilder,
    GetDocumentTask, GetDocumentTaskBuilder, QuerySelectorTask, QuerySelectorTaskBuilder,
};
pub use page_task::{
    CaptureScreenshotTask, CaptureScreenshotTaskBuilder, NavigateToTask, NavigateToTaskBuilder,
    PageEnableTask, PrintToPdfTask, PrintToPdfTaskBuilder, page_event,
};
pub use runtime_task::{
    RuntimeCallFunctionOnTask, RuntimeCallFunctionOnTaskBuilder, RuntimeEnableTask,
    RuntimeEnableTaskBuilder, RuntimeEvaluateTask, RuntimeEvaluateTaskBuilder,
    RuntimeGetPropertiesTask, RuntimeGetPropertiesTaskBuilder, runtime_event,
};

pub use other_task::{
    ChromeConnectedTask, ChromeConnectedTaskBuilder, FailTask, FailTaskBuilder, IntervalTask,
    IntervalTaskBuilder,
};

pub use target_task::{
    CreateTargetTask, CreateTargetTaskBuilder, SetDiscoverTargetsTask,
    SetDiscoverTargetsTaskBuilder, target_event,
};


pub trait TargetCallMethodTaskFace {
    fn get_session_id(&self) -> Option<&target::SessionID>;
    fn get_call_id(&self) -> usize;
    fn get_method_str(&self) -> String;

    fn _to_method_str<C>(&self, method: C) -> String
    where
        C: protocol::Method + serde::Serialize,
    {
        create_msg_to_send_with_session_id(method, self.get_session_id(), self.get_call_id())
    }

    fn _empty_method_str(&self, tip: &str) -> String {
        warn!("be called unexpectedly. {:?}", tip);
        String::from("")
    }    
}

pub trait BrowserCallMethodTaskFace {
    fn get_call_id(&self) -> usize;
    fn get_method_str(&self) -> String;

    fn _to_method_str<C>(&self, method: C) -> String
    where
        C: protocol::Method + serde::Serialize,
    {
        create_msg_to_send(method, MethodDestination::Browser, self.get_call_id())
    }

    fn _empty_method_str(&self, tip: &str) -> String {
        warn!("be called unexpectedly. {:?}", tip);
        String::from("")
    }    
}

pub trait HasCallId {
    fn get_call_id(&self) -> usize;
}

#[derive(Debug)]
pub enum BrowserCallMethodTask {
    CreateTarget(CreateTargetTask),
}

impl HasCallId for BrowserCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            BrowserCallMethodTask::CreateTarget(task) => task.get_call_id(),
        }
    }
}

#[derive(Debug)]
pub enum TargetCallMethodTask {
    NavigateTo(NavigateToTask),
    QuerySelector(QuerySelectorTask),
    DescribeNode(DescribeNodeTask),
    PrintToPDF(PrintToPdfTask),
    GetBoxModel(GetBoxModelTask),
    GetDocument(GetDocumentTask),
    PageEnable(PageEnableTask),
    RuntimeEnable(RuntimeEnableTask),
    CaptureScreenshot(CaptureScreenshotTask),
    TargetSetDiscoverTargets(SetDiscoverTargetsTask),
    RuntimeEvaluate(RuntimeEvaluateTask),
    RuntimeGetProperties(RuntimeGetPropertiesTask),
    RuntimeCallFunctionOn(RuntimeCallFunctionOnTask),
}

impl HasCallId for TargetCallMethodTask {
    fn get_call_id(&self) -> usize {
        match self {
            TargetCallMethodTask::NavigateTo(task) => task.get_call_id(),
            TargetCallMethodTask::QuerySelector(task) => task.get_call_id(),
            TargetCallMethodTask::DescribeNode(task) => task.get_call_id(),
            TargetCallMethodTask::PrintToPDF(task) => task.get_call_id(),
            TargetCallMethodTask::GetBoxModel(task) => task.get_call_id(),
            TargetCallMethodTask::GetDocument(task) => task.get_call_id(),
            TargetCallMethodTask::PageEnable(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeEnable(task) => task.get_call_id(),
            TargetCallMethodTask::CaptureScreenshot(task) => task.get_call_id(),
            TargetCallMethodTask::TargetSetDiscoverTargets(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeEvaluate(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeGetProperties(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_call_id(),
        }
    }
}

#[derive(Debug)]
pub enum PageEvent {
    DomContentEventFired(page_event::DomContentEventFired),
    FrameAttached(page_event::FrameAttached),
    FrameDetached(page_event::FrameDetached),
    FrameNavigated(page_event::FrameNavigated),
    FrameStartedLoading(page_event::FrameStartedLoading),
    FrameStoppedLoading(page_event::FrameStoppedLoading),
    LoadEventFired(page_event::LoadEventFired),
    PageCreated(page_event::PageCreated),
}

#[derive(Debug)]
pub enum RuntimeEvent {
    ConsoleAPICalled(runtime_event::ConsoleAPICalled),
    ExceptionRevoked(runtime_event::ExceptionRevoked),
    ExceptionThrown(runtime_event::ExceptionThrown),
    ExecutionContextCreated(runtime_event::ExecutionContextCreated),
    ExecutionContextDestroyed(runtime_event::ExecutionContextDestroyed),
    ExecutionContextsCleared(runtime_event::ExecutionContextsCleared),
    InspectRequested(runtime_event::InspectRequested),
}

#[derive(Debug)]
pub enum TargetEvent {
    ReceivedMessageFromTarget(target_event::ReceivedMessageFromTarget),
    TargetCreated(target_event::TargetCreated),
    TargetCrashed(target_event::TargetCrashed),
    TargetInfoChanged(target_event::TargetInfoChanged),
}

#[derive(Debug)]
pub enum TaskDescribe {
    TargetCallMethod(TargetCallMethodTask),
    BrowserCallMethod(BrowserCallMethodTask),
    PageEvent(PageEvent),
    RuntimeEvent(RuntimeEvent),
    TargetEvent(TargetEvent),
}

// #[derive(Debug)]
// pub enum TaskDescribe {
//     NavigateTo(NavigateToTask),
//     QuerySelector(QuerySelectorTask),
//     DescribeNode(DescribeNodeTask),
//     // ResolveNode(ResolveNode),
//     PrintToPDF(PrintToPdfTask),
//     GetBoxModel(GetBoxModelTask),
//     SetChildNodes(target::TargetId, dom::NodeId, Vec<dom::Node>),
//     GetDocument(GetDocumentTask),
//     PageEnable(PageEnableTask),
//     RuntimeEnable(RuntimeEnableTask),
//     Interval(IntervalTask),
//     FrameAttached(page::events::FrameAttachedParams, CommonDescribeFields),
//     FrameDetached(page::types::FrameId, CommonDescribeFields),
//     FrameStartedLoading(String, CommonDescribeFields),
//     FrameNavigated(Box<page::Frame>, CommonDescribeFields),
//     FrameStoppedLoading(String, CommonDescribeFields),
//     LoadEventFired(target::TargetId, f32),
//     TargetInfoChanged(target::TargetInfo, CommonDescribeFields),
//     PageCreated(target::TargetInfo, Option<String>),
//     PageAttached(target::TargetInfo, target::SessionID),
//     CaptureScreenshot(CaptureScreenshotTask),
//     TargetSetDiscoverTargets(SetDiscoverTargetsTask),
//     ChromeConnected(ChromeConnectedTask),
//     Fail(FailTask),
//     RuntimeEvaluate(RuntimeEvaluateTask),
//     RuntimeGetProperties(RuntimeGetPropertiesTask),
//     RuntimeExecutionContextCreated(
//         runtime::types::ExecutionContextDescription,
//         CommonDescribeFields,
//     ),
//     RuntimeExecutionContextDestroyed(runtime::types::ExecutionContextId, CommonDescribeFields),
//     RuntimeConsoleAPICalled(inner_events::ConsoleAPICalledParams, CommonDescribeFields),
//     RuntimeCallFunctionOn(RuntimeCallFunctionOnTask),
//     CreateTarget(CreateTargetTask),
// }

impl std::convert::From<&TaskDescribe> for String {
    fn from(task_describe: &TaskDescribe) -> Self {
        match task_describe {
            TaskDescribe::TargetCallMethod(target_call) => match target_call {
                TargetCallMethodTask::QuerySelector(query_selector) => query_selector.get_method_str(),
                TargetCallMethodTask::DescribeNode(describe_node) => describe_node.get_method_str(),
                TargetCallMethodTask::PrintToPDF(print_to_pdf) => print_to_pdf.get_method_str(),
                TargetCallMethodTask::GetBoxModel(get_box_model) => get_box_model.get_method_str(),
                TargetCallMethodTask::CaptureScreenshot(capture_screenshot) => capture_screenshot.get_method_str(),
                TargetCallMethodTask::GetDocument(get_document) => get_document.get_method_str(),
                TargetCallMethodTask::NavigateTo(navigate_to) => navigate_to.get_method_str(),
                TargetCallMethodTask::PageEnable(task) => task.get_method_str(),
                TargetCallMethodTask::RuntimeEnable(task) => task.get_method_str(),
                TargetCallMethodTask::TargetSetDiscoverTargets(task) => task.get_method_str(),
                TargetCallMethodTask::RuntimeEvaluate(task) => task.get_method_str(),
                TargetCallMethodTask::RuntimeGetProperties(task) => task.get_method_str(),
                TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_method_str(),
            }
            TaskDescribe::BrowserCallMethod(browser_call) => match browser_call {
                BrowserCallMethodTask::CreateTarget(task) => task.get_method_str(),
            }
            _ => {
                error!("task describe to string failed. {:?}", task_describe);
                "should not be called.".into()
            }
        }
    }
}

// #[derive(Debug)]
// pub struct ResolveNode {
//     pub common_fields: CommonDescribeFields,
//     pub selector: Option<&'static str>,
//     pub node_id: Option<dom::NodeId>,
//     pub backend_node_id: Option<dom::NodeId>,
//     pub object_group: Option<String>,
//     pub execution_context_id: Option<String>,
// }

#[derive(Debug, Clone, Default, Builder)]
#[builder(setter(into))]
pub struct CommonDescribeFields {
    #[builder(default = "None")]
    pub target_id: Option<target::TargetId>,
    #[builder(default = "None")]
    pub session_id: Option<target::SessionID>,
    #[builder(default = "create_unique_usize()")]
    #[builder(setter(prefix = "_abc"))]
    pub task_id: TaskId,
    #[builder(default = "next_call_id()")]
    pub call_id: usize,
}

impl From<(Option<String>, Option<String>)> for CommonDescribeFields {
    fn from(session_id_target_id: (Option<String>, Option<String>)) -> Self {
        CommonDescribeFieldsBuilder::default()
            .target_id(session_id_target_id.1)
            .session_id(session_id_target_id.0.map(Into::into))
            .build()
            .unwrap()
    }
}

impl CommonDescribeFieldsBuilder {
    pub fn task_id(&mut self, task_id: impl Into<Option<TaskId>>) -> &mut Self {
        let o = task_id.into();
        if o.is_some() {
            self.task_id = o;
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;

    trait Traited1 { 
        fn get_call_id(&self) -> usize;
    }

    enum A1 {
        A11
    }

    enum A2 {
        A22
    }

    enum A12 {
        A1(A1),
        A2(A2),
    }

    #[test]
    fn enum_and_trait() {
        assert!(true);
        let v = A12::A1(A1::A11);
    }

}
