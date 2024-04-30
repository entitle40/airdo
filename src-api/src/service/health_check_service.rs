use tokio::{
    self,
    time::Duration,
};
use crate::config::state::AppState;
use crate::mapper::health_check_mapper;
use crate::service::proxy_service;

pub async fn check(app_state: &AppState, node_name: &str) -> anyhow::Result<()> {
    let mut entity = health_check_mapper::HealthCheck {
        request_time: Some(chrono::Local::now().timestamp()),
        node_name: Some(node_name.to_string()),
        ..Default::default()
    };
    let delay = proxy_service::test_delay(&app_state.proxy_api_info.as_ref().unwrap(), Duration::from_secs(app_state.config.health_check.connect_timeout), node_name, &app_state.config.health_check.test_url).await;
    match delay {
        Ok(delay_ms) => {
            entity.status_code = Some(200);
            entity.status_des = Some("Success".to_string());
            entity.delay_ms = Some(delay_ms as i32);
        },
        Err(e) => {
            if e.is::<super::proxy_service::DelayStatusError>() {
                let delay_status = e.downcast_ref::<super::proxy_service::DelayStatusError>().unwrap();
                entity.status_code = Some(delay_status.status);
                entity.status_des = Some(delay_status.des.clone());
                entity.delay_ms = Some(-1);
            } else {
                entity.status_code = Some(0);
                entity.status_des = Some(e.to_string());
                entity.delay_ms = Some(-2);
                return Err(e);
            }
        },
    }
    entity.create_time = Some(chrono::Local::now().timestamp());
    let _ = health_check_mapper::create(entity, &app_state.db_pool).await?;
    anyhow::Ok(())
}
