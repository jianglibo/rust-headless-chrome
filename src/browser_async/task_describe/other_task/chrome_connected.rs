use super::super::{TaskDescribe, CommonDescribeFields, TargetCallMethodTaskFace};
use crate::protocol::{target};


#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct ChromeConnectedTask {
}

// impl From<PageEnableTask> for TaskDescribe {
//     fn from(task: PageEnableTask) -> Self {
//         TaskDescribe::PageEnable(task)
//     }
// }

impl TargetCallMethodTaskFace for ChromeConnectedTask {
    fn get_session_id(&self) -> Option<&target::SessionID> {
        None
    }

    fn get_call_id(&self) -> usize {
        0
    }

    fn get_method_str(&self) -> String {
        self._empty_method_str("ChromeConnectedTask")
    }
}
