use crate::{CanisterName, Error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, HashSet}, path::PathBuf};

const CANISTER_URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalCanister {
    pub sha256: String,
    pub version: String,
    pub wasm_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalCanister {
    pub binary: String,
    pub candid: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub cache_directory: PathBuf,
    pub external: BTreeMap<CanisterName, ExternalCanister>,
    pub local: BTreeMap<CanisterName, LocalCanister>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct LegacyConfig {
    cache_directory: PathBuf,
    canisters: BTreeMap<CanisterName, ExternalCanister>,
}

impl ExternalCanister {
    fn url(&self) -> String {
        CANISTER_URL_TEMPLATE
            .replace("{version}", &self.version)
            .replace("{wasm_file}", &self.wasm_file)
    }

    fn cache_filename(&self) -> String {
        let parts: Vec<&str> = self.wasm_file.split(".wasm").collect();
        format!("{}_{}.wasm{}", parts[0], self.version, parts[1])
    }
}

impl Config {
    pub fn load() -> Result<Self, Error> {
        // Try new format first
        if let Ok(content) = std::fs::read_to_string("canisters.toml") {
            let config: Self = toml::from_str(&content)?;
            config.validate()?;
            return Ok(config);
        }

        // Fall back to legacy format
        let content = std::fs::read_to_string("../external_canisters.toml")?;
        let legacy: LegacyConfig = toml::from_str(&content)?;
        
        let config = Self {
            cache_directory: legacy.cache_directory,
            external: legacy.canisters,
            local: BTreeMap::new(),
        };
        
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), Error> {
        // Check for duplicates
        let mut seen = HashSet::new();
        for name in self.external.keys().chain(self.local.keys()) {
            if !seen.insert(name) {
                return Err(Error::DuplicateCanister(name.clone()));
            }
        }

        // Ensure cache directory exists
        let cache_path = self.cache_dir();
        std::fs::create_dir_all(cache_path)?;
        Ok(())
    }

    pub fn cache_dir(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&self.cache_directory)
    }
}

pub fn get_wasm(config: &Config, info: &ExternalCanister) -> Result<Vec<u8>, Error> {
    let cache_file = config.cache_dir().join(info.cache_filename());

    if cache_file.exists() {
        let data = std::fs::read(&cache_file)?;
        if verify_hash(&data, &info.sha256)? {
            return Ok(data);
        }
    }

    let data = reqwest::blocking::get(&info.url())?.bytes()?.to_vec();
    verify_hash(&data, &info.sha256)?;
    std::fs::write(&cache_file, &data)?;
    
    Ok(data)
}

fn verify_hash(data: &[u8], expected: &str) -> Result<bool, Error> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let got = format!("{:x}", hasher.finalize());
    
    if got != expected {
        return Err(Error::HashMismatch {
            expected: expected.to_string(),
            got,
        });
    }
    
    Ok(true)
}
