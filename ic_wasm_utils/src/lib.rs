use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use once_cell::sync::Lazy;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuildError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command execution failed: {0}")]
    Command(String),
    #[error("Missing required file: {0}")]
    MissingFile(String),
    #[error("Configuration error: {0}")]
    Config(String),
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanisterConfig {
    pub wasm_file: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExternalCanisters {
    pub url_template: String,
    pub canisters: HashMap<String, CanisterConfig>,
}

static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    dirs::cache_dir()
        .expect("Failed to get cache dir")
        .join("ic_wasm_cache")
});

static CONFIG: Lazy<ExternalCanisters> = Lazy::new(|| {
    let config_path = find_config()
        .expect("Failed to find external_canisters.toml");
    let content = fs::read_to_string(config_path)
        .expect("Failed to read external_canisters.toml");
    toml::from_str(&content)
        .expect("Failed to parse external_canisters.toml")
});

fn find_config() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().ok()?;
    loop {
        let config_path = current_dir.join("external_canisters.toml");
        if config_path.exists() {
            return Some(config_path);
        }
        if !current_dir.pop() {
            break;
        }
    }
    None
}

pub struct ExternalWasm;

impl ExternalWasm {
    pub fn get(name: &str) -> Vec<u8> {
        Self::try_get(name).unwrap_or_else(|e| panic!("Failed to load {}: {}", name, e))
    }

    pub fn try_get(name: &str) -> Result<Vec<u8>, BuildError> {
        let config = CONFIG.canisters.get(name)
            .ok_or_else(|| BuildError::Config(format!("Canister '{}' not found in config", name)))?;

        let cache_path = CACHE_DIR.join(format!("{}-{}.wasm", config.wasm_file, config.version));

        if !cache_path.exists() {
            fs::create_dir_all(&*CACHE_DIR)?;

            let url = CONFIG.url_template
                .replace("{version}", &config.version)
                .replace("{wasm_file}", &config.wasm_file);

            println!("cargo:warning=Downloading {} from {}", config.wasm_file, url);

            let response = reqwest::blocking::get(&url)
                .map_err(|e| BuildError::Config(format!("Failed to download: {}", e)))?;

            let content = response.bytes()
                .map_err(|e| BuildError::Config(format!("Failed to read response: {}", e)))?;

            // Verify SHA256
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(&content);
            let hash = format!("{:x}", hasher.finalize());

            if hash != config.sha256 {
                return Err(BuildError::Config(format!(
                    "SHA256 mismatch for {}: expected {}, got {}",
                    config.wasm_file, config.sha256, hash
                )));
            }

            fs::write(&cache_path, content)?;
        }

        Ok(fs::read(&cache_path)?)
    }

    pub fn list() -> Vec<String> {
        CONFIG.canisters.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_ledger() {
        let wasm = ExternalWasm::get("ledger");
        assert!(!wasm.is_empty());
    }

    #[test]
    fn test_list_canisters() {
        let canisters = ExternalWasm::list();
        assert!(canisters.contains(&"ledger".to_string()));
    }
}
