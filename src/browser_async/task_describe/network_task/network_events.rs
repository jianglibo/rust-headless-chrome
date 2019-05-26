use super::NetworkEvent;
use super::super::TaskDescribe;
use crate::browser_async::embedded_events;
use crate::protocol::{network};


wrapper_raw_event!(
    TaskDescribe::NetworkEvent,
    NetworkEvent::RequestWillBeSent,
    RequestWillBeSent,
    embedded_events::RequestWillBeSent
);

wrapper_raw_event!(
    TaskDescribe::NetworkEvent,
    NetworkEvent::ResponseReceived,
    ResponseReceived,
    embedded_events::ResponseReceived
);

impl ResponseReceived {
    pub fn into_raw_parameters(self) -> embedded_events::ResponseReceivedParams {
        self.raw_event.params
    }
}

wrapper_raw_event!(
    TaskDescribe::NetworkEvent,
    NetworkEvent::DataReceived,
    DataReceived,
    embedded_events::DataReceived
);

wrapper_raw_event!(
    TaskDescribe::NetworkEvent,
    NetworkEvent::LoadingFinished,
    LoadingFinished,
    embedded_events::LoadingFinished
);


wrapper_raw_event!(
    TaskDescribe::NetworkEvent,
    NetworkEvent::RequestIntercepted,
    RequestIntercepted,
    network::events::RequestInterceptedEvent
);