use super::RuntimeEvent;
use super::super::TaskDescribe;
use crate::browser_async::embedded_events;
use crate::protocol::{network, page, runtime};

#[derive(Debug)]
pub struct ExceptionRevoked {}

#[derive(Debug)]
pub struct ExceptionThrown {}

wrapper_raw_event!(
    TaskDescribe::RuntimeEvent,
    RuntimeEvent::ConsoleAPICalled,
    ConsoleAPICalled,
    embedded_events::ConsoleAPICalled
);

impl ConsoleAPICalled {
    pub fn into_raw_parameters(self) -> embedded_events::ConsoleAPICalledParams {
        self.raw_event.params
    }
}
wrapper_raw_event!(
    TaskDescribe::RuntimeEvent,
    RuntimeEvent::ExecutionContextCreated,
    ExecutionContextCreated,
    embedded_events::ExecutionContextCreated
);

impl ExecutionContextCreated {
    pub fn get_execution_context_description(&self) -> runtime::ExecutionContextDescription {
        self.raw_event.params.context.clone()
    }
}

wrapper_raw_event!(
    TaskDescribe::RuntimeEvent,
    RuntimeEvent::ExecutionContextDestroyed,
    ExecutionContextDestroyed,
    embedded_events::ExecutionContextDestroyed
);

impl ExecutionContextDestroyed {
    pub fn into_execution_context_id(self) -> runtime::ExecutionContextId {
        self.raw_event.params.execution_context_id
    }
}

#[derive(Debug)]
pub struct ExecutionContextsCleared {}

#[derive(Debug)]
pub struct InspectRequested {}