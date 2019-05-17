use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace};
use crate::protocol::{target};


#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct FailTask {
    pub common_fields: CommonDescribeFields,
}

// impl From<PageEnableTask> for TaskDescribe {
//     fn from(task: PageEnableTask) -> Self {
//         TaskDescribe::PageEnable(task)
//     }
// }

impl TargetCallMethodTaskFace for FailTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        self.common_fields.session_id.as_ref()
    }

    fn get_call_id(&self) -> usize {
        self.common_fields.call_id
    }

    fn get_method_str(&self) -> String {
        self._empty_method_str("FailTask")
    }
}
