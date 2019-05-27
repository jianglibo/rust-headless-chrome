// use super::target_message_event::target_message_events;
use crate::browser_async::{
    create_msg_to_send_with_session_id, next_call_id, TaskId, create_unique_task_id,
};
use crate::protocol::{self, target};
use log::*;

pub mod dom_tasks;
pub mod other_tasks;
pub mod page_tasks;
pub mod runtime_tasks;
pub mod target_tasks;
pub mod security_tasks;
pub mod target_call_methods;
pub mod browser_call_methods;
pub mod network_tasks;

pub use dom_tasks::{
    DescribeNodeTask, DescribeNodeTaskBuilder, GetBoxModelTask, GetBoxModelTaskBuilder,
    GetDocumentTask, GetDocumentTaskBuilder, QuerySelectorTask, QuerySelectorTaskBuilder, dom_events, DomEvent,
};
pub use page_tasks::{
    CaptureScreenshotTask, CaptureScreenshotTaskBuilder, NavigateToTask, NavigateToTaskBuilder,
    PageEnableTask, PrintToPdfTask, PrintToPdfTaskBuilder, page_events, PageEvent,
};
pub use runtime_tasks::{
    RuntimeCallFunctionOnTask, RuntimeCallFunctionOnTaskBuilder, RuntimeEnableTask,
    RuntimeEnableTaskBuilder, RuntimeEvaluateTask, RuntimeEvaluateTaskBuilder,
    RuntimeGetPropertiesTask, RuntimeGetPropertiesTaskBuilder, runtime_events, RuntimeEvent,
};
pub use security_tasks::{
    SecurityEnableTask, SecurityEnableTaskBuilder,
    SetIgnoreCertificateErrorsTask, SetIgnoreCertificateErrorsTaskBuilder,
};

pub use target_tasks::{
    CreateTargetTask, CreateTargetTaskBuilder, SetDiscoverTargetsTask,
    SetDiscoverTargetsTaskBuilder, target_events, TargetEvent,
};

pub use network_tasks::{NetworkEnableTask, NetworkEnableTaskBuilder, network_events,
NetworkEvent, SetRequestInterceptionTask, SetRequestInterceptionTaskBuilder, handle_network_event,
 ContinueInterceptedRequestTask, ContinueInterceptedRequestTaskBuilder, GetResponseBodyForInterceptionTask, GetResponseBodyForInterceptionTaskBuilder,};

pub use target_call_methods::{TargetCallMethodTask, handle_target_method_call};
pub use browser_call_methods::{BrowserCallMethodTask, handle_browser_method_call};

pub trait HasSessionId {
    fn get_session_id(&self) -> target::SessionID;
}

pub trait HasCallId {
    fn get_call_id(&self) -> protocol::CallId;
}

pub trait HasTaskId {
    fn get_task_id(&self) -> TaskId;
    // fn set_task_id(&mut self) -> &mut Self;
}

pub trait HasCommonField {
    fn get_common_fields(&self) -> &CommonDescribeFields;
}

impl<T> HasCallId for T where T: HasCommonField {
    fn get_call_id(&self) -> protocol::CallId {
        self.get_common_fields().call_id
    }
}

impl<T> HasTaskId for T where T: HasCommonField {
    fn get_task_id(&self) -> TaskId {
        self.get_common_fields().task_id.clone()
    }
    // fn set_task_id(&mut self) -> &mut Self {
    //     self
    // }
}



pub trait CanCreateMethodString {
    fn create_method_str<C>(&self, method: C) -> String where
        C: protocol::Method + serde::Serialize,;
}

impl<T> CanCreateMethodString for T where T: HasCommonField {
    fn create_method_str<C>(&self, method: C) -> String where
        C: protocol::Method + serde::Serialize, {
            create_msg_to_send_with_session_id(method, self.get_common_fields().session_id.as_ref(), self.get_common_fields().call_id)
        }
}

pub trait AsMethodCallString {
    fn get_method_str(&self) -> Result<String, failure::Error>;

    fn _empty_method_str(&self, tip: &str) -> String {
        warn!("be called unexpectedly. {:?}", tip);
        String::from("")
    }    
}

#[derive(Debug)]
pub enum TaskDescribe {
    TargetCallMethod(TargetCallMethodTask),
    BrowserCallMethod(BrowserCallMethodTask),
    PageEvent(PageEvent),
    RuntimeEvent(RuntimeEvent),
    TargetEvent(TargetEvent),
    DomEvent(DomEvent),
    NetworkEvent(NetworkEvent),
    Interval,
    ChromeConnected,
}

impl std::convert::TryFrom<&TaskDescribe> for String {
    type Error = failure::Error;

    fn try_from(task_describe: &TaskDescribe) -> Result<Self, Self::Error> {
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
                TargetCallMethodTask::RuntimeEvaluate(task) => task.get_method_str(),
                TargetCallMethodTask::RuntimeGetProperties(task) => task.get_method_str(),
                TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_method_str(),
                TargetCallMethodTask::NetworkEnable(task) => task.get_method_str(),
                TargetCallMethodTask::SetRequestInterception(task) => task.get_method_str(),
                TargetCallMethodTask::GetResponseBodyForInterception(task) => task.get_method_str(),
                TargetCallMethodTask::ContinueInterceptedRequest(task) => task.get_method_str(),
            }
            TaskDescribe::BrowserCallMethod(browser_call) => match browser_call {
                BrowserCallMethodTask::CreateTarget(task) => task.get_method_str(),
                BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_method_str(),
                BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_method_str(),
                BrowserCallMethodTask::SecurityEnable(task) => task.get_method_str(),
            }
            _ => {
                error!("task describe to string failed. {:?}", task_describe);
                failure::bail!("should not be called.")
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
    #[builder(default = "create_unique_task_id()")]
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
