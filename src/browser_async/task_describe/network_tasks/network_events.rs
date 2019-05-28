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

impl RequestWillBeSent {
    pub fn get_request_id(&self) -> network::RequestId {
        self.raw_event.params.request_id.clone()
    }

    pub fn get_request_object(&self) -> &network::Request {
        &self.raw_event.params.request
    }
}

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
    NetworkEvent::LoadingFailed,
    LoadingFailed,
    embedded_events::LoadingFailed
);

impl LoadingFailed {
    pub fn get_request_id(&self) -> network::RequestId {
        self.raw_event.params.request_id.clone()
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

impl LoadingFinished {
    pub fn get_request_id(&self) -> network::RequestId {
        self.raw_event.params.request_id.clone()
    }
}


wrapper_raw_event!(
    TaskDescribe::NetworkEvent,
    NetworkEvent::RequestIntercepted,
    RequestIntercepted,
    network::events::RequestInterceptedEvent
);

impl RequestIntercepted {
    pub fn get_raw_parameters(&self) -> &network::events::RequestInterceptedEventParams {
        &self.raw_event.params
    }

    pub fn get_interception_id(&self) -> String {
        self.raw_event.params.interception_id.clone()
    }
}