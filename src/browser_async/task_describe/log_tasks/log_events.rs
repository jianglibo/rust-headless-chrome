use super::super::super::protocol::chrome_log;
use super::super::TaskDescribe;
use super::LogEvent;

wrapper_raw_event!(
    TaskDescribe::LogEvent,
    LogEvent::EntryAdded,
    LogEntryAdded,
    chrome_log::events::LogEntryAdded
);
