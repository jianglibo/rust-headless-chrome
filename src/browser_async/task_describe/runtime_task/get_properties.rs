use super::super::{TaskDescribe, CommonDescribeFields, CreateMethodCallString, create_msg_to_send_with_session_id};
use crate::protocol::{runtime};
use crate::browser::transport::{SessionId};

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

impl From<RuntimeGetPropertiesTask> for TaskDescribe {
    fn from(get_properties: RuntimeGetPropertiesTask) -> Self {
        TaskDescribe::RuntimeGetProperties(Box::new(get_properties))
    }
}


impl CreateMethodCallString for RuntimeGetPropertiesTask {
    fn create_method_call_string(&self, session_id: Option<&SessionId>, call_id: usize) -> String {
        let method = runtime::methods::GetProperties {
                        object_id: self.object_id.as_str(),
                        own_properties: self.own_properties,
                        accessor_properties_only: self.accessor_properties_only,
                        generate_preview: self.generate_preview,
        };
                create_msg_to_send_with_session_id(
                    method,
                    session_id,
                    call_id,
                )
    }
}