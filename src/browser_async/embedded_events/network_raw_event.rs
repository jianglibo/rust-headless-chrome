use serde::Deserialize;
use crate::protocol::{network, page};

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestWillBeSent {
    pub params: RequestWillBeSentParams,
}

// "{\"method\":\"Network.requestWillBeSent\",\"params\":{\"requestId\":\"1000022340.129\",
// \"loaderId\":\"B437570341CB0E65C27EB311E43BD1C4\",\"documentURL\":\"https://59.202.58.131/home\",
// \"request\":{\"url\":\"https://59.202.58.131/api/league/manager/list?_=1558774807951\",\"method\":\"GET\",\"headers\":{\"Accept\":\"*/*\",\"Referer\":\"https://59.202.58.131/home\",\"X-Requested-With\":\"XMLHttpRequest\",\"csrftoken\":\"d6422cb0-3f0f-4019-a73c-582a0b09ee15\",\"User-Agent\":\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/74.0.3729.169 Safari/537.36\",\"Content-Type\":\"application/json;charset:utf-8\"},\"mixedContentType\":\"none\",\"initialPriority\":\"High\",\"referrerPolicy\":\"no-referrer-when-downgrade\"},
// \"timestamp\":602350.487155,\"wallTime\":1558774808.11041,
// \"initiator\":{\"type\":\"script\",\"stack\":{\"callFrames\":[{\"functionName\":\"send\",\"scriptId\":\"42\",\"url\":\"https://59.202.58.131/assets/scripts/vendor-daf58f8629.js\",\"lineNumber\":84,\"columnNumber\":3511}]}},
// \"type\":\"XHR\",\"frameId\":\"D8ACD4C37323FA44FAA676C48987E694\",\"hasUserGesture\":false}}"

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RequestWillBeSentParams {
    pub request_id: network::RequestId,
    pub loader_id: network::LoaderId,
    #[serde(rename = "documentURL")]
    pub document_url: String,
    pub request: network::Request,
    pub timestamp: network::MonotonicTime,
    pub wall_time: network::TimeSinceEpoch,
    pub initiator: network::Initiator,
    pub redirect_response: Option<network::Response>,
    #[serde(rename = "type")]
    pub resource_type: Option<network::ResourceType>,
    pub frame_id: Option<page::FrameId>,
    pub has_user_gesture: Option<bool>,
}


#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseReceived {
    pub params: ResponseReceivedParams,
}
// "{\"method\":\"Network.responseReceived\",\"params\":{\"requestId\":\"1000009324.131\",\"loaderId\":\"C22D55F00A3F0778809CE5336D1DC450\",
// \"timestamp\":602980.547859,\"type\":\"XHR\",
// \"response\":{\"url\":\"https://59.202.58.131/api/assess/getAssessDataOfFirstIndex?curDate=2019%2F05%2F24&orgId=1&_=1558775437908\",\"status\":200,\"statusText\":\"OK\",\"headers\":{\"Date\":\"Sat, 25 May 2019 09:18:22 GMT\",\"Server\":\"nginx/1.6.3\",\"Connection\":\"keep-alive\",\"Content-Length\":\"1325\",\"X-Application-Context\":\"dingplus-user-web-dubbo:localtest,menu,privilege\",\"Content-Type\":\"application/json;charset=utf-8\"},\"mimeType\":\"application/json\",\"connectionReused\":true,\"connectionId\":395,\"remoteIPAddress\":\"59.202.58.131\",\"remotePort\":443,\"fromDiskCache\":false,\"fromServiceWorker\":false,\"encodedDataLength\":242,\"timing\":{\"requestTime\":602980.457097,\"proxyStart\":-1,\"proxyEnd\":-1,\"dnsStart\":-1,\"dnsEnd\":-1,\"connectStart\":-1,\"connectEnd\":-1,\"sslStart\":-1,\"sslEnd\":-1,\"workerStart\":-1,\"workerReady\":-1,\"sendStart\":0.436,\"sendEnd\":0.494,\"pushStart\":0,\"pushEnd\":0,\"receiveHeadersEnd\":89.751},\"protocol\":\"http/1.1\",\"securityState\":\"insecure\",\"securityDetails\":{\"protocol\":\"TLS 1.2\",\"keyExchange\":\"ECDHE_RSA\",\"keyExchangeGroup\":\"P-256\",\"cipher\":\"AES_128_GCM\",\"certificateId\":0,\"subjectName\":\"\",\"sanList\":[],\"issuer\":\"\",\"validFrom\":1481188811,\"validTo\":1796548811,\"signedCertificateTimestampList\":[],\"certificateTransparencyCompliance\":\"unknown\"}},
// \"frameId\":\"BAE1B46F570F91C8B3858CCB2F8BB9A4\"}}"
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResponseReceivedParams {
    pub request_id: network::RequestId,
    pub loader_id: network::LoaderId,
    pub timestamp: network::MonotonicTime,
    #[serde(rename = "type")]
    pub resource_type: network::ResourceType,
    pub response: network::Response,
    pub frame_id: Option<page::FrameId>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataReceived {
    pub params: DataReceivedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataReceivedParams {
    pub request_id: network::RequestId,
    pub timestamp: network::MonotonicTime,
    pub data_length: u32,
    pub encoded_data_length: u32,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadingFinished {
    pub params: LoadingFinishedParams,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct LoadingFinishedParams {
    pub request_id: network::RequestId,
    pub timestamp: network::MonotonicTime,
    pub encoded_data_length: u32,
    pub should_report_corb_blocking: Option<bool>,
}