use log::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::protocol::{self};
use super::chrome_debug_session::{ChromeDebugSession};

#[derive(Debug)]
pub struct Tab {
    chrome_session: Arc<Mutex<ChromeDebugSession>>,
    pub target_info: protocol::target::TargetInfo,
}
