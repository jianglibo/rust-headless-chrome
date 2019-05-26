use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{network};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SetRequestInterceptionTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default)]
    pub patterns: Vec<network::methods::RequestPattern>,
}

impl SetRequestInterceptionTask {
    pub fn add_request_pattern(&mut self, url_pattern: Option<&str>, resource_type: Option<network::ResourceType>, interception_stage: Option<network::InterceptionStage>) {
        self.patterns.push(network::methods::RequestPattern {
            url_pattern: url_pattern.map(str::to_owned),
            resource_type,
            interception_stage,
        });
    }
}

impl_has_common_fields!(SetRequestInterceptionTask);

impl AsMethodCallString for SetRequestInterceptionTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = network::methods::SetRequestInterception {
            patterns: &self.patterns,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::SetRequestInterception, SetRequestInterceptionTask);
