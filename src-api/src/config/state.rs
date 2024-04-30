use sqlx::{Pool, Sqlite};

use crate::service::proxy_service::ProxyApiInfo;

#[derive(Clone)]
pub struct AppState {
    pub config: crate::config::app_config::Config,
    pub db_pool: Pool<Sqlite>,
    pub proxy_api_info: Option<ProxyApiInfo>,
    pub all_proxy: Option<Vec<String>>,
}