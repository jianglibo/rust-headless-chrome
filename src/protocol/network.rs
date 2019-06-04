type Headers = HashMap<String, String>;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::protocol::{runtime, page, security};

    pub type LoaderId = String;
    pub type MonotonicTime = f32;
    pub type RequestId = String;
    pub type TimeSinceEpoch = f32;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub enum InterceptionStage {
        Request,
        HeadersReceived,
    }

    
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum BlockedReason {
    Other, Csp,
     MixedContent,
     Origin,
     Inspector,
     SubresourceFilter,
     ContentYype,
     CollapsedByClient,
}
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub enum ResourceType {
        Document,
         Stylesheet,
          Image, Media,
           Font, 
           Script, 
           TextTrack, 
           XHR, 
           Fetch, 
           EventSource, 
           WebSocket,
            Manifest, 
            SignedExchange, 
            Ping,
             CSPViolationReport,
              Other
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub enum InitiatorType {
        Parser,
        Script, 
        Preload, 
        #[serde(rename = "SignedExchange")]
        SignedExchange,
        Other,
    }

// \"initiator\":{\"type\":\"script\",\"stack\":{\"callFrames\":[{\"functionName\":\"send\",\"scriptId\":\"42\",\"url\":\"https://59.202.58.131/assets/scripts/vendor-daf58f8629.js\",\"lineNumber\":84,\"columnNumber\":3511}]}},
    #[derive(Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Initiator {
        #[serde(rename = "type")]
        pub initiator_type: InitiatorType,
        pub stack: Option<runtime::StackTrace>,
        pub url: Option<String>,
        pub line_number: Option<u32>,
    }

// {\"requestTime\":656873.862454,\"proxyStart\":-1,\"proxyEnd\":-1,\"dnsStart\":3.705,\"dnsEnd\":21.607,
// \"connectStart\":21.607,\"connectEnd\":108.254,\"sslStart\":39.61,\"sslEnd\":108.248,\"workerStart\":-1,
// \"workerReady\":-1,\"sendStart\":108.602,\"sendEnd\":109.147,\"pushStart\":0,\"pushEnd\":0,\"receiveHeadersEnd\":129.336}
    #[derive(Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct ResourceTiming {
        pub request_time: f32,
        pub proxy_start: f32,
        pub proxy_end: f32,
        pub dns_start: f32,
        pub dns_end: f32,
        pub connect_start: f32,
        pub connect_end: f32,
        pub ssl_start: f32,
        pub ssl_end: f32,
        pub worker_start: f32,
        pub worker_ready: f32,
        pub send_start: f32,
        pub send_end: f32,
        pub push_start: f32,
        pub push_end: f32,
        pub receive_headers_end: f32,
    }

    #[derive(Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SignedCertificateTimestamp {
        pub status: String,
        pub origin: String,
        pub log_description: String,
        pub log_id: String,
        pub timestamp: TimeSinceEpoch,
        pub has_algorithm: String,
        pub signature_algorithm: String,
        pub signature_data: String,
    }

// \"securityDetails\":{\"protocol\":\"TLS 1.2\",\"keyExchange\":\"ECDHE_RSA\",\"keyExchangeGroup\":\"P-256\",
// \"cipher\":\"AES_128_GCM\",\"certificateId\":0,\"subjectName\":\"\",\"sanList\":[],\"issuer\":\"\",
// \"validFrom\":1481188811,\"validTo\":1796548811,\"signedCertificateTimestampList\":[],
// \"certificateTransparencyCompliance\":\"unknown\"}},
    #[derive(Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SecurityDetails {
        pub protocol: String,
        pub key_exchange: String,
        pub key_exchange_group: Option<String>,
        pub cipher: String,
        pub mac: Option<String>,
        pub certificate_id: security::CertificateId,
        pub subject_name: String,
        pub san_list: Vec<String>,
        pub issuer: String,
        pub valid_from: TimeSinceEpoch,
        pub valid_to: TimeSinceEpoch,
        pub signed_certificate_timestamp_list: Vec<SignedCertificateTimestamp>,
        // Allow value: unknown, not-compliant, compliant
        pub certificate_transparency_compliance: String,
    }

