use std::time::Duration;

use anyhow::{anyhow, Ok};
use reqwest::header::{HeaderValue, HeaderMap};

use crate::service::proxy_service::ProxyApiInfo;

const USER_AGENT_KEY: &str = "User-Agent";
const USER_AGENT_VALUE: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0";

pub async fn get_proxy(api_info: &ProxyApiInfo, url: &str) -> anyhow::Result<reqwest::Response> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT_KEY, HeaderValue::from_static(USER_AGENT_VALUE));
    headers.insert("Authorization", HeaderValue::from_str(&format!("Bearer {}", api_info.secret)).unwrap());
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let response = client.get(format!("http://{}{}", api_info.external_controller, url)).send().await?;
    Ok(response)
}

#[allow(dead_code)]
pub async fn test_delay(proxy_url: &str, url: &str, timeout: Duration) -> anyhow::Result<(Duration, reqwest::Response)> {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all(proxy_url)?)
        .build()?;
    let start = std::time::Instant::now();
    let response = client.get(url).timeout(timeout).send().await;
    let elapsed = start.elapsed();
    Ok((elapsed, response?))
}

pub async fn get_text_from_url(url: &str) -> anyhow::Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT_KEY, HeaderValue::from_static(USER_AGENT_VALUE));
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let response = client.get(url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("请求失败：{:?}", response.status()));
    }
    Ok(response.text().await?)
}

#[cfg(test)]
mod http_util_test {
    use super::*;

    #[tokio::test]
    async fn test_delay_test() {
        loop {
            let res = test_delay("socks5://127.0.0.1:10808", "https://cp.cloudflare.com", Duration::from_secs(5)).await;
            println!("{:?}", res);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

}