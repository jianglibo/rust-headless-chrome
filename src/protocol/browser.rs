pub mod methods {
    use crate::protocol::Method;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Debug)]
    pub struct GetVersion {}
    #[derive(Deserialize, Clone, Debug)]
    #[serde(rename_all = "camelCase")]
    /// Version information returned by `Browser.getVersion`
    pub struct VersionInformationReturnObject {
        /// Protocol version
        pub protocol_version: String,
        /// Product version
        pub product: String,
        /// Product revision
        pub revision: String,
        /// User-Agent
        pub user_agent: String,
        /// V8 version.
        pub js_version: String,
    }
    impl Method for GetVersion {
        const NAME: &'static str = "Browser.getVersion";
        type ReturnObject = VersionInformationReturnObject;
    }


    #[derive(Serialize, Debug)]
    #[serde(rename_all = "camelCase")]
    pub struct GetBrowserCommandLine {}
    #[derive(Debug, Deserialize, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct GetBrowserCommandLineReturnObject {
        pub arguments: Vec<String>,
    }

    impl Method for GetBrowserCommandLine {
        const NAME: &'static str = "Browser.getBrowserCommandLine";
        type ReturnObject = GetBrowserCommandLineReturnObject;
    }
}
