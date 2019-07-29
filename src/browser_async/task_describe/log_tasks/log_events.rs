use super::LogEvent;
use super::super::TaskDescribe;
use super::super::super::{embedded_events};
use super::super::super::protocol::{chrome_log};

wrapper_raw_event!(
    TaskDescribe::LogEvent,
    LogEvent::EntryAdded,
    EntryAdded,
    chrome_log::events::LogEntryAdded
);
