use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace};
use crate::protocol::{runtime, target};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeGetPropertiesTask {
    pub common_fields: CommonDescribeFields,
    pub object_id: runtime::types::RemoteObjectId,
    #[builder(default = "None")]
    pub own_properties: Option<bool>,
    #[builder(default = "None")]
    pub accessor_properties_only: Option<bool>,
    #[builder(default = "None")]
    pub generate_preview: Option<bool>,
    #[builder(default = "None")]
    pub task_result: Option<runtime::methods::GetPropertiesReturnObject>,
}

// impl From<RuntimeGetPropertiesTask> for TaskDescribe {
//     fn from(get_properties: RuntimeGetPropertiesTask) -> Self {
//         TaskDescribe::RuntimeGetProperties(Box::new(get_properties))
//     }
// }

impl TargetCallMethodTaskFace for RuntimeGetPropertiesTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = runtime::methods::GetProperties {
                        object_id: self.object_id.as_str(),
                        own_properties: self.own_properties,
                        accessor_properties_only: self.accessor_properties_only,
                        generate_preview: self.generate_preview,
        };
        self._to_method_str(method)
    }
}