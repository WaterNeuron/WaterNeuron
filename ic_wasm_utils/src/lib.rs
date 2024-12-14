use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, HashSet}, path::PathBuf};
use thiserror::Error;

const CANISTER_URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}";

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Config error: {0}")]
    Config(#[from] toml::de::Error),
    #[error("Hash mismatch - expected: {expected}, got: {got}")]
    HashMismatch { expected: String, got: String },
    #[error("Duplicate canister found: {0:?}")]
    DuplicateCanister(CanisterName),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CanisterName {
    Ledger,
    NnsGovernance,
    Cmc,
    SnsGovernance,
    SnsSwap,
    Sns,
    SnsRoot,
    Icrc1Ledger,
    Icrc1IndexNg,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CanisterInfo {
    sha256: String,
    version: String,
    wasm_file: String,
}

impl CanisterInfo {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Config {
    canisters: BTreeMap<CanisterName, CanisterInfo>,
    cache_directory: PathBuf,
}

impl Config {
    fn load() -> Result<Self, Error> {
        let content = std::fs::read_to_string("../external_canisters.toml")?;
        let config: Self = toml::from_str(&content)?;
        
        // Verify uniqueness of canisters
        let mut seen = HashSet::new();
        for name in config.canisters.keys() {
            if !seen.insert(name) {
                return Err(Error::DuplicateCanister(name.clone()));
            }
        }
        
        // Ensure cache directory exists
        let cache_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&config.cache_directory);
            
        std::fs::create_dir_all(&cache_path)?;
        
        Ok(config)
    }
}

/// Get the WASM binary for a given canister.
pub fn get_wasm(name: CanisterName) -> Result<Vec<u8>, Error> {
    let config = Config::load()?;
    
    let info = match config.canisters.get(&name) {
        Some(info) => info,
        None => return Ok(vec![]),  // Return empty if not found
    };
    
    let cache_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(&config.cache_directory);

    let cache_file = cache_path.join(info.cache_filename());

    // Check cache
    if cache_file.exists() {
        let data = std::fs::read(&cache_file)?;
        if verify_hash(&data, &info.sha256)? {
            return Ok(data);
        }
    }

    // Download if not in cache
    let data = reqwest::blocking::get(&info.url())?.bytes()?.to_vec();
    
    // Verify hash
    verify_hash(&data, &info.sha256)?;
    
    // Write to cache
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
