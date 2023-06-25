use lazy_static::lazy_static;
use tokio::sync::RwLock;

use crate::gen_simple_config_manager;

static HTTP_PROXY_CONFIG: &str = "http_proxy";

gen_simple_config_manager!(
    HttpProxyConfigManager,
    HttpProxyConfig,
    HTTP_PROXY_CONFIG,
    gen_default_config
);

lazy_static! {
    pub static ref HTTP_PROXY_CONFIG_MANAGER: RwLock<HttpProxyConfigManager> =
        RwLock::new(HttpProxyConfigManager::init());
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpProxyAuthentication {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HttpProxyConfig {
    pub(crate) enable: bool,
    pub(crate) hostname: Option<String>,
    pub(crate) port: Option<i32>,
    #[serde(rename = "enableAuth")]
    pub(crate) enable_auth: bool,
    pub(crate) authentication: Option<HttpProxyAuthentication>,
}

fn gen_default_config() -> HttpProxyConfig {
    HttpProxyConfig {
        enable: false,
        hostname: None,
        port: None,
        enable_auth: false,
        authentication: None,
    }
}
