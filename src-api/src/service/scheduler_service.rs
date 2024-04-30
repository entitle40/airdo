use std::time::Duration;

use anyhow::Ok;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::config::state::AppState;

use super::health_check_service;

pub async fn init(app_state: &AppState) -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    let all_proxy = app_state.all_proxy.as_ref().unwrap().clone();
    for proxy_node in all_proxy {
        let app_state_clone = app_state.clone();
        sched.add(Job::new_repeated_async(Duration::from_secs(app_state_clone.config.health_check.interval_time), move |_uuid, _l| {
            let app_state = app_state_clone.clone();
            let proxy_node = proxy_node.clone();
            Box::pin(async move {
                let res = health_check_service::check(&app_state, &proxy_node).await;
                if res.is_err() {
                    tracing::error!("测试延迟错误：{:?}", res);
                }
            })
        })?).await?;
    }

    sched.start().await?;
    Ok(())
}
