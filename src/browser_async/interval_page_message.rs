use futures::{Poll};
use websocket::futures::{Stream};
use tokio::timer::{Interval};
use super::task_describe::TaskDescribe;
use std::time::{Duration};
use crate::protocol::target;

#[derive(Debug)]
pub struct IntervalPageMessage {
    interval: Interval,
}

impl IntervalPageMessage {
    pub fn new() -> Self {
        Self {
            interval: Interval::new_interval(Duration::from_secs(1)),
        }
    } 
}

impl Default for IntervalPageMessage {
    fn default() -> Self {
        Self::new()
    }   
}

impl Stream for IntervalPageMessage {
    type Item = (Option<target::SessionID>, Option<target::TargetId>, TaskDescribe);
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if try_ready!(self.interval.poll()).is_some() {
                return Ok(Some((None, None, TaskDescribe::Interval)).into());
            }
        }
    }
}