use futures::{Poll, Async};
use websocket::futures::{Stream};
use tokio::timer::{Interval};
// use super::page_message::{PageMessage};
use super::task_describe::TaskDescribe;
use std::time::{Duration};

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

impl Stream for IntervalPageMessage {
    type Item = TaskDescribe;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        loop {
            if let Some(_) = try_ready!(self.interval.poll()) {
                return Ok(Async::Ready(Some(TaskDescribe::Interval)));
            }
        }
    }
}