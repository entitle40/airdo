use std::time::Duration;

use axum::{
    extract::State,
    response::IntoResponse, Json,
};
use serde::{Deserialize, Serialize};

use crate::{config::state::AppState, mapper::health_check_mapper, service::proxy_service, util::response_util::ApiResponse};

pub async fn all_delay(State(app_state): State<AppState>) -> impl IntoResponse {
    let res = proxy_service::test_all_delay(&app_state.proxy_api_info.unwrap(), Duration::from_secs(app_state.config.health_check.connect_timeout), &app_state.config.health_check.test_url).await;
    if res.is_err() {
        return ApiResponse::error(&format!("延迟测试失败：{:?}", res.err().unwrap()));
    }
    ApiResponse::ok_data(res.unwrap())
}

pub async fn get_all_node(State(app_state): State<AppState>) -> impl IntoResponse {
    let res = proxy_service::get_all_node(&app_state.proxy_api_info.unwrap()).await;
    if res.is_err() {
        return ApiResponse::error(&format!("获取所有节点失败：{:?}", res.err().unwrap()));
    }
    ApiResponse::ok_data(res.unwrap())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ListHealthCheckParam {
    start_time: i64,
    end_time: i64,
}

pub async fn list_health_check(State(app_state): State<AppState>, body: Json<ListHealthCheckParam>) -> impl IntoResponse {
    let res = health_check_mapper::list(body.start_time, body.end_time, &app_state.db_pool).await;
    if res.is_err() {
        return ApiResponse::error(&res.err().unwrap().to_string());
    }
    ApiResponse::ok_data(res.unwrap())
}
