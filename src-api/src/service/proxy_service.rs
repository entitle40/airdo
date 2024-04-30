use std::time::Duration;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::{io::AsyncWriteExt as _, net::TcpStream};

use crate::{config::app_config::Config, util::{command_util, http_util}};

pub async fn start(config: &Config) -> anyhow::Result<u32> {
    tracing::debug!("启动 proxy");

    let core = config.proxy.inner.as_ref().unwrap().core.clone();

    let commands = if core.eq_ignore_ascii_case("mihomo") {
        if cfg!(windows) {
            Vec::from([
                format!(r".\mihomo.exe -d ."),
            ])
        } else {
            Vec::from([
                format!("./mihomo -d ."),
            ])
        }
    } else {
        if cfg!(windows) {
            Vec::from([
                format!(r".\sing-box.exe run -c config.json"),
            ])
        } else {
            Vec::from([
                format!("./sing-box run -c config.json"),
            ])
        }
    };
    let (pid, mut rx) = command_util::execute_async(core, commands);
    tokio::spawn(async move {
        loop {
            let msg = match rx.recv().await {
                Some(msg) => msg,
                None => break,
            };
            tracing::info!(target: "proxy", "{}", msg);
        }
    });
    
    match pid {
        Some(pid) => anyhow::Ok(pid),
        None => Err(anyhow!("启动失败")),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct ProxyApiInfo {
    pub external_controller: String,
    pub secret: String,
}

pub async fn get_all_proxy(proxy_api_info: &ProxyApiInfo) -> anyhow::Result<Vec<String>> {
    let response = http_util::get_proxy(&proxy_api_info, "/proxies/").await?;
    if !response.status().is_success() {
        return Err(anyhow!("获取所有的代理节点失败：{:?}", response));
    }
    let json = response.json::<Value>().await?;
    let all_proxy_value = json["proxies"]["GLOBAL"]["all"].as_array().unwrap();
    let mut all_proxy = vec![];
    for value in all_proxy_value {
        let value = value.as_str().unwrap().to_string();
        if value.eq_ignore_ascii_case("DIRECT") || value.eq_ignore_ascii_case("REJECT") {
            continue;
        }
        all_proxy.push(value);
    }
    anyhow::Ok(all_proxy)
}

#[derive(Debug)]
pub struct DelayStatusError {
    pub status: u16,
    pub des: String,
}

impl std::fmt::Display for DelayStatusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DelayStatusError {} {}", self.status, self.des)
    }
}

impl std::error::Error for DelayStatusError {}

impl DelayStatusError {
    fn new(status: u16, des: &str) -> Self {
        DelayStatusError {
            status,
            des: des.to_string()
        }
    }
}

pub async fn test_delay(proxy_api_info: &ProxyApiInfo, timeout: Duration, node_name: &str, test_url: &str) -> anyhow::Result<i64> {
    let response = http_util::get_proxy(&proxy_api_info, &format!("/proxies/{}/delay?timeout={}&url={}", percent_encoding::utf8_percent_encode(node_name, percent_encoding::NON_ALPHANUMERIC), timeout.as_millis(), percent_encoding::utf8_percent_encode(test_url, percent_encoding::NON_ALPHANUMERIC))).await?;
    let status = response.status();
    let delay = response.json::<Value>().await?;
    if !status.is_success() {
        tracing::debug!("测试代理延迟失败：{}，状态：{}", delay, status);
        return Err(anyhow!(DelayStatusError::new(status.as_u16(), status.canonical_reason().unwrap_or("<unknown status code>"))));
    }
    return Ok(delay["delay"].as_i64().unwrap());
}

pub async fn test_all_delay(proxy_api_info: &ProxyApiInfo, timeout: Duration, test_url: &str) -> anyhow::Result<serde_json::Map<String, Value>> {
    let response = http_util::get_proxy(&proxy_api_info, &format!("/group/proxy/delay?timeout={}&url={}", timeout.as_millis(), percent_encoding::utf8_percent_encode(test_url, percent_encoding::NON_ALPHANUMERIC))).await?;
    if !response.status().is_success() {
        tracing::error!("测试所有代理延迟失败：{:?}", response);
        return Err(anyhow!("测试所有代理延迟失败：{:?}", response));
    }
    let delay_res = response.json::<serde_json::Map<String, Value>>().await?;
    anyhow::Ok(delay_res)
}

pub async fn get_all_node(proxy_api_info: &ProxyApiInfo) -> anyhow::Result<Value> {
    let response = http_util::get_proxy(&proxy_api_info, "/proxies").await?;
    if !response.status().is_success() {
        tracing::error!("获取所有代理节点失败：{:?}", response);
        return Err(anyhow!("获取所有代理节点失败：{:?}", response));
    }
    let proxies = response.json::<Value>().await?;
    let mut json = json!({"all": [], "now": ""});
    let all = json["all"].as_array_mut().unwrap();
    let proxies = &proxies["proxies"];
    for node in proxies["proxy"]["all"].as_array().unwrap() {
        all.push(proxies[node.as_str().unwrap()].clone());
    }
    json.as_object_mut().unwrap().insert("now".to_string(), proxies["proxy"]["now"].clone());
    Ok(json)
}

pub async fn check_api_started(external_controller: &str) -> bool {
    let mut api_started = false;
    for _ in 0..5 {
        let tcp = tokio::time::timeout(
            Duration::from_secs(5),
            TcpStream::connect(external_controller)
        ).await;

        if let Ok(Ok(mut stream)) = tcp {
            let _ = stream.shutdown().await;
            api_started = true;
            break;
        }
        tracing::warn!("代理尚未启动完成，等待中...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    api_started
}