// \"response\":{\"url\":\"https://59.202.58.131/api/assess/getAssessDataOfFirstIndex?curDate=2019%2F05%2F24&orgId=1&_=1558775437908\",
// \"status\":200,\"statusText\":\"OK\",
// \"headers\":{\"Date\":\"Sat, 25 May 2019 09:18:22 GMT\",\"Server\":\"nginx/1.6.3\",\"Connection\":\"keep-alive\",\"Content-Length\":\"1325\",\"X-Application-Context\":\"dingplus-user-web-dubbo:localtest,menu,privilege\",\"Content-Type\":\"application/json;charset=utf-8\"},
// \"mimeType\":\"application/json\",
// \"connectionReused\":true,\"connectionId\":395,\"remoteIPAddress\":\"59.202.58.131\",
// \"remotePort\":443,\"fromDiskCache\":false,\"fromServiceWorker\":false,
// \"encodedDataLength\":242,
// \"timing\":{\"requestTime\":602980.457097,\"proxyStart\":-1,\"proxyEnd\":-1,\"dnsStart\":-1,\"dnsEnd\":-1,\"connectStart\":-1,\"connectEnd\":-1,\"sslStart\":-1,\"sslEnd\":-1,\"workerStart\":-1,\"workerReady\":-1,\"sendStart\":0.436,\"sendEnd\":0.494,\"pushStart\":0,\"pushEnd\":0,\"receiveHeadersEnd\":89.751},
// \"protocol\":\"http/1.1\",\"securityState\":\"insecure\",
// \"securityDetails\":{\"protocol\":\"TLS 1.2\",\"keyExchange\":\"ECDHE_RSA\",\"keyExchangeGroup\":\"P-256\",\"cipher\":\"AES_128_GCM\",\"certificateId\":0,\"subjectName\":\"\",\"sanList\":[],\"issuer\":\"\",\"validFrom\":1481188811,\"validTo\":1796548811,\"signedCertificateTimestampList\":[],\"certificateTransparencyCompliance\":\"unknown\"}},
    #[derive(Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Response {
        pub url: String,
        pub status: i32,
        pub status_text: String,
        pub headers: Headers,
        pub headers_text: Option<String>,
        pub mime_type: String,
        pub request_headers: Option<Headers>,
        pub request_headers_text: Option<String>,
        pub connection_reused: bool,
        pub connection_id: u32,
        #[serde(rename = "remoteIPAddress")]
        pub remote_ip_address: Option<String>,
        pub remote_port: Option<u32>,
        pub from_disk_cache: Option<bool>,
        pub from_service_worker: Option<bool>,
        pub from_prefetch_cache: Option<bool>,
        pub encoded_data_length: u32,
        pub timing: Option<ResourceTiming>,
        pub protocol: Option<String>,
        pub security_state: security::SecurityState,
        pub security_details: Option<SecurityDetails>,
    }


// \"request\":{\"url\":\"https://59.202.58.131/api/league/manager/list?_=1558774807951\",
// \"method\":\"GET\",\"headers\":{\"Accept\":\"*/*\",\"Referer\":\"https://59.202.58.131/home\",\"X-Requested-With\":\"XMLHttpRequest\",\"csrftoken\":\"d6422cb0-3f0f-4019-a73c-582a0b09ee15\",\"User-Agent\":\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/74.0.3729.169 Safari/537.36\",\"Content-Type\":\"application/json;charset:utf-8\"},
// \"mixedContentType\":\"none\",\"initialPriority\":\"High\",
// \"referrerPolicy\":\"no-referrer-when-downgrade\"},
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub url: String,
    pub url_fragment: Option<String>,
    pub method: String,
    pub headers: Headers,
    pub post_data: Option<String>,
    pub has_post_data: Option<bool>,
    pub mixed_content_type: Option<String>,
    /// Loading priority of a resource request.
    /// Allow values: VeryLow, Low, Medium, High, VeryHigh
    pub initial_priority: String,
    /// The referrer policy of the request, as defined in https://www.w3.org/TR/referrer-policy/
    /// Allow values: unsafe-url, no-referrer-when-downgrade, no-referrer, origin, origin-when-cross-origin, same-origin, strict-origin, strict-origin-when-cross-origin
    pub referrer_policy: String,
    pub is_link_preload: Option<bool>,
}

