use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{network};
use failure;


#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct GetResponseBodyForInterceptionTask {
    pub common_fields: CommonDescribeFields,
    pub interception_id: String,
    #[builder(default = "None")]
    pub task_result: Option<network::methods::GetResponseBodyForInterceptionReturnObject>,
}

impl_has_common_fields!(GetResponseBodyForInterceptionTask);

impl AsMethodCallString for GetResponseBodyForInterceptionTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = network::methods::GetResponseBodyForInterception {
           interception_id: self.interception_id.as_str(),
        };
        Ok(self.create_method_str(method))
    }
}
impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetResponseBodyForInterception, GetResponseBodyForInterceptionTask);