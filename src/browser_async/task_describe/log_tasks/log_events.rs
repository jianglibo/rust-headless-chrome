use super::LogEvent;
use super::super::TaskDescribe;
use super::super::super::{embedded_events};
use super::super::super::protocol::{dom};

wrapper_raw_event!(
    TaskDescribe::LogEvent,
    LogEvent::EntryAdded,
    EntryAdded,
    embedded_events::ChildNodeCountUpdated
);