pub mod events {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthChallenge {
        #[serde(skip_serializing_if = "Option::is_none")]
        /// Source of the authentication challenge. Allowed values: Server, Proxy
        pub source: Option<String>,
        pub origin: String,
        pub scheme: String,
        pub realm: String,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestInterceptedEventParams {
        pub interception_id: String,
        pub request: super::Request,
        pub frame_id: String,
        pub resource_type: String,
        pub is_navigation_request: bool,
        pub is_download: Option<bool>,
        pub redirect_url: Option<String>,
        pub auth_challenge: Option<AuthChallenge>,
        /// Network level fetch failure reason.
        /// Allow values:
        /// Failed, Aborted, TimedOut, AccessDenied, ConnectionClosed, ConnectionReset, ConnectionRefused, ConnectionAborted, ConnectionFailed, NameNotResolved, InternetDisconnected, AddressUnreachable, BlockedByClient, BlockedByResponse
        pub response_error_reason: Option<String>,
        pub response_status_code: Option<i32>,
        pub response_headers: Option<super::Headers>,
        pub request_id: Option<super::RequestId>,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestInterceptedEvent {
        pub params: RequestInterceptedEventParams,
    }

    #[test]
    fn can_parse_request_intercepted_event() {
        use crate::protocol;
        use serde_json::json;

        let json_message = json!({
             "method":"Network.requestIntercepted",
             "params":{
                 "frameId":"41AF9B7E70803C38860A845DBEB8F85F",
                 "interceptionId":"id-1",
                 "isNavigationRequest":true,
                 "request":{
                     "headers":{
                         "Accept":"text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8",
                         "Upgrade-Insecure-Requests":"1",
                         "User-Agent":"Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/72.0.3626.119 Safari/537.36"
                     },
                     "initialPriority":"VeryHigh",
                     "method":"GET",
                     "referrerPolicy":"no-referrer-when-downgrade",
                     "url":"http://127.0.0.1:38157/"
                 },
                 "resourceType":"Document"
             }
        });

        let _request =
            serde_json::from_value::<super::Request>(json_message["params"]["request"].clone())
                .unwrap();
        let _event = serde_json::from_value::<protocol::Message>(json_message).unwrap();
    }
}

pub mod methods {
    use serde::{Deserialize, Serialize};

    use crate::protocol::Method;
    use std::collections::HashMap;
    use super::*;

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct Enable {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_total_buffer_size: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_resource_buffer_size: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max_post_data_size: Option<u32>,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct EnableReturnObject {}
    impl Method for Enable {
        const NAME: &'static str = "Network.enable";
        type ReturnObject = EnableReturnObject;
    }

    #[derive(Serialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct RequestPattern {
        /// Wildcards ('*' -> zero or more, '?' -> exactly one) are allowed.
        /// Escape character is backslash. Omitting is equivalent to "*".
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url_pattern: Option<String>,
        /// Resource type as it was perceived by the rendering engine.
        ///
        /// Allowed values:
        /// Document, Stylesheet, Image, Media, Font, Script, TextTrack, XHR, Fetch, EventSource, WebSocket, Manifest, SignedExchange, Ping, CSPViolationReport, Other
        #[serde(skip_serializing_if = "Option::is_none")]
        pub resource_type: Option<ResourceType>,

        /// Stages of the interception to begin intercepting. Request will intercept before the
        /// request is sent. Response will intercept after the response is received.
        ///
        /// Allowed values:
        /// Request, HeadersReceived
        #[serde(skip_serializing_if = "Option::is_none")]
        pub interception_stage: Option<InterceptionStage>,
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct SetRequestInterception<'a> {
        pub patterns: &'a [RequestPattern],
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct SetRequestInterceptionReturnObject {}
    impl<'a> Method for SetRequestInterception<'a> {
        const NAME: &'static str = "Network.setRequestInterception";
        type ReturnObject = SetRequestInterceptionReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct AuthChallengeResponse<'a> {
        pub response: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub password: Option<&'a str>,
    }

    #[derive(Serialize, Debug, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinueInterceptedRequest<'a> {
        pub interception_id: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub error_reason: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub raw_response: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub url: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub method: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub post_data: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub headers: Option<HashMap<&'a str, &'a str>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub auth_challenge_response: Option<AuthChallengeResponse<'a>>,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct ContinueInterceptedRequestReturnObject {}
    impl<'a> Method for ContinueInterceptedRequest<'a> {
        const NAME: &'static str = "Network.continueInterceptedRequest";
        type ReturnObject = ContinueInterceptedRequestReturnObject;
    }

    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetResponseBodyForInterception<'a> {
        pub interception_id: &'a str,
    }
    #[derive(Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct GetResponseBodyForInterceptionReturnObject {
        pub body: String,
        pub base64_encoded: bool,
    }
    impl<'a> Method for GetResponseBodyForInterception<'a> {
        const NAME: &'static str = "Network.getResponseBodyForInterception";
        type ReturnObject = GetResponseBodyForInterceptionReturnObject;
    }

}
