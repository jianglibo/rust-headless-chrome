use futures::{Poll, Async};
use websocket::futures::{Stream};
use super::chrome_debug_session::{ChromeDebugSession};
use super::page_message::{PageMessage};
use super::chrome_browser::{ChromeBrowser};
use super::interval_page_message::{IntervalPageMessage};
use failure::{self};
use std::default::Default;
use log::*;

/// An adapter for merging the output of two streams.
///
/// The merged stream produces items from either of the underlying streams as
/// they become available, and the streams are polled in a round-robin fashion.
/// Errors, however, are not merged: you get at most one error at a time.
// #[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct DebugSession {
    interval_page_message: IntervalPageMessage,
    pub chrome_debug_session: ChromeDebugSession,
    seconds_from_start: usize,
    flag: bool,
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
        Self {
            interval_page_message,
            chrome_debug_session,
            seconds_from_start: 0,
            flag: false,
        }
    }
    pub fn navigate_to(&mut self, url: &str, timeout_seconds: usize) {
        self.chrome_debug_session.navigate_to(url);
    }

    pub fn send_page_message(&mut self, item: PageMessage) -> Poll<Option<PageMessage>, failure::Error> {
        info!("{:?}", item);
        match &item {
            PageMessage::Interval => {
                self.seconds_from_start += 1;
                return Ok(Some(PageMessage::SecondsElapsed(self.seconds_from_start)).into());
            },
            _ => ()
        }
        return Ok(Some(item).into());
    }
}


impl Stream for DebugSession {
    type Item = PageMessage;
    type Error = failure::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let (a, b) = if self.flag {
            (&mut self.chrome_debug_session as &mut Stream<Item=_, Error=_>,
             &mut self.interval_page_message as &mut Stream<Item=_, Error=_>)
        } else {
            (&mut self.interval_page_message as &mut Stream<Item=_, Error=_>,
             &mut self.chrome_debug_session as &mut Stream<Item=_, Error=_>)
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