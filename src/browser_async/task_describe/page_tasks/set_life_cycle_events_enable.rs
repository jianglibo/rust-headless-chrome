use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString, };
use crate::protocol::{page};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct SetLifecycleEventsEnabledTask {
    pub common_fields: CommonDescribeFields,
    pub enabled: bool,
}

impl_has_common_fields!(SetLifecycleEventsEnabledTask, "SetLifecycleEventsEnabledTask");

impl AsMethodCallString for SetLifecycleEventsEnabledTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = page::methods::SetLifecycleEventsEnabled{
            enabled: self.enabled,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::SetLifecycleEventsEnabled, SetLifecycleEventsEnabledTask);