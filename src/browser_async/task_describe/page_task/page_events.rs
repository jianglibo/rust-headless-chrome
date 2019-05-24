use super::super::{PageEvent, TaskDescribe};
use crate::browser_async::{embedded_events, page_message::PageResponse};
use crate::protocol::{page, target};

// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.domContentEventFired\\\",\\\"params\\\":{\\\"timestamp\\\":130939.223244}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::DomContentEventFired,
    DomContentEventFired,
    embedded_events::DomContentEventFired
);

// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.frameAttached\\\",\\\"params\\\":{\\\"frameId\\\":\\\"63A0D1EE2ADA69483E5991055F145C44\\\",\\\"parentFrameId\\\":\\\"74FEEFE9CACC814F52F89930129A15ED\\\",\\\"stack\\\":{\\\"callFrames\\\":[{\\\"functionName\\\":\\\"zi\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":115615},{\\\"functionName\\\":\\\"ji\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":120346},{\\\"functionName\\\":\\\"qi\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":123741},{\\\"functionName\\\":\\\"\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":137076},{\\\"functionName\\\":\\\"I.unstable_runWithPriority\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":7735},{\\\"functionName\\\":\\\"xa\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":137011},{\\\"functionName\\\":\\\"Ea\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":136787},{\\\"functionName\\\":\\\"Sa\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":136139},{\\\"functionName\\\":\\\"Va\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":135162},{\\\"functionName\\\":\\\"Ji\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":134053},{\\\"functionName\\\":\\\"Fa\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":138042},{\\\"functionName\\\":\\\"Na\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":138096},{\\\"functionName\\\":\\\"Da.render\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":141884},{\\\"functionName\\\":\\\"\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":139742},{\\\"functionName\\\":\\\"Pa\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":137318},{\\\"functionName\\\":\\\"Ba\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":139675},{\\\"functionName\\\":\\\"render\\\",\\\"scriptId\\\":\\\"12\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/0.57d1c5ef.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":142888},{\\\"functionName\\\":\\\"SurW\\\",\\\"scriptId\\\":\\\"14\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.db061593.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":12080},{\\\"functionName\\\":\\\"i\\\",\\\"scriptId\\\":\\\"14\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.db061593.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":517},{\\\"functionName\\\":\\\"u\\\",\\\"scriptId\\\":\\\"14\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.db061593.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":386},{\\\"functionName\\\":\\\"\\\",\\\"scriptId\\\":\\\"14\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.db061593.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":1469},{\\\"functionName\\\":\\\"\\\",\\\"scriptId\\\":\\\"14\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.db061593.js\\\",\\\"lineNumber\\\":0,\\\"columnNumber\\\":1473}]}}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameAttached,
    FrameAttached,
    page::events::FrameAttachedEvent
);

impl FrameAttached {
    pub fn into_raw_parameters(self) -> page::events::FrameAttachedParams {
        self.raw_event.params
    }
}

wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameDetached,
    FrameDetached,
    page::events::FrameDetachedEvent
);

impl FrameDetached {
    pub fn into_frame_id(self) -> page::FrameId {
        self.raw_event.params.frame_id
    }
}


// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.frameNavigated\\\",\\\"params\\\":{\\\"frame\\\":{\\\"id\\\":\\\"74FEEFE9CACC814F52F89930129A15ED\\\",\\\"loaderId\\\":\\\"53524592197E3E19D8E72E1379A32393\\\",\\\"url\\\":\\\"https://pc.xuexi.cn/points/login.html?ref=https://www.xuexi.cn/\\\",\\\"securityOrigin\\\":\\\"https://pc.xuexi.cn\\\",\\\"mimeType\\\":\\\"text/html\\\"}}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameNavigated,
    FrameNavigated,
    page::events::FrameNavigatedEvent
);

impl FrameNavigated {
    pub fn into_frame(self) -> page::Frame {
        self.raw_event.params.frame
    }
}

// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.loadEventFired\\\",\\\"params\\\":{\\\"timestamp\\\":130944.691823}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::LoadEventFired,
    LoadEventFired,
    embedded_events::LoadEventFired
);

impl LoadEventFired {
    pub fn into_page_response(self) -> PageResponse {
        PageResponse::LoadEventFired(self.raw_event.params.timestamp)
    }
}

// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.frameStoppedLoading\\\",\\\"params\\\":{\\\"frameId\\\":\\\"9EE608110784CAE5FAF6D3033FE20B3B\\\"}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameStoppedLoading,
    FrameStoppedLoading,
    page::events::FrameStoppedLoadingEvent
);

impl FrameStoppedLoading {
    pub fn into_frame_id(self) -> page::FrameId {
        self.raw_event.params.frame_id
    }
}

// {\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"1B34295E2E49181EC18E08C21FD08148\",\"message\":\"{\\\"method\\\":\\\"Page.frameStartedLoading\\\",\\\"params\\\":{\\\"frameId\\\":\\\"74FEEFE9CACC814F52F89930129A15ED\\\"}}\",\"targetId\":\"74FEEFE9CACC814F52F89930129A15ED\"}}
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::FrameStartedLoading,
    FrameStartedLoading,
    page::events::FrameStartedLoadingEvent
);

impl FrameStartedLoading {
    pub fn into_frame_id(self) -> page::FrameId {
        self.raw_event.params.frame_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::*;

    #[macro_export]
    macro_rules! add {
        {one to $input:expr} => ($input + 1);
        {two to $input:expr} => ($input + 2);
    }

    #[test]
    fn a_macro() {
        println!("Add two: {}", add!(two to 25/5));
    }
}
