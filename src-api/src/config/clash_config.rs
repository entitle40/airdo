use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClashConfig {
    pub proxies: Vec<ClashProxiesConfig>,
    #[serde(rename = "external-controller")]
    pub external_controller: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClashProxiesConfig {
    pub name: String,
    pub r#type: String,
    pub server: String,
    #[serde(deserialize_with = "deserialize_u16")]
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sni: Option<String>,
    #[serde(rename = "skip-cert-verify", skip_serializing_if = "Option::is_none")]
    pub skip_cert_verify: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(rename = "plugin-opts", skip_serializing_if = "Option::is_none")]
    pub plugin_opts: Option<ClashProxiesPluginOptsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ClashProxiesPluginOptsConfig {
    pub mode: String,
    pub host: String,
}

fn deserialize_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    str.parse::<u16>().map_err(serde::de::Error::custom)
}