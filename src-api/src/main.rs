use crate::service::proxy_service::ProxyApiInfo;

mod config;
mod controller;
mod service;
mod util;
mod mapper;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    config::log::init();

    service::signal_service::handle();

    let config = config::app_config::get_config();
    tracing::debug!("Read Config: {:#?}", &config);

    tracing::info!("airdo {} {} {}", service::version_service::get_build_version(), service::version_service::get_build_time(), service::version_service::get_git_sha());

    let db_pool = config::db::init().await?;
    config::db::clear(&config, &db_pool).await;

    let mut app_state = config::state::AppState {
        config: config.clone(),
        db_pool,
        proxy_api_info: None,
        all_proxy: None,
    };

    if config.proxy.enabled {
        let proxy_api_info = if config.proxy.inner.is_some() && !config.proxy.external.as_ref().unwrap().enabled {
            tracing::info!("使用内置代理应用");
            service::subscriptions_service::pull_subscriptions(&config).await?;
            let pid = service::proxy_service::start(&config).await;
            if pid.is_err() {
                tracing::error!("proxy 启动失败 {:?}", &pid);
            }
            ProxyApiInfo {
                external_controller: config.proxy.inner.as_ref().unwrap().controller.clone(),
                secret: config.proxy.inner.as_ref().unwrap().secret.clone(),
            }
        } else {
            tracing::info!("使用外置代理应用");
            ProxyApiInfo {
                external_controller: config.proxy.external.as_ref().unwrap().external_controller.clone(),
                secret: config.proxy.external.as_ref().unwrap().secret.clone(),
            }
        };
        if !service::proxy_service::check_api_started(&proxy_api_info.external_controller).await {
            tracing::error!("代理启动失败，请检查代理配置是否有误");
            return anyhow::Ok(());
        }
        let all_proxy = service::proxy_service::get_all_proxy(&proxy_api_info).await;
        if all_proxy.is_err() {
            tracing::error!("获取所有的代理节点失败：{:?}", all_proxy.err());
            std::process::exit(1);
        }
    
        app_state.proxy_api_info = Some(proxy_api_info);
        app_state.all_proxy = Some(all_proxy?);
    
        service::scheduler_service::init(&app_state).await?;    
    }

    let router = config::route::init(app_state).await;

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", &config.server.bind, &config.server.port)).await?;
    tracing::info!("listening on {:?}", listener);
    axum::serve(listener, router).await
        .unwrap_or_else(|e| {
            panic!("start service fail {:#?}", e)
        });

    anyhow::Ok(())
}
