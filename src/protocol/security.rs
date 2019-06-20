use serde::Deserialize;

pub type CertificateId = u32;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum MixedContentType {
    Blockable,
    OptionallyBlockable,
    #[serde(rename(deserialize = "none"))]
    Arimasen,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum SecurityState {
    Unknown,
    Neutral,
    Insecure,
    Secure,
    Info,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum CertificateErrorAction {
    #[serde(rename(deserialize = "continue"))]
    Tsuduku,
    Cancel,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SecurityStateExplanation {
    pub security_state: SecurityState,
    pub title: String,
    pub summary: String,
    pub description: String,
    pub mixed_content_type: MixedContentType,
    pub certificate: Vec<String>,
    pub recommendations: Option<Vec<String>>,
}

pub mod events {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    pub struct CertificateError {
        pub params: CertificateErrorParams,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct CertificateErrorParams {
        pub event_id: u32,
        pub error_type: String,
        #[serde(rename = "requestURL")]
        pub request_url: String,
    }


    #[derive(Deserialize, Debug, Clone)]
    pub struct SecurityStateChangedEvent {
        pub params: SecurityStateChangedParams,
    }

    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SecurityStateChangedParams {
        pub security_state: SecurityState,
        pub scheme_is_cryptographic: bool,
        pub explanations: Vec<SecurityStateExplanation>,
        pub insecure_content_status: Option<serde_json::Value>,
        pub summary: Option<String>,
    }
}

pub mod methods {
    use serde::{Deserialize, Serialize};
    use crate::protocol::Method;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetIgnoreCertificateErrors {
        pub ignore: bool,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SetIgnoreCertificateErrorsObject {}
    impl Method for SetIgnoreCertificateErrors {
        const NAME: &'static str = "Security.setIgnoreCertificateErrors";
        type ReturnObject = SetIgnoreCertificateErrorsObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EnableReturnObject {}
    impl Method for Enable {
        const NAME: &'static str = "Security.enable";
        type ReturnObject = EnableReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Disable {}
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DisableReturnObject {}
    impl Method for Disable {
        const NAME: &'static str = "Security.disable";
        type ReturnObject = DisableReturnObject;
    }

}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use log::*;

//     #[test]
//     fn test_certificate_error() {
//         assert!(true);
//     }
// }
