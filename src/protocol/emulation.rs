// use serde::{Deserializer};
use serde::{Serialize};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum ScreenOrientationType {
    PortraitPrimary,
    PortraitSecondary,
    LandscapePrimary,
    LandscapeSecondary,
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ScreenOrientation {
    #[serde(rename = "type")]
    pub orientation_type: ScreenOrientationType,
    pub angle: i16,
}

pub mod events {
    // use crate::protocol::runtime;
    // use serde::Deserialize;

    // #[derive(Deserialize, Debug, Clone)]
    // pub struct LifecycleEvent {
    //     pub params: LifecycleParams,
    // }
    // #[derive(Deserialize, Debug, Clone)]
    // #[serde(rename_all = "camelCase")]
    // pub struct LifecycleParams {
    //     pub frame_id: String,
    //     pub loader_id: String,
    //     pub name: String,
    //     pub timestamp: f32,
    // }
}

pub mod methods {
    use super::*;
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};
    use super::super::{network, dom, page, EmptyReturnObject};

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct CanEmulate {
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct CanEmulateReturnObject {
        pub result: bool,
    }
    impl Method for CanEmulate {
        const NAME: &'static str = "Emulation.canEmulate";
        type ReturnObject = CanEmulateReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ClearDeviceMetricsOverride {
    }
    impl Method for ClearDeviceMetricsOverride {
        const NAME: &'static str = "Emulation.clearDeviceMetricsOverride";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ClearGeolocationOverride {
    }
    impl Method for ClearGeolocationOverride {
        const NAME: &'static str = "Emulation.clearGeolocationOverride";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ResetPageScaleFactor {
    }
    impl Method for ResetPageScaleFactor {
        const NAME: &'static str = "Emulation.resetPageScaleFactor";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetFocusEmulationEnabled {
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetFocusEmulationEnabledReturnObject {
        pub enabled: bool,
    }
    impl Method for SetFocusEmulationEnabled {
        const NAME: &'static str = "Emulation.setFocusEmulationEnabled";
        type ReturnObject = SetFocusEmulationEnabledReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetCPUThrottlingRate {
        pub rate: u8,
    }
    impl Method for SetCPUThrottlingRate {
        const NAME: &'static str = "Emulation.setCPUThrottlingRate";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetDefaultBackgroundColorOverride {
        pub color: Option<dom::RGBA>,
    }
    impl Method for SetDefaultBackgroundColorOverride {
        const NAME: &'static str = "Emulation.setDefaultBackgroundColorOverride";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetDeviceMetricsOverride {
        pub width: u64,
        pub height: u64,
        pub device_scale_factor: f64,
        pub mobile: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub scale: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub screen_width: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub screen_height: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub position_x: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub position_y: Option<u64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub dont_set_visible_size: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub screen_orientation: Option<ScreenOrientation>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub viewport: Option<page::Viewport>,
    }

    impl Method for SetDeviceMetricsOverride {
        const NAME: &'static str = "Emulation.setDeviceMetricsOverride";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetScrollbarsHidden {
        pub hidden: bool,
    }
    impl Method for SetScrollbarsHidden {
        const NAME: &'static str = "Emulation.setScrollbarsHidden";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetDocumentCookieDisabled {
        pub hidden: bool,
    }
    impl Method for SetDocumentCookieDisabled {
        const NAME: &'static str = "Emulation.setDocumentCookieDisabled";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetEmitTouchEventsForMouse {
        pub enabled: bool,
        pub configuration: Option<String>,
    }
    impl Method for SetEmitTouchEventsForMouse {
        const NAME: &'static str = "Emulation.setEmitTouchEventsForMouse";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetEmulatedMedia {
        pub media: String,
    }
    impl Method for SetEmulatedMedia {
        const NAME: &'static str = "Emulation.setEmulatedMedia";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetGeolocationOverride {
        pub latitude: Option<f64>,
        pub longitude: Option<f64>,
        pub accuracy: Option<f64>,
    }
    impl Method for SetGeolocationOverride {
        const NAME: &'static str = "Emulation.setGeolocationOverride";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetPageScaleFactor {
        pub page_scale_factor: f64,
    }
    impl Method for SetPageScaleFactor {
        const NAME: &'static str = "Emulation.setPageScaleFactor";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetScriptExecutionDisabled {
        pub value: bool,
    }
    impl Method for SetScriptExecutionDisabled {
        const NAME: &'static str = "Emulation.setScriptExecutionDisabled";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetTouchEmulationEnabled {
        pub enabled: bool,
        pub max_touch_points: Option<u8>,
    }
    impl Method for SetTouchEmulationEnabled {
        const NAME: &'static str = "Emulation.setTouchEmulationEnabled";
        type ReturnObject = EmptyReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetVirtualTimePolicy {
        pub policy: String, //advance, pause, pauseIfNetworkFetchesPending
        pub budget: Option<f64>,
        pub max_virtual_time_task_starvation_count: Option<u64>,
        pub wait_for_navigation: Option<bool>,
        pub initial_virtual_time: Option<network::TimeSinceEpoch>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetVirtualTimePolicyReturnObject {
        pub virtual_time_ticks_base: f64,
    }

    impl Method for SetVirtualTimePolicy {
        const NAME: &'static str = "Emulation.setVirtualTimePolicy";
        type ReturnObject = SetVirtualTimePolicyReturnObject;
    }



    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetTimezoneOverride {
        pub timezone_id: String,
    }

    impl Method for SetTimezoneOverride {
        const NAME: &'static str = "Emulation.setTimezoneOverride";
        type ReturnObject = EmptyReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetUserAgentOverride {
        pub user_agent: String,
        pub accept_language: Option<String>,
        pub platform: Option<String>,
    }
    impl Method for SetUserAgentOverride {
        const NAME: &'static str = "Emulation.setUserAgentOverride";
        type ReturnObject = EmptyReturnObject;
    }

}