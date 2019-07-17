use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{runtime};
use failure;

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct GetPropertiesTask {
    pub common_fields: CommonDescribeFields,
    #[builder(default = "None")]
    pub object_id: Option<runtime::RemoteObjectId>,
    #[builder(default = "None")]
    pub own_properties: Option<bool>,
    #[builder(default = "None")]
    pub accessor_properties_only: Option<bool>,
    #[builder(default = "None")]
    pub generate_preview: Option<bool>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::GetPropertiesReturnObject>,
}

impl_has_common_fields!(GetPropertiesTask);

impl GetPropertiesTask {
    /// GetPropertiesReturnObject may contains fields other than array items. The array item's name property is a number. for example name: "0" etc.
    pub fn get_array_of_remote_object(&self) -> Vec<&runtime::RemoteObject> {
        if let Some(gro) = self.task_result.as_ref() {
            gro.result.iter().filter_map(|pd|  {
                if let Ok(_idx) = pd.name.parse::<usize>() {
                    pd.value.as_ref()
                } else {
                    None
                }
            }).collect()
        } else {
            Vec::new()
        }
    }

    pub fn get_array_of_remote_object_id(&self) -> Vec<&runtime::RemoteObjectId> {
        self.get_array_of_remote_object().iter().filter_map(|ro|ro.object_id.as_ref()).collect()
    }
}

impl AsMethodCallString for GetPropertiesTask {
    fn get_method_str(&self) -> Result<String, failure::Error> {
        let object_id = self.object_id.as_ref().cloned().expect("get_method_str for GetPropertiesTask need obejct_id property.");
        let method = runtime::methods::GetProperties {
                        object_id: object_id.as_str(),
                        own_properties: self.own_properties,
                        accessor_properties_only: self.accessor_properties_only,
                        generate_preview: self.generate_preview,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::GetProperties, GetPropertiesTask);