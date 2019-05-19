use super::super::{RuntimeEvent, TaskDescribe};
use crate::browser_async::embedded_events;
use crate::protocol::{network, page};

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
wrapper_raw_event!(
    TaskDescribe::RuntimeEvent,
    RuntimeEvent::ExecutionContextCreated,
    ExecutionContextCreated,
    embedded_events::ExecutionContextCreated
);
wrapper_raw_event!(
    TaskDescribe::RuntimeEvent,
    RuntimeEvent::ExecutionContextDestroyed,
    ExecutionContextDestroyed,
    embedded_events::ExecutionContextDestroyed
);

#[derive(Debug)]
pub struct ExecutionContextsCleared {}

#[derive(Debug)]
pub struct InspectRequested {}