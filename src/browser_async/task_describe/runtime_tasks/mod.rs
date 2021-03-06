// place holder.
pub mod call_function_on;
pub mod evaluate;
pub mod get_properties;
pub mod runtime_enable;
pub mod runtime_events;

pub use call_function_on::{CallFunctionOnTask, CallFunctionOnTaskBuilder};
pub use evaluate::{EvaluateTask, EvaluateTaskBuilder};
pub use get_properties::{GetPropertiesTask, GetPropertiesTaskBuilder};
pub use runtime_enable::{RuntimeEnableTask, RuntimeEnableTaskBuilder};

use crate::browser_async::{DebugSession};
use crate::browser_async::page_message::{PageResponse, PageResponseWrapper, ReceivedEvent};
use crate::protocol::{target};
use log::*;

#[derive(Debug)]
pub enum RuntimeEvent {
    ConsoleAPICalled(runtime_events::ConsoleAPICalled),
    ExceptionRevoked(runtime_events::ExceptionRevoked),
    ExceptionThrown(runtime_events::ExceptionThrown),
    ExecutionContextCreated(runtime_events::ExecutionContextCreated),
    ExecutionContextDestroyed(runtime_events::ExecutionContextDestroyed),
    ExecutionContextsCleared(runtime_events::ExecutionContextsCleared),
    InspectRequested(runtime_events::InspectRequested),
}

pub    fn handle_runtime_event(
        debug_session: &mut DebugSession,
        runtime_event: RuntimeEvent,
        _maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match runtime_event {
            RuntimeEvent::ConsoleAPICalled(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                let console_call_parameters = event.into_raw_parameters();
                tab.verify_execution_context_id(&console_call_parameters);
                Ok(PageResponseWrapper::default())
            }
            RuntimeEvent::ExceptionRevoked(_event) => {
                Ok(PageResponseWrapper::default())
            }
            RuntimeEvent::ExceptionThrown(event) => {
                warn!("ExceptionThrown: {:?}", event);
                Ok(PageResponseWrapper::default())
            }
            RuntimeEvent::ExecutionContextCreated(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.runtime_execution_context_created(event.get_execution_context_description());
                Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: None,
                    page_response: PageResponse::ReceivedEvent(ReceivedEvent::ExecutionContextCreated(event)),
                })
            }
            RuntimeEvent::ExecutionContextDestroyed(event) => {
                let execution_context_id = event.into_execution_context_id();
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.runtime_execution_context_destroyed(execution_context_id);
                Ok(PageResponseWrapper::default())
            }
            RuntimeEvent::ExecutionContextsCleared(_event) => {
                Ok(PageResponseWrapper::default())
            }
            RuntimeEvent::InspectRequested(_event) => {
                Ok(PageResponseWrapper::default())
            }
        }
    }