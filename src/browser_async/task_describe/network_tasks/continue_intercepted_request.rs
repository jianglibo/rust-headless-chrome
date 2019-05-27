use super::super::{TaskDescribe, CommonDescribeFields, AsMethodCallString, TargetCallMethodTask,  HasCommonField, CanCreateMethodString,};
use crate::protocol::{network};
use failure;


#[derive(Debug, Builder, Clone)]
pub struct AuthChallengeResponse {
    pub response: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Builder, Clone)]
#[builder(setter(into))]
pub struct ContinueInterceptedRequestTask {
    pub common_fields: CommonDescribeFields,
        pub interception_id: String,
    #[builder(default = "None")]
        pub error_reason: Option<String>,
    #[builder(default = "None")]
        pub raw_response: Option<String>,
    #[builder(default = "None")]
        pub url: Option<String>,
    #[builder(default = "None")]
        pub method: Option<String>,
    #[builder(default = "None")]
        pub post_data: Option<String>,
    #[builder(default = "None")]
        pub headers: Option<std::collections::HashMap<String, String>>,
    #[builder(default = "None")]
        pub auth_challenge_response: Option<AuthChallengeResponse>,
}

impl_has_common_fields!(ContinueInterceptedRequestTask);

impl AsMethodCallString for ContinueInterceptedRequestTask {
    fn get_method_str(&self) -> Result<String, failure::Error>{
        let headers: Option<std::collections::HashMap<&str, &str>> = self.headers.as_ref().map(|hm|hm.iter().map(|(k, v)| (&**k, &**v)).collect());
        let auth_challenge_response = if let Some(acr) = &self.auth_challenge_response {
            Some(network::methods::AuthChallengeResponse {
                response: &*acr.response,
                username: if let Some(un) = acr.username.as_ref() {Some(&*un)} else {None},
                password: if let Some(pw) = acr.password.as_ref() {Some(&*pw)} else {None},
            })
        } else {
            None
        };
        let method = network::methods::ContinueInterceptedRequest {
            interception_id: self.interception_id.as_str(),
            error_reason: self.error_reason.as_ref().map(|s|&**s),
            raw_response: self.raw_response.as_ref().map(|s|&**s),
            url: self.url.as_ref().map(|s|&**s),
            method: self.method.as_ref().map(|s|&**s),
            post_data: self.post_data.as_ref().map(|s|&**s),
            headers,
            auth_challenge_response,
        };
        Ok(self.create_method_str(method))
    }
}

impl_into_task_describe!(TaskDescribe::TargetCallMethod, TargetCallMethodTask::ContinueInterceptedRequest, ContinueInterceptedRequestTask);
