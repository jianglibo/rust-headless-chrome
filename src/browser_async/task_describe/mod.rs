// use super::target_message_event::target_message_events;
use super::super::protocol::{self, target};
use super::{create_msg_to_send_with_session_id, create_unique_task_id, next_call_id, TaskId};
use log::*;

pub mod browser_call_methods;
pub mod browser_tasks;
pub mod dom_tasks;
pub mod emulation_tasks;
pub mod input_tasks;
pub mod log_tasks;
pub mod network_tasks;
pub mod other_tasks;
pub mod page_tasks;
pub mod runtime_tasks;
pub mod security_tasks;
pub mod target_call_methods;
pub mod target_tasks;

pub use log_tasks::{handle_log_event, log_events, LogEnableTask, LogEnableTaskBuilder, LogEvent};

pub use browser_tasks::{GetBrowserCommandLineTask, GetBrowserCommandLineTaskBuilder};

pub use dom_tasks::{
    dom_events, handle_dom_event, DescribeNodeTask, DescribeNodeTaskBuilder, DomEvent,
    GetBoxModelTask, GetBoxModelTaskBuilder, GetDocumentTask, GetDocumentTaskBuilder,
    QuerySelectorTask, QuerySelectorTaskBuilder,
};
pub use page_tasks::{
    handle_page_event, page_events, CaptureScreenshotTask, CaptureScreenshotTaskBuilder,
    NavigateToTask, NavigateToTaskBuilder, PageEnableTask, PageEvent, PageReloadTask,
    PageReloadTaskBuilder, PrintToPdfTask, PrintToPdfTaskBuilder,
};
pub use runtime_tasks::{
    handle_runtime_event, runtime_events, CallFunctionOnTask, CallFunctionOnTaskBuilder,
    EvaluateTask, EvaluateTaskBuilder, GetPropertiesTask, GetPropertiesTaskBuilder,
    RuntimeEnableTask, RuntimeEnableTaskBuilder, RuntimeEvent,
};
pub use security_tasks::{
    SecurityEnableTask, SecurityEnableTaskBuilder, SetIgnoreCertificateErrorsTask,
    SetIgnoreCertificateErrorsTaskBuilder,
};

pub use target_tasks::{
    handle_target_event, target_events, ActivateTargetTask, ActivateTargetTaskBuilder,
    CreateTargetTask, CreateTargetTaskBuilder, GetTargetsTask, SetDiscoverTargetsTask,
    SetDiscoverTargetsTaskBuilder, TargetEvent,
};

pub use network_tasks::{
    handle_network_event, network_events, ContinueInterceptedRequestTask,
    ContinueInterceptedRequestTaskBuilder, GetResponseBodyForInterceptionTask,
    GetResponseBodyForInterceptionTaskBuilder, NetworkEnableTask, NetworkEnableTaskBuilder,
    NetworkEvent, SetRequestInterceptionTask, SetRequestInterceptionTaskBuilder,
};

pub use browser_call_methods::{handle_browser_method_call, BrowserCallMethodTask};
pub use target_call_methods::{handle_target_method_call, TargetCallMethodTask};

pub trait HasSessionId {
    fn set_session_id(&mut self, session_id: target::SessionID);
}

pub trait HasCallId {
    fn get_call_id(&self) -> protocol::CallId;
    fn renew_call_id(&mut self);
}

pub trait HasTaskId {
    fn get_task_id(&self) -> TaskId;
    fn task_id_equal(&self, pattern: &str) -> bool;
    fn task_id_starts_with(&self, pattern: &str) -> bool;
}

pub trait HasTaskName {
    fn get_task_name(&self) -> &str;
}

pub trait HasCommonField {
    const TASK_NAME: &'static str;
    fn get_common_fields(&self) -> &CommonDescribeFields;
    fn get_common_fields_mut(&mut self) -> &mut CommonDescribeFields;
    fn get_task_name(&self) -> &str {
        Self::TASK_NAME
    }
}

