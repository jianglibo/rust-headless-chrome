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