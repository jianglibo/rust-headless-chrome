use super::{network, runtime};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LogSource {
    Xml,
    Javascript,
    Network,
    Storage,
    Appcache,
    Rendering,
    Security,
    Deprecation,
    Worker,
    Violation,
    Intervention,
    Recommendation,
    Other,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    Verbose,
    Info,
    Warning,
    Error,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub source: LogSource,
    pub level: LogLevel,
    pub text: String,
    pub timestamp: runtime::Timestamp,
    pub url: Option<String>,
    pub line_number: Option<u64>,
    pub stack_trace: Option<runtime::StackTrace>,
    pub network_request_id: Option<network::RequestId>,
    pub worker_id: Option<String>,
    pub args: Option<Vec<runtime::RemoteObject>>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViolationSetting {
    pub name: String, // longTask, longLayout, blockedEvent, blockedParser, discouragedAPIUse, handler, recurringHandler
    pub threshold: u64,
}

pub mod events {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct LogEntryAdded {
        pub params: LogEntryAddedParams,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct LogEntryAddedParams {
        pub entry: LogEntry,
    }

}

pub mod methods {
    use super::super::{EmptyReturnObject, Method};
    use serde::{Serialize};

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {}
    impl Method for Enable {
        const NAME: &'static str = "Log.enable";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Disable {}
    impl Method for Disable {
        const NAME: &'static str = "Log.disable";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Clear {}
    impl Method for Clear {
        const NAME: &'static str = "Log.clear";
        type ReturnObject = EmptyReturnObject;
    }

} 