impl<T> HasCallId for T
where
    T: HasCommonField,
{
    fn get_call_id(&self) -> protocol::CallId {
        self.get_common_fields()
            .call_id
            .expect("call_id should exists when call get_call_id.")
    }

    fn renew_call_id(&mut self) {
        self.get_common_fields_mut().call_id.replace(next_call_id());
    }
}

impl<T> HasTaskId for T
where
    T: HasCommonField,
{
    fn get_task_id(&self) -> TaskId {
        self.get_common_fields().task_id.clone()
    }

    fn task_id_equal(&self, pattern: &str) -> bool {
        self.get_common_fields().task_id == pattern
    }

    fn task_id_starts_with(&self, pattern: &str) -> bool {
        self.get_common_fields().task_id.starts_with(pattern)
    }
    // fn set_task_id(&mut self) -> &mut Self {
    //     self
    // }
}

pub trait CanCreateMethodString {
    fn create_method_str<C>(&self, method: C) -> String
    where
        C: protocol::Method + serde::Serialize;
}

impl<T> CanCreateMethodString for T
where
    T: HasCommonField,
{
    fn create_method_str<C>(&self, method: C) -> String
    where
        C: protocol::Method + serde::Serialize,
    {
        create_msg_to_send_with_session_id(
            method,
            self.get_common_fields().session_id.as_ref(),
            self.get_common_fields()
                .call_id
                .expect("call_id should exists when call create_method_str."),
        )
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
    LogEvent(LogEvent),
    NetworkEvent(NetworkEvent),
    Interval,
    ChromeConnected,
}

impl_iro_iro_for_task_describe!(
    [
        TargetCallMethodTask::QuerySelector,
        TargetCallMethodTask::DescribeNode,
        TargetCallMethodTask::PrintToPDF,
        TargetCallMethodTask::GetBoxModel,
        TargetCallMethodTask::GetContentQuads,
        TargetCallMethodTask::CaptureScreenshot,
        TargetCallMethodTask::GetDocument,
        TargetCallMethodTask::NavigateTo,
        TargetCallMethodTask::PageEnable,
        TargetCallMethodTask::RuntimeEnable,
        TargetCallMethodTask::Evaluate,
        TargetCallMethodTask::GetProperties,
        TargetCallMethodTask::RuntimeCallFunctionOn,
        TargetCallMethodTask::NetworkEnable,
        TargetCallMethodTask::SetRequestInterception,
        TargetCallMethodTask::GetResponseBodyForInterception,
        TargetCallMethodTask::ContinueInterceptedRequest,
        TargetCallMethodTask::PageReload,
        TargetCallMethodTask::GetLayoutMetrics,
        TargetCallMethodTask::BringToFront,
        TargetCallMethodTask::PageClose,
        TargetCallMethodTask::DispatchMouseEvent,
        TargetCallMethodTask::CanEmulate,
        TargetCallMethodTask::SetDeviceMetricsOverride,
        TargetCallMethodTask::SetLifecycleEventsEnabled,
        TargetCallMethodTask::LogEnable
    ],
    [
        BrowserCallMethodTask::CreateTarget,
        BrowserCallMethodTask::SetDiscoverTargets,
        BrowserCallMethodTask::SetIgnoreCertificateErrors,
        BrowserCallMethodTask::SecurityEnable,
        BrowserCallMethodTask::AttachedToTarget,
        BrowserCallMethodTask::CloseTarget,
        BrowserCallMethodTask::ActivateTarget,
        BrowserCallMethodTask::GetTargets,
        BrowserCallMethodTask::GetBrowserCommandLine
    ]
);

// impl std::convert::TryFrom<&TaskDescribe> for String {
//     type Error = failure::Error;

//     fn try_from(task_describe: &TaskDescribe) -> Result<Self, Self::Error> {
//         match task_describe {
//             TaskDescribe::TargetCallMethod(target_call) => match target_call {
//                 TargetCallMethodTask::QuerySelector(task) => task.get_method_str(),
//                 TargetCallMethodTask::DescribeNode(task) => task.get_method_str(),
//                 TargetCallMethodTask::PrintToPDF(task) => task.get_method_str(),
//                 TargetCallMethodTask::GetBoxModel(task) => task.get_method_str(),
//                 TargetCallMethodTask::GetContentQuads(task) => task.get_method_str(),
//                 TargetCallMethodTask::CaptureScreenshot(task) => task.get_method_str(),
//                 TargetCallMethodTask::GetDocument(task) => task.get_method_str(),
//                 TargetCallMethodTask::NavigateTo(task) => task.get_method_str(),
//                 TargetCallMethodTask::PageEnable(task) => task.get_method_str(),
//                 TargetCallMethodTask::RuntimeEnable(task) => task.get_method_str(),
//                 TargetCallMethodTask::Evaluate(task) => task.get_method_str(),
//                 TargetCallMethodTask::GetProperties(task) => task.get_method_str(),
//                 TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_method_str(),
//                 TargetCallMethodTask::NetworkEnable(task) => task.get_method_str(),
//                 TargetCallMethodTask::SetRequestInterception(task) => task.get_method_str(),
//                 TargetCallMethodTask::GetResponseBodyForInterception(task) => task.get_method_str(),
//                 TargetCallMethodTask::ContinueInterceptedRequest(task) => task.get_method_str(),
//                 TargetCallMethodTask::PageReload(task) => task.get_method_str(),
//                 TargetCallMethodTask::GetLayoutMetrics(task) => task.get_method_str(),
//                 TargetCallMethodTask::BringToFront(task) => task.get_method_str(),
//                 TargetCallMethodTask::PageClose(task) => task.get_method_str(),
//                 TargetCallMethodTask::DispatchMouseEvent(task) => task.get_method_str(),
//                 TargetCallMethodTask::CanEmulate(task) => task.get_method_str(),
//                 TargetCallMethodTask::SetDeviceMetricsOverride(task) => task.get_method_str(),
//                 TargetCallMethodTask::SetLifecycleEventsEnabled(task) => task.get_method_str(),
//                 TargetCallMethodTask::LogEnable(task) => task.get_method_str(),
//             },
//             TaskDescribe::BrowserCallMethod(browser_call) => match browser_call {
//                 BrowserCallMethodTask::CreateTarget(task) => task.get_method_str(),
//                 BrowserCallMethodTask::SetDiscoverTargets(task) => task.get_method_str(),
//                 BrowserCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_method_str(),
//                 BrowserCallMethodTask::SecurityEnable(task) => task.get_method_str(),
//                 BrowserCallMethodTask::AttachedToTarget(task) => task.get_method_str(),
//                 BrowserCallMethodTask::CloseTarget(task) => task.get_method_str(),
//                 BrowserCallMethodTask::ActivateTarget(task) => task.get_method_str(),
//                 BrowserCallMethodTask::GetTargets(task) => task.get_method_str(),
//                 BrowserCallMethodTask::GetBrowserCommandLine(task) => task.get_method_str(),
//             },
//             _ => {
//                 error!("task describe to string failed. {:?}", task_describe);
//                 failure::bail!("should not be called.")
//             }
//         }
//     }
// }

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
    // #[builder(default = "next_call_id()")]
    // pub call_id: usize,
    #[builder(default = "None")]
    pub call_id: Option<usize>,
}

impl From<(Option<String>, Option<String>)> for CommonDescribeFields {
    fn from(session_id_target_id: (Option<String>, Option<String>)) -> Self {
        CommonDescribeFieldsBuilder::default()
            .target_id(session_id_target_id.1)
            .session_id(session_id_target_id.0.map(Into::into))
            .build()
            .expect("tuple to common_fields should work.")
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

    impl<T> TraitSum for T
    where
        T: TraitOne + TraitTwo,
    {
        fn sum(&self) -> usize {
            self.get_one() + self.get_two()
        }
    }

    #[test]
    fn enum_and_trait() {
        let v = WithOneTwo { one: 5, two: 10 };
        assert_eq!(v.sum(), 15);
    }

}
