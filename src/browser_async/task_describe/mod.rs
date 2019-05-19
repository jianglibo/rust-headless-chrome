// use super::target_message_event::target_message_events;
use crate::browser_async::{
    create_msg_to_send_with_session_id,create_msg_to_send, MethodDestination, create_unique_usize, next_call_id, TaskId,
};
use crate::protocol::{self, target};
use log::*;

pub mod dom_task;
pub mod other_task;
pub mod page_task;
pub mod runtime_task;
pub mod target_task;

pub use dom_task::{
    DescribeNodeTask, DescribeNodeTaskBuilder, GetBoxModelTask, GetBoxModelTaskBuilder,
    GetDocumentTask, GetDocumentTaskBuilder, QuerySelectorTask, QuerySelectorTaskBuilder, dom_events,
};
pub use page_task::{
    CaptureScreenshotTask, CaptureScreenshotTaskBuilder, NavigateToTask, NavigateToTaskBuilder,
    PageEnableTask, PrintToPdfTask, PrintToPdfTaskBuilder, page_events,
};
pub use runtime_task::{
    RuntimeCallFunctionOnTask, RuntimeCallFunctionOnTaskBuilder, RuntimeEnableTask,
    RuntimeEnableTaskBuilder, RuntimeEvaluateTask, RuntimeEvaluateTaskBuilder,
    RuntimeGetPropertiesTask, RuntimeGetPropertiesTaskBuilder, runtime_events,
};

pub use target_task::{
    CreateTargetTask, CreateTargetTaskBuilder, SetDiscoverTargetsTask,
    SetDiscoverTargetsTaskBuilder, target_events,
};

pub trait HasSessionId {
    fn get_session_id(&self) -> target::SessionID;
}

pub trait HasCallId {
    fn get_call_id(&self) -> protocol::CallId;
}

pub trait HasTaskId {
    fn get_task_id(&self) -> TaskId;
}

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
    DomContentEventFired(page_events::DomContentEventFired),
    FrameAttached(page_events::FrameAttached),
    FrameDetached(page_events::FrameDetached),
    FrameNavigated(page_events::FrameNavigated),
    FrameStartedLoading(page_events::FrameStartedLoading),
    FrameStoppedLoading(page_events::FrameStoppedLoading),
    LoadEventFired(page_events::LoadEventFired),
    // PageCreated(page_events::PageCreated),
}

#[derive(Debug)]
pub enum RuntimeEvent {
    ConsoleAPICalled(runtime_events::ConsoleAPICalled),
    ExceptionRevoked(runtime_events::ExceptionRevoked),
    ExceptionThrown(runtime_events::ExceptionThrown),
    ExecutionContextCreated(runtime_events::ExecutionContextCreated),
    ExecutionContextDestroyed(runtime_events::ExecutionContextDestroyed),
    ExecutionContextsCleared(runtime_events::ExecutionContextsCleared),
    InspectRequested(runtime_events::InspectRequested),
}

#[derive(Debug)]
pub enum DomEvent {
    AttributeModified(dom_events::AttributeModified),
    AttributeRemoved(dom_events::AttributeRemoved),
    CharacterDataModified(dom_events::CharacterDataModified),
    ChildNodeCountUpdated(dom_events::ChildNodeCountUpdated),
    ChildNodeInserted(dom_events::ChildNodeInserted),
    ChildNodeRemoved(dom_events::ChildNodeRemoved),
    DocumentUpdated(dom_events::DocumentUpdated),
    SetChildNodes(dom_events::SetChildNodes),
}

#[derive(Debug)]
pub enum TargetEvent {
    ReceivedMessageFromTarget(target_events::ReceivedMessageFromTarget),
    TargetCreated(target_events::TargetCreated),
    TargetCrashed(target_events::TargetCrashed),
    TargetInfoChanged(target_events::TargetInfoChanged),
    AttachedToTarget(target_events::AttachedToTarget),
}

#[derive(Debug)]
pub enum TaskDescribe {
    TargetCallMethod(TargetCallMethodTask),
    BrowserCallMethod(BrowserCallMethodTask),
    PageEvent(PageEvent),
    RuntimeEvent(RuntimeEvent),
    TargetEvent(TargetEvent),
    DomEvent(DomEvent),
    Interval,
    ChromeConnected,
}

impl std::convert::From<&TaskDescribe> for String {
    fn from(task_describe: &TaskDescribe) -> Self {
        match task_describe {
            TaskDescribe::TargetCallMethod(target_call) => match target_call {
                TargetCallMethodTask::QuerySelector(task) => task.get_method_str(),
                TargetCallMethodTask::DescribeNode(task) => task.get_method_str(),
                TargetCallMethodTask::PrintToPDF(task) => task.get_method_str(),
                TargetCallMethodTask::GetBoxModel(task) => task.get_method_str(),
                TargetCallMethodTask::CaptureScreenshot(task) => task.get_method_str(),
                TargetCallMethodTask::GetDocument(task) => task.get_method_str(),
                TargetCallMethodTask::NavigateTo(task) => task.get_method_str(),
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


// https://doc.rust-lang.org/reference/macros-by-example.html
// macro_rules! example {
//     ($(I $i:ident)* E $e:expr) => { ($($i)-*) * $e };
// }
// let foo = 2;
// let bar = 3;
// // The following expands to `(foo - bar) * 5`
// example!(I foo I bar E 5);

#[cfg(test)]
mod tests {
    trait TraitOne { 
        fn get_one(&self) -> usize;
    }

    trait TraitTwo {
        fn get_two(&self) -> usize;
    }

    trait TraitSum {
        fn sum(&self) -> usize;
    }

    struct WithOneTwo {
        pub one: usize,
        pub two: usize,
    }

    impl TraitOne for WithOneTwo {
        fn get_one(&self) -> usize {
            self.one
        }
    }

    impl TraitTwo for WithOneTwo {
        fn get_two(&self) -> usize {
            self.two
        }
    }

    impl<T> TraitSum for T where T : TraitOne + TraitTwo {
        fn sum(&self) -> usize {
            self.get_one() + self.get_two()
        }
    }


    #[test]
    fn enum_and_trait() {
        let v = WithOneTwo{one: 5, two: 10};
        assert_eq!(v.sum(), 15);
    }

}
