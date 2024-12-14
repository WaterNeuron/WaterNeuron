mod compile;
mod external;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

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
    #[error("Compilation error: {0}")]
    Compile(#[from] compile::Error),
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

/// Get the WASM binary for a given canister.
pub fn get_wasm(name: CanisterName) -> Result<Vec<u8>, Error> {
    let config = external::Config::load()?;
    
    match (config.external.get(&name), config.local.get(&name)) {
        (Some(info), _) => external::get_wasm(&config, info),
        (_, Some(info)) => compile::get_wasm(&config, info),
        _ => Ok(vec![]),
    }
}
