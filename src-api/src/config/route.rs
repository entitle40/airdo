use std::any::Any;

use axum::http::Response;
use axum::response::IntoResponse;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tower_http::services::ServeDir;
use tower_http::catch_panic::CatchPanicLayer;
use crate::controller::proxy_controller;
use crate::config::state::AppState;
use crate::util::response_util::ApiResponse;

pub async fn init(app_state: AppState) -> Router {
    let proxy = Router::new()
        .route("/all_delay", get(proxy_controller::all_delay))
        .route("/get_all_node", get(proxy_controller::get_all_node))
        .route("/list_health_check", post(proxy_controller::list_health_check));

    let api = Router::new()
        .nest("/proxy", proxy);

    let auth = app_state.config.server.auth.clone();
    let web = app_state.config.server.web.clone();

    let mut router = Router::new()
        .nest("/api", api)
        .nest_service("/", ServeDir::new(web));
    if auth.is_some() {
        let auth = auth.unwrap();
        router = router.layer(ValidateRequestHeaderLayer::basic(&auth.username, &auth.password));
    }
    router
        .layer(CatchPanicLayer::custom(handle_panic))
        .with_state(app_state)
}

fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response<axum::body::Body> {
    let details = if let Some(s) = err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "Unknown panic message".to_string()
    };

    ApiResponse::<()>::error(&details).into_response()
}