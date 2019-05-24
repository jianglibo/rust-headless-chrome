use super::{
    DescribeNodeTask, DescribeNodeTaskBuilder, GetBoxModelTask, GetBoxModelTaskBuilder,
    GetDocumentTask, GetDocumentTaskBuilder, QuerySelectorTask, QuerySelectorTaskBuilder, dom_events,
};
use super::{
    CaptureScreenshotTask, CaptureScreenshotTaskBuilder, NavigateToTask, NavigateToTaskBuilder,
    PageEnableTask, PrintToPdfTask, PrintToPdfTaskBuilder, page_events,
};
use super::{
    RuntimeCallFunctionOnTask, RuntimeCallFunctionOnTaskBuilder, RuntimeEnableTask,
    RuntimeEnableTaskBuilder, RuntimeEvaluateTask, RuntimeEvaluateTaskBuilder,
    RuntimeGetPropertiesTask, RuntimeGetPropertiesTaskBuilder, runtime_events,
};
use super::{
    SecurityEnableTask, SecurityEnableTaskBuilder,
    SetIgnoreCertificateErrorsTask, SetIgnoreCertificateErrorsTaskBuilder,
};

use super::{
    CreateTargetTask, CreateTargetTaskBuilder, SetDiscoverTargetsTask,
    SetDiscoverTargetsTaskBuilder, target_events,
};
use super::{HasCallId};

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
    RuntimeEvaluate(RuntimeEvaluateTask),
    RuntimeGetProperties(RuntimeGetPropertiesTask),
    RuntimeCallFunctionOn(RuntimeCallFunctionOnTask),
    SecurityEnable(SecurityEnableTask),
    SetIgnoreCertificateErrors(SetIgnoreCertificateErrorsTask),
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
            TargetCallMethodTask::RuntimeEvaluate(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeGetProperties(task) => task.get_call_id(),
            TargetCallMethodTask::RuntimeCallFunctionOn(task) => task.get_call_id(),
            TargetCallMethodTask::SecurityEnable(task) => task.get_call_id(),
            TargetCallMethodTask::SetIgnoreCertificateErrors(task) => task.get_call_id(),
        }
    }
}