use std::collections::HashMap;

use reqwest::Url;

use crate::{config::{app_config::{Config, InnerProxyConfig}, clash_config::{ClashConfig, ClashProxiesConfig, ClashProxiesPluginOptsConfig}, sing_box_config::{SingBoxConfig, SingBoxExperimentalCacheFileConfig, SingBoxExperimentalClashApiConfig, SingBoxExperimentalConfig, SingBoxLogConfig, SingBoxOutboundsConfig, SingBoxTlsConfig}}, util::{base64_util, file_util, http_util}};

pub async fn pull_subscriptions(config: &Config) -> anyhow::Result<()> {
    tracing::info!("拉取订阅开始");
    let inner_proxy_config = config.proxy.inner.as_ref().unwrap();
    if inner_proxy_config.airport.url_type.eq_ignore_ascii_case("Clash") {
        pull_clash_subscriptions(&inner_proxy_config).await
    } else {
        pull_universal_subscriptions(&inner_proxy_config).await
    }
}

async fn pull_clash_subscriptions(inner_proxy_config: &InnerProxyConfig) -> anyhow::Result<()> {
    tracing::info!("正在打开订阅链接，拉取 clash 订阅");
    let url_text = http_util::get_text_from_url(&inner_proxy_config.airport.url).await?;
    tracing::info!("拉取成功，开始解析 yaml");
    let mut clash_config = serde_yaml::from_str::<ClashConfig>(&url_text)?;
    tracing::info!("解析成功，开始编写配置文件");
    if inner_proxy_config.core.eq_ignore_ascii_case("mihomo") {
        clash_config.external_controller = inner_proxy_config.controller.clone();
        clash_config.secret = Some(inner_proxy_config.secret.clone());

        let path = file_util::get_current_dir().join("mihomo/config.yaml");
        let _ = file_util::create_file_if_not_exist(path.clone());
        file_util::write_file(path, &serde_yaml::to_string(&clash_config).unwrap());
    } else {
        let mut sing_box_config = SingBoxConfig::default();
        sing_box_config.log = SingBoxLogConfig {
            level: "info".to_string(),
        };
        sing_box_config.experimental = SingBoxExperimentalConfig {
            cache_file: SingBoxExperimentalCacheFileConfig {
                enabled: true,
            },
            clash_api: SingBoxExperimentalClashApiConfig {
                external_controller: inner_proxy_config.controller.clone(),
                secret: inner_proxy_config.secret.clone(),
            }
        };
        sing_box_config.outbounds = vec![];
        for proxie in clash_config.proxies {
            let mut ob = SingBoxOutboundsConfig::default();
            let name = proxie.name;
            ob.server = proxie.server.clone();
            ob.server_port = proxie.port;
            ob.tag = name.clone();
            ob.password = proxie.password;
            if proxie.r#type == "ss" {
                ob.r#type = "shadowsocks".to_string();
                ob.method = proxie.cipher;
                if let Some(plugin) = proxie.plugin {
                    if plugin == "obfs" {
                        ob.plugin = Some("obfs-local".to_string());
                        if let Some(plugin_opts) = proxie.plugin_opts {
                            ob.plugin_opts = Some(format!("obfs={};obfs-host={}", plugin_opts.mode, plugin_opts.host));
                        }
                    }
                }
            } else if proxie.r#type == "trojan" {
                ob.r#type = "trojan".to_string();
                ob.tls = Some(SingBoxTlsConfig {
                    enabled: true,
                    server_name: proxie.sni.unwrap_or(proxie.server),
                    insecure: proxie.skip_cert_verify.is_some() && proxie.skip_cert_verify.unwrap(),
                });
            }
            sing_box_config.outbounds.push(ob);
            tracing::info!("{} 节点已添加", &name);
        }
        let path = file_util::get_current_dir().join("sing-box/config.json");
        let _ = file_util::create_file_if_not_exist(path.clone());
        file_util::write_file(path, &serde_json::to_string(&sing_box_config).unwrap());
    }
    tracing::info!("拉取订阅完成");
    anyhow::Ok(())
}

