pub mod create_target;
pub mod set_discover_target_task;
pub mod target_events;

pub use create_target::{CreateTargetTask, CreateTargetTaskBuilder};
pub use set_discover_target_task::{SetDiscoverTargetsTask, SetDiscoverTargetsTaskBuilder};



#[derive(Debug)]
pub enum TargetEvent {
    ReceivedMessageFromTarget(target_events::ReceivedMessageFromTarget),
    TargetCreated(target_events::TargetCreated),
    TargetCrashed(target_events::TargetCrashed),
    TargetInfoChanged(target_events::TargetInfoChanged),
    AttachedToTarget(target_events::AttachedToTarget),
}