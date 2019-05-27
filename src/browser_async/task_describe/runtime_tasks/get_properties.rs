use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{runtime, target};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeGetPropertiesTask {
    pub common_fields: CommonDescribeFields,
    pub object_id: runtime::RemoteObjectId,
    #[builder(default = "None")]
    pub own_properties: Option<bool>,
    #[builder(default = "None")]
    pub accessor_properties_only: Option<bool>,
    #[builder(default = "None")]
    pub generate_preview: Option<bool>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::GetPropertiesReturnObject>,
}

impl_has_common_fields!(RuntimeGetPropertiesTask);

impl AsMethodCallString for RuntimeGetPropertiesTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let method = runtime::methods::GetProperties {
                        object_id: self.object_id.as_str(),
                        own_properties: self.own_properties,
                        accessor_properties_only: self.accessor_properties_only,
                        generate_preview: self.generate_preview,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::RuntimeGetProperties, RuntimeGetPropertiesTask);