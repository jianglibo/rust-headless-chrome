use super::super::super::embedded_events;
use super::super::TaskDescribe;
use super::NetworkEvent;
use crate::protocol::network;
use std::collections::HashMap;
use failure;

lazy_static! {
    static ref STATUS_TO_STATUS_TEXT: HashMap<i32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(100, "Continue");
        m.insert(101, "Switching Protocols");
        m.insert(200, "OK");
        m.insert(201, "Created");
        m.insert(202, " Accepted");
        m.insert(203, "Non-Authoritative Information");
        m.insert(204, "No Content");
        m.insert(205, "Reset Content");
        m.insert(206, "Partial Content");
        m.insert(300, "Multiple Choices");
        m.insert(301, "Moved Permanently");
        m.insert(302, "Found");
        m.insert(303, "See Other");
        m.insert(304, "Not Modified");
        m.insert(305, "Use Proxy");
        m.insert(307, "Temporary Redirect");
        m.insert(400, "Bad Request");
        m.insert(401, "Unauthorized");
        m.insert(402, "Payment Required");
        m.insert(403, "Forbidden");
        m.insert(404, "Not Found");
        m.insert(405, "Method Not Allowed");
        m.insert(406, "Not Acceptable");
        m.insert(407, "Proxy Authentication Required");
        m.insert(408, "Request Time-out");
        m.insert(409, "Conflict");
        m.insert(410, "Gone");
        m.insert(411, "Length Required");
        m.insert(412, "Precondition Failed");
        m.insert(413, "Request Entity Too Large");
        m.insert(414, "Request-URI Too Large");
        m.insert(415, "Unsupported Media Type");
        m.insert(416, "Requested range not satisfiable");
        m.insert(417, "Expectation Failed");
        m.insert(500, "Internal Server Error");
        m.insert(501, "Not Implemented");
        m.insert(502, "Bad Gateway");
        m.insert(503, "Service Unavailable");
        m.insert(504, "Gateway Time-out");
        m.insert(505, "HTTP Version not supported");
        m
    };
}

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
    pub fn get_raw_parameters(&self) -> &embedded_events::ResponseReceivedParams {
        &self.raw_event.params
    }

    pub fn get_request_id(&self) -> network::RequestId {
        self.raw_event.params.request_id.clone()
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

    pub fn construct_raw_response_from_response(&self, saved_response: Option<&ResponseReceived>, decoded_body_string: Result<&str, &failure::Error>) -> String {
        let body = decoded_body_string.expect("body should exists.");
        if let Some(saved_response) = saved_response {
            self.construct_raw_response(saved_response.get_raw_parameters().response.status, &saved_response.get_raw_parameters().response.status_text, body)
        } else {
            let code = self.get_raw_parameters().response_status_code.unwrap_or(200);
            self.construct_raw_response(code , STATUS_TO_STATUS_TEXT.get(&code).expect("status should has matching status text."), body)
        }
    }

    /// https://www.w3.org/Protocols/rfc2616/rfc2616-sec6.html
    pub fn construct_raw_response(&self, status: i32, status_text: &str, decoded_body_string: &str) -> String {
        let rd = self.get_raw_parameters();
        let status_line = format!("{} {} {}\r\n", "HTTP/1.1", status, status_text);
        let header_string = if let Some(headers) = &rd.response_headers {
            let mut ss: Vec<String> = headers.iter().map(|(k,v)|format!("{}:{}\r\n", k, v)).collect();
            ss.sort();
            ss.join("")
        } else {
            String::from("")
        };

        let status_headers = format!("{}{}\r\n{}", status_line, header_string, decoded_body_string);
        base64::encode(&status_headers)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use crate::browser_async::task_describe::network_events;
    use crate::protocol;
    use log::*;
    #[test]
    fn test_construct_raw_response() {
        let msg = "{\"method\":\"Target.receivedMessageFromTarget\",\"params\":{\"sessionId\":\"7E85CA49A2D740675BCBFB3CDF778050\",\"message\":\"{\\\"method\\\":\\\"Network.requestIntercepted\\\",\\\"params\\\":{\\\"interceptionId\\\":\\\"interception-job-4.0\\\",\\\"request\\\":{\\\"url\\\":\\\"https://59.202.58.131/api/org/getTreeOrg?orgId=1&deep=1&_=1559563518235\\\",\\\"method\\\":\\\"GET\\\",\\\"headers\\\":{\\\"Accept\\\":\\\"application/json, text/javascript, */*; q=0.01\\\",\\\"X-Requested-With\\\":\\\"XMLHttpRequest\\\",\\\"csrftoken\\\":\\\"4a6cb39a-7f3d-4298-83b8-a1b793718f90\\\",\\\"User-Agent\\\":\\\"Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/74.0.3729.169 Safari/537.36\\\",\\\"Referer\\\":\\\"https://59.202.58.131/orgstructure/orgstructure-manage?orgId=100016626\\\",\\\"Cookie\\\":\\\"sid=eaecfb55-be67-407d-b8be-1cff6ee2ba7d; UAASESSIONID=63d2b60f-ff65-4c0b-9142-d3dc381ad3c0; refresh=false\\\"},\\\"initialPriority\\\":\\\"High\\\",\\\"referrerPolicy\\\":\\\"no-referrer-when-downgrade\\\"},\\\"frameId\\\":\\\"5FC90EF2AED61BEE38D00DC041BFB856\\\",\\\"resourceType\\\":\\\"XHR\\\",\\\"isNavigationRequest\\\":false,\\\"isDownload\\\":false,\\\"responseStatusCode\\\":200,\\\"responseHeaders\\\":{\\\"Server\\\":\\\"nginx/1.10.2\\\",\\\"Date\\\":\\\"Mon, 03 Jun 2019 12:06:01 GMT\\\",\\\"Content-Type\\\":\\\"application/json;charset=utf-8\\\",\\\"Content-Length\\\":\\\"2660\\\",\\\"Connection\\\":\\\"keep-alive\\\",\\\"X-Application-Context\\\":\\\"dingplus-user-web-dubbo:localtest,menu,privilege\\\"}}}\",\"targetId\":\"5FC90EF2AED61BEE38D00DC041BFB856\"}}";
        let ev = protocol::parse_raw_message(&msg).expect("got message.");
        let mut message_parsed = false;
        if let protocol::Message::Event(protocol::Event::ReceivedMessageFromTarget(
            target_message_event,
        )) = ev
        {
            let message_field = &target_message_event.params.message;
            let parsed_ev =
                protocol::parse_raw_message(&message_field).expect("it's intercepted event.");
            if let protocol::Message::Event(protocol_event) = parsed_ev {
                if let protocol::Event::RequestIntercepted(raw_event) = protocol_event {
                    let rice = network_events::RequestIntercepted::new(raw_event);
                    let rs = rice.construct_raw_response(200, "OK", "abc");
                    assert_eq!(rs, String::from("SFRUUC8xLjEgMjAwIE9LDQpYLUFwcGxpY2F0aW9uLUNvbnRleHQ6ZGluZ3BsdXMtdXNlci13ZWItZHViYm86bG9jYWx0ZXN0LG1lbnUscHJpdmlsZWdlDQpDb250ZW50LUxlbmd0aDoyNjYwDQpEYXRlOk1vbiwgMDMgSnVuIDIwMTkgMTI6MDY6MDEgR01UDQpDb25uZWN0aW9uOmtlZXAtYWxpdmUNCkNvbnRlbnQtVHlwZTphcHBsaWNhdGlvbi9qc29uO2NoYXJzZXQ9dXRmLTgNClNlcnZlcjpuZ2lueC8xLjEwLjINCg0Kabc"));
                    message_parsed = true;
                }
            }
        }
        assert!(message_parsed);
    }
}
