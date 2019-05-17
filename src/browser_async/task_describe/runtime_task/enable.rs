use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace};
use crate::protocol::{runtime, target};

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct RuntimeEnableTask {
    pub common_fields: CommonDescribeFields,
}

impl TargetCallMethodTaskFace for RuntimeEnableTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        let method = runtime::methods::Enable {};
        self._to_method_str(method)
    }
}
