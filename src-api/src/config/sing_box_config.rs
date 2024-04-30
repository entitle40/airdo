use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxConfig {
    pub outbounds: Vec<SingBoxOutboundsConfig>,
    pub experimental: SingBoxExperimentalConfig,
    pub log: SingBoxLogConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxLogConfig {
    pub level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxExperimentalCacheFileConfig {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxExperimentalClashApiConfig {
    pub external_controller: String,
    pub secret: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxExperimentalConfig {
    pub cache_file: SingBoxExperimentalCacheFileConfig,
    pub clash_api: SingBoxExperimentalClashApiConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxOutboundsConfig {
    pub r#type: String,
    pub tag: String,
    pub server: String,
    #[serde(deserialize_with = "deserialize_u16")]
    pub server_port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_opts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<SingBoxTlsConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct SingBoxTlsConfig {
    pub enabled: bool,
    pub server_name: String,
    pub insecure: bool,
}

fn deserialize_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    let str: String = Deserialize::deserialize(deserializer)?;
    str.parse::<u16>().map_err(serde::de::Error::custom)
}