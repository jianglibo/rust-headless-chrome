use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{page};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct PageReloadTask {
    pub common_fields: CommonDescribeFields,
    pub ignore_cache: bool,
    #[builder(default = "None")]
    pub script_to_evaluate: Option<String>,
}

impl_has_common_fields!(PageReloadTask, "PageReloadTask");

impl AsMethodCallString for PageReloadTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = page::methods::Reload{
            ignore_cache: self.ignore_cache,
            script_to_evaluate: self.script_to_evaluate.as_ref().map(String::as_str),
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::PageReload, PageReloadTask);