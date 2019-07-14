use super::PageEvent;
use super::super::TaskDescribe;
use crate::browser_async::{embedded_events, page_message::{PageResponse, ReceivedEvent,}};
use crate::protocol::{page};

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
    pub fn get_frame(&self) -> &page::Frame {
        &self.raw_event.params.frame
    }
    pub fn clone_frame(&self) -> page::Frame {
        self.raw_event.params.frame.clone()
    }

    pub fn url_contains(&self, url_part: &str) -> bool {
        self.raw_event.params.frame.url.contains(url_part)
    }
    pub fn url_starts_with(&self, url_part: &str) -> bool {
        self.raw_event.params.frame.url.starts_with(url_part)
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
        PageResponse::ReceivedEvent(ReceivedEvent::LoadEventFired(self.raw_event.params.timestamp))
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

// "{\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"E4863C4370687B1CD691540302E0A216\",\"message\":\"{\\\"method\\\":\\\"Page.lifecycleEvent\\\",\\\"params\\\":{\\\"frameId\\\":\\\"C50C87D5E5FBED3D00A80C15EEC95C55\\\",\\\"loaderId\\\":\\\"5D2AF99A577628F908336CB00B7E309C\\\",\\\"name\\\":\\\"firstMeaningfulPaintCandidate\\\",\\\"timestamp\\\":467696.10251}}\",\"targetId\":\"C50C87D5E5FBED3D00A80C15EEC95C55\"}}"
wrapper_raw_event!(
    TaskDescribe::PageEvent,
    PageEvent::LifeCycle,
    LifeCycle,
    page::events::LifecycleEvent
);

impl LifeCycle {

    pub fn get_name(&self) -> &str {
        &*self.raw_event.params.name
    }
    pub fn is_init(&self) -> bool {
        self.raw_event.params.name == "init"
    }

    pub fn is_first_paint(&self) -> bool {
        self.raw_event.params.name == "firstPaint"
    }

    pub fn is_first_contentful_paint(&self) -> bool {
        self.raw_event.params.name == "firstContentfulPaint"
    }

    pub fn is_fist_meaningful_paint_candicate(&self) -> bool {
        self.raw_event.params.name == "firstMeaningfulPaintCandidate"
    }

    pub fn is_first_image_paint(&self) -> bool {
        self.raw_event.params.name == "firstImagePaint"
    }

    pub fn is_commit(&self) -> bool {
        self.raw_event.params.name == "commit"
    }

    pub fn is_load(&self) -> bool {
        self.raw_event.params.name == "load"
    }

    pub fn is_dom_content_loaded(&self) -> bool {
        self.raw_event.params.name == "DOMContentLoaded"
    }

    pub fn is_network_almost_idle(&self) -> bool {
        self.raw_event.params.name == "networkAlmostIdle"
    }

    pub fn is_network_idle(&self) -> bool {
        self.raw_event.params.name == "networkIdle"
    }
}


// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "commit", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "DOMContentLoaded", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "load", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "networkAlmostIdle", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "networkIdle", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "networkAlmostIdle", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "63D840A62CD6045DB067488920C6CB95", name: "networkIdle", timestamp: 467692.44 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "init", timestamp: 467694.75 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "DOMContentLoaded", timestamp: 467695.16 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "load", timestamp: 467695.16 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "networkAlmostIdle", timestamp: 467695.16 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "firstPaint", timestamp: 467696.1 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "firstContentfulPaint", timestamp: 467696.1 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "firstMeaningfulPaintCandidate", timestamp: 467696.1 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "firstMeaningfulPaintCandidate", timestamp: 467696.16 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "firstImagePaint", timestamp: 467696.5 } })
// LifecycleParams { frame_id: "C50C87D5E5FBED3D00A80C15EEC95C55", loader_id: "5D2AF99A577628F908336CB00B7E309C", name: "firstMeaningfulPaintCandidate", timestamp: 467696.5 } })

#[cfg(test)]
mod tests {
    // use super::*;
    // use log::*;

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
