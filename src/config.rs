use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub bootstrap: BootstrapConfig,
    pub crypto: CryptoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub salt_hash: String,
    pub port: u16,
    pub mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub hash_algorithm: String,
}

impl Default for Config {
    fn default() -> Self {
        let device_id = uuid::Uuid::new_v4().to_string();

        Self {
            server: ServerConfig {
                salt_hash: device_id[0..12].to_string(),
                port: 3000,
                mode: "both".to_string(),
            },
            bootstrap: BootstrapConfig {
                nodes: vec![
                    "node1.globy.io:3000".to_string(),
                    "node2.globy.io:3000".to_string(),
                ],
            },
            crypto: CryptoConfig {
                hash_algorithm: "sha256".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load_or_default(path: &Option<PathBuf>) -> Result<Self> {
        match path {
            Some(p) => {
                let contents = std::fs::read_to_string(p)?;
                let config = toml::from_str(&contents)?;
                Ok(config)
            }
            None => Ok(Config::default()),
        }
    }

    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let contents = toml::to_string_pretty(&self)?;
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
