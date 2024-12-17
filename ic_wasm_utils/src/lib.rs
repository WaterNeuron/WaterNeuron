use cargo_metadata::MetadataCommand;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;
use std::result;
use thiserror::Error;

const CANISTER_URL_TEMPLATE: &str =
    "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}";
const DEFAULT_BUILD_DIR: &str = "target/wasm32-unknown-unknown/release/";

pub type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Config error: {0}")]
    Config(#[from] toml::de::Error),
    #[error("Hash mismatch - expected: {expected}, got: {got}")]
    HashMismatch { expected: String, got: String },
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Metadata error: {0}")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("Package {0} not found in workspace")]
    PackageNotFound(String),
    #[error("Build failed: {0}")]
    Build(String),
    #[error("Candid processing failed: {0}")]
    Candid(String),
    #[error("Canister {0} not found")]
    CanisterNotFound(String),
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
    Local(String),
}

impl fmt::Display for CanisterName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ledger => write!(f, "ledger"),
            Self::NnsGovernance => write!(f, "nns_governance"),
            Self::Cmc => write!(f, "cmc"),
            Self::SnsGovernance => write!(f, "sns_governance"),
            Self::SnsSwap => write!(f, "sns_swap"),
            Self::Sns => write!(f, "sns"),
            Self::SnsRoot => write!(f, "sns_root"),
            Self::Icrc1Ledger => write!(f, "icrc1_ledger"),
            Self::Icrc1IndexNg => write!(f, "icrc1_index_ng"),
            Self::Local(inner) => write!(f, "{}", inner),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExternalCanister {
    pub sha256: String,
    pub version: String,
    pub wasm_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Config {
    pub cache_directory: PathBuf,
    pub external: BTreeMap<CanisterName, ExternalCanister>,
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
    pub fn load() -> Result<Self> {
        let content = std::fs::read_to_string("canisters.toml")?;
        let config: Self = toml::from_str(&content)?;

        let cache_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&config.cache_directory);

        std::fs::create_dir_all(cache_path)?;

        Ok(config)
    }
}

/// Get the WASM binary for a given canister.
pub fn get_wasm(name: CanisterName) -> Result<Vec<u8>> {
    let config = Config::load()?;

    if let Some(info) = config.external.get(&name) {
        let cache_file = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&config.cache_directory)
            .join(info.cache_filename());

        if cache_file.exists() {
            let data = std::fs::read(&cache_file)?;
            if verify_hash(&data, &info.sha256)? {
                return Ok(data);
            }
        }

        let data = reqwest::blocking::get(&info.url())?.bytes()?.to_vec();
        verify_hash(&data, &info.sha256)?;
        std::fs::write(&cache_file, &data)?;

        return Ok(data);
    }

    let builder = CargoWasmBuilder::new()?;
    match builder.compile_wasm(&name.to_string().to_lowercase()) {
        Ok(path) => Ok(std::fs::read(path)?),
        Err(Error::PackageNotFound(_)) => Err(Error::CanisterNotFound(name.to_string())),
        Err(e) => Err(e),
    }
}

fn verify_hash(data: &[u8], expected: &str) -> Result<bool> {
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

struct CargoWasmBuilder {
    metadata: cargo_metadata::Metadata,
}

impl CargoWasmBuilder {
    fn new() -> Result<Self> {
        let metadata = MetadataCommand::new().no_deps().exec()?;
        Ok(Self { metadata })
    }

    fn compile_wasm(&self, binary_name: &str) -> Result<PathBuf> {
        let package = self
            .metadata
            .packages
            .iter()
            .find(|p| p.name == binary_name)
            .ok_or_else(|| Error::PackageNotFound(binary_name.to_string()))?;

        let manifest_dir = package.manifest_path.parent().unwrap();
        let build_dir = PathBuf::from(DEFAULT_BUILD_DIR);
        
        Ok(build_dir)
    }
}
