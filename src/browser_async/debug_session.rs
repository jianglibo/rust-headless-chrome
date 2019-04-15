use super::chrome_browser::ChromeBrowser;
use super::chrome_debug_session::ChromeDebugSession;
use super::dev_tools_method_util::SessionId;
use super::interval_page_message::IntervalPageMessage;
use super::tab::Tab;
use super::task_describe::TaskDescribe;
use crate::protocol;
use failure;
use futures::{Async, Poll};
use log::*;
use std::collections::HashMap;
use std::default::Default;
use websocket::futures::Stream;
use std::sync::{Arc, Mutex};

const DEFAULT_TAB_NAME: &'static str = "_default_tab_";


struct Wrapper {
    pub chrome_debug_session: Arc<Mutex<ChromeDebugSession>>,
}

impl Stream for Wrapper {
    type Item = TaskDescribe;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        self.chrome_debug_session.lock().unwrap().poll()
    }
}

/// An adapter for merging the output of two streams.
///
/// The merged stream produces items from either of the underlying streams as
/// they become available, and the streams are polled in a round-robin fashion.
/// Errors, however, are not merged: you get at most one error at a time.
// #[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct DebugSession {
    interval_page_message: IntervalPageMessage,
    pub chrome_debug_session: Arc<Mutex<ChromeDebugSession>>,
    seconds_from_start: usize,
    flag: bool,
    tabs: HashMap<&'static str, Tab>,
    wrapper: Wrapper,
}

impl Default for DebugSession {
    fn default() -> Self {
        let browser = ChromeBrowser::new();
        let chrome_debug_session = ChromeDebugSession::new(browser);
        Self::new(chrome_debug_session)
    }
}


impl DebugSession {
    pub fn new(chrome_debug_session: ChromeDebugSession) -> Self {
        let interval_page_message = IntervalPageMessage::new();
        let arc_cds = Arc::new(Mutex::new(chrome_debug_session));
        Self {
            interval_page_message,
            chrome_debug_session: arc_cds.clone(),
            seconds_from_start: 0,
            flag: false,
            tabs: HashMap::new(),
            wrapper: Wrapper {
                chrome_debug_session: arc_cds,
            },
        }
    }
    pub fn get_tab_by_id_mut(&mut self, tab_id: String) -> Option<&mut Tab> {
        self.tabs
            .values_mut()
            .find(|t| t.target_info.target_id == tab_id)
    }

    pub fn get_tab_by_id(&self, tab_id: String) -> Option<&Tab> {
        self.tabs
            .values()
            .find(|t| t.target_info.target_id == tab_id)
    }

    pub fn main_tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(DEFAULT_TAB_NAME)
    }
    pub fn main_tab(&self) -> Option<&Tab> {
        self.tabs.get(DEFAULT_TAB_NAME)
    }

    pub fn send_page_message(
        &mut self,
        item: TaskDescribe,
    ) -> Poll<Option<TaskDescribe>, failure::Error> {
        match &item {
            TaskDescribe::Interval => {
                self.seconds_from_start += 1;
                return Ok(Some(TaskDescribe::SecondsElapsed(self.seconds_from_start)).into());
            }
            TaskDescribe::PageCreated(target_info, page_name) => {
                trace!(
                    "receive page created event: {:?}, {:?}",
                    target_info,
                    page_name
                );
                let mut tab = Tab::new(target_info.clone(), Arc::clone(&self.chrome_debug_session));
                tab.attach_to_page();
                self.tabs.insert(page_name.unwrap_or(DEFAULT_TAB_NAME), tab);
            }
            TaskDescribe::PageAttached(target_info, session_id) => {
                trace!(
                    "receive page attached event: {:?}, {:?}",
                    target_info,
                    session_id.clone()
                );
                if let Some(tab) = self.get_tab_by_id_mut(target_info.target_id.clone()) {
                    tab.session_id.replace(session_id.clone());
                    tab.page_enable();
                } else {
                    error!("got attach event, but cannot find target.");
                }
            }
            TaskDescribe::FrameNavigated(target_id, changing_frame) => {
                if let Some(tab) = self.get_tab_by_id_mut(target_id.clone()) {
                    tab._frame_navigated(changing_frame.clone());
                } else {
                    error!("got frame navigated event, but cannot find target.");
                }
            }
            TaskDescribe::GetDocument(_task_id, target_id, node) => {
                let t_id = target_id.as_ref().cloned().expect("get document task got none target_id.");
                if let Some(tab) = self.get_tab_by_id_mut(t_id) {
                    tab.root_node = node.as_ref().cloned();
                } else {
                    error!("got get document event, but cannot find target.");
                }
            }
            _ => (),
        }
        return Ok(Some(item).into());
    }
}


impl Stream for DebugSession {
    type Item = TaskDescribe;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let (a, b) = if self.flag {
            (
                &mut self.wrapper as &mut Stream<Item = _, Error = _>,
                &mut self.interval_page_message as &mut Stream<Item = _, Error = _>,
            )
        } else {
            (
                &mut self.interval_page_message as &mut Stream<Item = _, Error = _>,
                &mut self.wrapper as &mut Stream<Item = _, Error = _>,
            )
        };
        self.flag = !self.flag;
        let a_done = match a.poll()? {
            Async::Ready(Some(item)) => return self.send_page_message(item),
            Async::Ready(None) => true,
            Async::NotReady => false,
        };

        match b.poll()? {
            Async::Ready(Some(item)) => {
                // If the other stream isn't finished yet, give them a chance to
                // go first next time as we pulled something off `b`.
                if !a_done {
                    self.flag = !self.flag;
                }
                self.send_page_message(item)
            }
            Async::Ready(None) if a_done => Ok(None.into()),
            Async::Ready(None) | Async::NotReady => Ok(Async::NotReady),
        }
    }
}