use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{page};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct PageCloseTask {
    pub common_fields: CommonDescribeFields,
}

impl_has_common_fields!(PageCloseTask, "PageCloseTask");

impl AsMethodCallString for PageCloseTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = page::methods::Close{};
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::PageClose, PageCloseTask);
