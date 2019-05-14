use crate::protocol::network;
use serde::{Deserialize, Serialize};

pub mod types {
    use serde::{Deserialize, Serialize};
    pub type FrameId = String;

    #[derive(Debug, Serialize, Clone)]
    #[serde(rename_all = "snake_case")]
    pub enum TransitionType {
        Link,
        Typed,
        AddressBar,
        AutoBookmark,
        AutoSubframe,
        ManualSubframe,
        Generated,
        AutoToplevel,
        FormSubmit,
        Reload,
        Keyword,
        KeywordGenerated,
        Other,
    }

}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Frame {
    pub id: String,
    pub parent_id: Option<String>,
    pub loader_id: network::types::LoaderId,
    pub name: Option<String>,
    pub url: String,
    pub security_origin: String,
    pub mime_type: String,
    pub unreachable_url: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum InternalScreenshotFormat {
    JPEG,
    PNG,
}

/// Viewport for capturing screenshot.
#[derive(Debug, Clone, Serialize)]
pub struct Viewport {
    /// X offset in device independent pixels
    pub x: f64,
    /// Y offset in device independent pixels
    pub y: f64,
    /// Rectangle width in device independent pixels
    pub width: f64,
    /// Rectangle height in device independent pixels
    pub height: f64,
    /// Page scale factor
    pub scale: f64,
}

/// The format a screenshot will be captured in
#[derive(Debug, Clone)]
pub enum ScreenshotFormat {
    /// Optionally compression quality from range [0..100]
    JPEG(Option<u8>),
    PNG,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PrintToPdfOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub landscape: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_header_footer: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub print_background: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_width: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paper_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_ranges: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_invalid_page_ranges: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub footer_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefer_css_page_size: Option<bool>,
}

pub mod events {
    use crate::protocol::runtime;
    use serde::Deserialize;
    #[derive(Deserialize, Debug, Clone)]
    pub struct LifecycleEvent {
        pub params: LifecycleParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct LifecycleParams {
        pub frame_id: String,
        pub loader_id: String,
        pub name: String,
        pub timestamp: f32,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameStartedLoadingEvent {
        pub params: FrameStartedLoadingParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameStartedLoadingParams {
        pub frame_id: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameNavigatedEvent {
        pub params: FrameNavigatedParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameNavigatedParams {
        pub frame: super::Frame,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameDetachedEvent {
        pub params: FrameDetachedParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameDetachedParams {
        pub frame_id: String,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameAttachedEvent {
        pub params: FrameAttachedParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameAttachedParams {
        pub frame_id: String,
        pub parent_frame_id: String,
        pub stack: Option<runtime::types::StackTrace>,
    }

    #[derive(Deserialize, Debug, Clone)]
    pub struct FrameStoppedLoadingEvent {
        pub params: FrameStoppedLoadingParams,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameStoppedLoadingParams {
        pub frame_id: String,
    }
}

pub mod methods {
    use super::PrintToPdfOptions;
    use super::*;
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct CaptureScreenshot {
        pub format: super::InternalScreenshotFormat,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub quality: Option<u8>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub clip: Option<super::Viewport>,
        pub from_surface: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CaptureScreenshotReturnObject {
        pub data: String,
    }
    impl Method for CaptureScreenshot {
        const NAME: &'static str = "Page.captureScreenshot";
        type ReturnObject = CaptureScreenshotReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub(crate) struct PrintToPdf {
        #[serde(flatten)]
        pub options: Option<PrintToPdfOptions>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct PrintToPdfReturnObject {
        pub data: String,
    }
    impl Method for PrintToPdf {
        const NAME: &'static str = "Page.printToPDF";
        type ReturnObject = PrintToPdfReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Reload<'a> {
        pub ignore_cache: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub script_to_evaluate: Option<&'a str>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ReloadReturnObject {}
    impl<'a> Method for Reload<'a> {
        const NAME: &'static str = "Page.reload";
        type ReturnObject = ReloadReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetLifecycleEventsEnabled {
        pub enabled: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetLifecycleEventsEnabledReturnObject {}
    impl Method for SetLifecycleEventsEnabled {
        const NAME: &'static str = "Page.setLifecycleEventsEnabled";
        type ReturnObject = SetLifecycleEventsEnabledReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetFrameTree {}

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct FrameTree {
        pub frame: super::Frame,
        pub child_frames: Option<Vec<FrameTree>>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct GetFrameTreeReturnObject {
        pub frame_tree: FrameTree,
    }
    impl Method for GetFrameTree {
        const NAME: &'static str = "Page.getFrameTree";
        type ReturnObject = GetFrameTreeReturnObject;
    }

    #[derive(Serialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Navigate<'a> {
        pub url: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub referrer: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub transition_type: Option<types::TransitionType>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub frame_id: Option<types::FrameId>,
    }
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct NavigateReturnObject {
        pub frame_id: types::FrameId,
        pub loader_id: Option<network::types::LoaderId>,
        pub error_text: Option<String>,
    }
    impl<'a> Method for Navigate<'a> {
        const NAME: &'static str = "Page.navigate";
        type ReturnObject = NavigateReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EnableReturnObject {}
    impl Method for Enable {
        const NAME: &'static str = "Page.enable";
        type ReturnObject = EnableReturnObject;
    }
}

#[cfg(test)]
mod tests {
    use log::*;
    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_frame_attached_event() {
        let message = "{\"method\":\"Page.frameAttached\",\"params\":{\"frameId\":\"100FD572BD64BB38EB2CAEE354C93F52\",\"parentFrameId\":\"2D0E2292FC393BB4953C7629AF041862\",\"stack\":{\"callFrames\":[{\"functionName\":\"Ho\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":445965},{\"functionName\":\"_i\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":466041},{\"functionName\":\"Oi\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":463515},{\"functionName\":\"Ei\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":462867},{\"functionName\":\"Ci\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":461888},{\"functionName\":\"$o\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":460808},{\"functionName\":\"Ii\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":469592},{\"functionName\":\"Fi\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":469646},{\"functionName\":\"Bi.render\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":473435},{\"functionName\":\"\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":471293},{\"functionName\":\"Ni\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":468914},{\"functionName\":\"qi\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":471226},{\"functionName\":\"render\",\"scriptId\":\"12\",\"url\":\"https://pc.xuexi.cn/points/0.1f01cb06.js\",\"lineNumber\":0,\"columnNumber\":474407},{\"functionName\":\"SurW\",\"scriptId\":\"14\",\"url\":\"https://pc.xuexi.cn/points/login.46f4e7c1.js\",\"lineNumber\":0,\"columnNumber\":13244},{\"functionName\":\"a\",\"scriptId\":\"14\",\"url\":\"https://pc.xuexi.cn/points/login.46f4e7c1.js\",\"lineNumber\":0,\"columnNumber\":517},{\"functionName\":\"u\",\"scriptId\":\"14\",\"url\":\"https://pc.xuexi.cn/points/login.46f4e7c1.js\",\"lineNumber\":0,\"columnNumber\":386},{\"functionName\":\"\",\"scriptId\":\"14\",\"url\":\"https://pc.xuexi.cn/points/login.46f4e7c1.js\",\"lineNumber\":0,\"columnNumber\":1469},{\"functionName\":\"\",\"scriptId\":\"14\",\"url\":\"https://pc.xuexi.cn/points/login.46f4e7c1.js\",\"lineNumber\":0,\"columnNumber\":1473}]}}}";
        serde_json::from_str::<events::FrameAttachedEvent>(message.as_ref()).unwrap();
    }
}
