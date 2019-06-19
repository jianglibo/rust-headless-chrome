// place holder.
pub mod call_function_on;
pub mod evaluate;
pub mod get_properties;
pub mod runtime_enable;
pub mod runtime_events;

pub use call_function_on::{RuntimeCallFunctionOnTask, RuntimeCallFunctionOnTaskBuilder};
pub use evaluate::{RuntimeEvaluateTask, RuntimeEvaluateTaskBuilder};
pub use get_properties::{RuntimeGetPropertiesTask, RuntimeGetPropertiesTaskBuilder};
pub use runtime_enable::{RuntimeEnableTask, RuntimeEnableTaskBuilder};

use crate::browser_async::{DebugSession, Tab};
use crate::browser_async::page_message::{response_object, PageResponse, PageResponseWrapper, MethodCallDone, ReceivedEvent};
use crate::protocol::{target};
use log::*;
use std::sync::{Arc, Mutex};

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
        maybe_session_id: Option<target::SessionID>,
        maybe_target_id: Option<target::TargetId>,
    ) -> Result<PageResponseWrapper, failure::Error> {
        match runtime_event {
            RuntimeEvent::ConsoleAPICalled(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                let console_call_parameters = event.into_raw_parameters();
                tab.verify_execution_context_id(&console_call_parameters);
            }
            RuntimeEvent::ExceptionRevoked(event) => {}
            RuntimeEvent::ExceptionThrown(event) => {}
            RuntimeEvent::ExecutionContextCreated(event) => {
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                let frame_id = tab
                    .runtime_execution_context_created(event.get_execution_context_description());
                return Ok(PageResponseWrapper {
                    target_id: maybe_target_id,
                    task_id: None,
                    page_response: PageResponse::ReceivedEvent(ReceivedEvent::ExecutionContextCreated(event)),
                });
                // return handle_event_return(
                //     maybe_target_id,
                //     PageResponse::ReceivedEvent(ReceivedEvent::ExecutionContextCreated(event)),
                // );
            }
            RuntimeEvent::ExecutionContextDestroyed(event) => {
                let execution_context_id = event.into_execution_context_id();
                let tab = debug_session.find_tab_by_id_mut(maybe_target_id.as_ref())?;
                tab.runtime_execution_context_destroyed(execution_context_id);
            }
            RuntimeEvent::ExecutionContextsCleared(event) => {}
            RuntimeEvent::InspectRequested(event) => {}
        }
        warn!("unhandled branch handle_runtime_event");
        Ok(PageResponseWrapper::default())
    }