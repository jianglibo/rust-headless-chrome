use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask, HasCommonField, CanCreateMethodString,};
use crate::protocol::{page};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct GetLayoutMetricsTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub task_result: Option<String>,
}

impl_has_common_fields!(GetLayoutMetricsTask);

impl AsMethodCallString for GetLayoutMetricsTask {
    fn get_method_str(&self) ->  Result<String, failure::Error>{
        let method = page::methods::GetLayoutMetrics {};
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetLayoutMetrics, GetLayoutMetricsTask);