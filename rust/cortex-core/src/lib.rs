pub mod hardware;
pub mod memory;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BundleManifest {
    pub package: PackageInfo,
    #[serde(default)]
    pub agents: Vec<AgentInfo>,
    #[serde(default)]
    pub models: Vec<ModelInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    #[serde(default)]
    pub allow_network: bool,
    #[serde(default)]
    pub allowed_ips: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentInfo {
    pub name: String,
    pub entry_point: String,
    #[serde(default)]
    pub allow_network: bool,
    #[serde(default)]
    pub allowed_ips: Vec<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModelArchitecture {
    Llama,
    Bloom,
    Mistral,
    Falcon,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub path: String,
    pub architecture: Option<ModelArchitecture>,
    pub quantization: Option<String>,
    pub vocab_size: Option<usize>,
    pub checksum: Option<String>,
}
