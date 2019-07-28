use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{network};
use failure;
use log::*;


#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct GetResponseBodyForInterceptionTask {
    pub common_fields: CommonDescribeFields,
    pub interception_id: String,
    #[builder(default = "None")]
    pub request_id: Option<network::RequestId>,
    #[builder(default = "None")]
    pub task_result: Option<network::methods::GetResponseBodyForInterceptionReturnObject>,
}

impl_has_common_fields!(GetResponseBodyForInterceptionTask, "GetResponseBodyForInterceptionTask");

impl AsMethodCallString for GetResponseBodyForInterceptionTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let method = network::methods::GetResponseBodyForInterception {
           interception_id: self.interception_id.as_str(),
        };
        Ok(self.create_method_str(method))
    }
}

impl GetResponseBodyForInterceptionTask {
    pub fn get_body_string(&self) -> Result<String, failure::Error> {
        if let Some(task_result) = &self.task_result {
            // let task_result = self.task_result.as_ref().expect("GetResponseBodyForInterceptionTask task_result should exists.");
            if task_result.base64_encoded {
                let v8 = base64::decode(&task_result.body)?;
                Ok(String::from_utf8(v8)?)
            } else {
                Ok(task_result.body.clone())
            }
        } else {
            error!("no task_result: {:?}", self);
            Ok(String::from(""))
        }
    }

    pub fn get_raw_response_string(&self) -> Option<String> {
        self.task_result.as_ref().cloned().map(|it|it.body.clone())
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetResponseBodyForInterception, GetResponseBodyForInterceptionTask);