async fn pull_universal_subscriptions(inner_proxy_config: &InnerProxyConfig) -> anyhow::Result<()> {
    tracing::info!("正在打开订阅链接，拉取通用订阅");
    let url_text = http_util::get_text_from_url(&inner_proxy_config.airport.url).await?;
    tracing::info!("拉取成功，开始 base64 解码");
    let base64text = base64_util::decode(&url_text)?;
    let text = String::from_utf8(base64text)?;
    tracing::info!("解码成功，开始遍历添加节点");
    let lines = text.split("\n");
    if inner_proxy_config.core.eq_ignore_ascii_case("mihomo") {
        let mut clash_config = ClashConfig::default();
        clash_config.external_controller = inner_proxy_config.controller.clone();
        clash_config.secret = Some(inner_proxy_config.secret.clone());
        clash_config.proxies = vec![];
        for line in lines {
            if line.is_empty() {
                continue;
            }
            let url = Url::parse(line)?;
            let name = percent_encoding::percent_decode_str(url.fragment().unwrap()).decode_utf8()?;
            let name = name.trim();
            let mut cpc = ClashProxiesConfig::default();
            cpc.server = url.host_str().unwrap().to_string();
            cpc.port = url.port().unwrap();
            cpc.name = name.to_string();
            let mut password = url.username();
            if password.is_empty() {
                password = url.password().unwrap();
            }
            let query_pairs = url.query_pairs();
            let params: HashMap<String, String> = query_pairs.into_iter()
                .filter_map(|(key, value)| {
                    if key.is_empty() {
                        return None;
                    }
                    Some((key.to_string(), value.to_string()))
                }).collect::<HashMap<std::string::String, std::string::String>>();
            if url.scheme() == "ss" {
                cpc.r#type = "shadowsocks".to_string();
                let decode_password = base64_util::decode(password)?;
                let pwd = String::from_utf8(decode_password)?;
                let split = pwd.split(":").collect::<Vec<&str>>();
                cpc.cipher = Some(split.get(0).unwrap().to_string());
                cpc.password = Some(split.get(1).unwrap().to_string());
                if let Some(plugin) = params.get("plugin") {
                    let plugin = percent_encoding::percent_decode_str(plugin).decode_utf8()?.to_string();
                    let plugin = plugin.split(";").collect::<Vec<&str>>();
                    let plugin1 = plugin[1].split("=").collect::<Vec<&str>>();
                    let plugin2 = plugin[2].split("=").collect::<Vec<&str>>();
                    let mut map: HashMap<String, String> = HashMap::new();
                    map.insert(plugin1[0].to_string(), plugin1[1].to_string());
                    map.insert(plugin2[0].to_string(), plugin2[1].to_string());
                    cpc.plugin = Some(plugin[0].to_string());
                    cpc.plugin_opts = Some(ClashProxiesPluginOptsConfig {
                        mode: map.get("obfs").unwrap().to_string(),
                        host: map.get("obfs-host").unwrap().to_string(),
                    });
                }
            } else if url.scheme() == "trojan" {
                cpc.r#type = "trojan".to_string();
                cpc.password = Some(password.to_string());
                cpc.sni = Some(params.get("sni").unwrap_or(&cpc.server.clone()).to_string());
                let tls_insecure = (params.get("tls").is_some() && params.get("tls").unwrap() == "false") || (params.get("allowInsecure").is_some() && params.get("allowInsecure").unwrap() == "1");
                cpc.skip_cert_verify = Some(tls_insecure);
            }
            clash_config.proxies.push(cpc);
            tracing::info!("{} 节点已添加", &name);
        }
        let path = file_util::get_current_dir().join("mihomo/config.yaml");
        let _ = file_util::create_file_if_not_exist(path.clone());
        file_util::write_file(path, &serde_yaml::to_string(&clash_config).unwrap());
    } else {
        let mut sing_box_config = SingBoxConfig::default();
        sing_box_config.log = SingBoxLogConfig {
            level: "info".to_string(),
        };
        sing_box_config.experimental = SingBoxExperimentalConfig {
            cache_file: SingBoxExperimentalCacheFileConfig {
                enabled: true,
            },
            clash_api: SingBoxExperimentalClashApiConfig {
                external_controller: inner_proxy_config.controller.clone(),
                secret: inner_proxy_config.secret.clone(),
            }
        };
        for line in lines {
            if line.is_empty() {
                continue;
            }
            // index += 1;
            let url = Url::parse(line)?;
            let name = percent_encoding::percent_decode_str(url.fragment().unwrap()).decode_utf8()?;
            let name = name.trim();
            let mut oc = SingBoxOutboundsConfig::default();
            oc.server = url.host_str().unwrap().to_string();
            oc.server_port = url.port().unwrap();
            // oc.name = Some(format!("{:03} {}", index, name));
            oc.tag = name.to_string();
            let mut password = url.username();
            if password.is_empty() {
                password = url.password().unwrap();
            }
            let query_pairs = url.query_pairs();
            let params: HashMap<String, String> = query_pairs.into_iter()
                .filter_map(|(key, value)| {
                    if key.is_empty() {
                        return None;
                    }
                    Some((key.to_string(), value.to_string()))
                }).collect::<HashMap<std::string::String, std::string::String>>();
            if url.scheme() == "ss" {
                oc.r#type = "shadowsocks".to_string();
                let decode_password = base64_util::decode(password)?;
                let pwd = String::from_utf8(decode_password)?;
                let split = pwd.split(":").collect::<Vec<&str>>();
                oc.method = Some(split.get(0).unwrap().to_string());
                oc.password = Some(split.get(1).unwrap().to_string());
                if let Some(plugin) = params.get("plugin") {
                    let plugin = percent_encoding::percent_decode_str(plugin).decode_utf8()?.to_string();
                    if let Some(plugin) = plugin.split_once(";") {
                        oc.plugin = Some(plugin.0.to_string());
                        oc.plugin_opts = Some(plugin.1.to_string());
                    }
                }
            } else if url.scheme() == "trojan" {
                oc.r#type = "trojan".to_string();
                oc.password = Some(password.to_string());
                let tls_insecure = (params.get("tls").is_some() && params.get("tls").unwrap() == "false") || (params.get("allowInsecure").is_some() && params.get("allowInsecure").unwrap() == "1");
                oc.tls = Some(SingBoxTlsConfig {
                    enabled: true,
                    insecure: tls_insecure,
                    server_name: params.get("sni").unwrap_or(&oc.server.clone()).to_string(),
                });
            }
            sing_box_config.outbounds.push(oc);
            tracing::info!("{} 节点已添加", &name);
        }
        let path = file_util::get_current_dir().join("sing-box/config.json");
        let _ = file_util::create_file_if_not_exist(path.clone());
        file_util::write_file(path, &serde_json::to_string(&sing_box_config).unwrap());
    }
    tracing::info!("更新订阅完成");
    anyhow::Ok(())
}
