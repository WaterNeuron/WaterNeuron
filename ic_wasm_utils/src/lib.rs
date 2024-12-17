
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use std::result;
use crate::{CanisterName, Error};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::{BTreeMap, HashSet}, path::PathBuf};
use cargo_metadata::MetadataCommand;
use crate::external::{Config, LocalCanister};
use crate::{Result, Error};
use cargo_metadata::MetadataCommand;
use std::path::PathBuf;
use std::process::Command;

const CANISTER_URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}";
const DEFAULT_BUILD_DIR: &str = "target/canisters";

type Result<T> = result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Config error: {0}")]
    Config(#[from] toml::de::Error),
    #[error("Hash mismatch - expected: {expected}, got: {got}")]
    HashMismatch { expected: String, got: String },
    #[error("Duplicate canister found: {0:?}")]
    DuplicateCanister(CanisterName),
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

pub fn get_wasm(config: &Config, info: &LocalCanister) -> Result<Vec<u8>> {
    let builder = CargoWasmBuilder::new()?;
    let out_path = builder.compile_wasm(
        &info.binary,
        &info.candid,
        &get_build_dir()
    )?;
    Ok(std::fs::read(out_path)?)
}

pub fn get_wasm(config: &Config, info: &ExternalCanister) -> Result<Vec<u8>, Error> {
    let cache_file = config.cache_path.join(info.cache_filename());

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
        let content = std::fs::read_to_string("canisters.toml")?;
        
        let config: Self = toml::from_str(&content)?;
        
        let mut seen = HashSet::new();
        for name in config.external.keys().chain(self.local.keys()) {
            if !seen.insert(name) {
                return Err(Error::DuplicateCanister(name.clone()));
            }
        }

        let cache_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(&self.cache_directory)
 
        std::fs::create_dir_all(cache_path)?;

        Ok(config)
    }
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

fn get_build_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_BUILD_DIR)
}

struct CargoWasmBuilder {
    metadata: cargo_metadata::Metadata,
}

impl CargoWasmBuilder {
    fn new() -> Result<Self> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()?;
            
        Ok(Self { metadata })
    }
    
    fn compile_wasm(
        &self,
        binary_name: &str,
        candid_path: &PathBuf,
        build_dir: &PathBuf,
    ) -> Result<PathBuf> {
        let package = self.metadata.packages
            .iter()
            .find(|p| p.name == binary_name)
            .ok_or_else(|| Error::PackageNotFound(binary_name.to_string()))?;
            
        let manifest_dir = package.manifest_path.parent().unwrap();
        std::fs::create_dir_all(build_dir)?;

        // Build WASM
        let status = Command::new("cargo")
            .current_dir(manifest_dir)
            .args([
                "build",
                "--target", "wasm32-unknown-unknown",
                "--release",
                "-p", binary_name,
            ])
            .args([
                "-C", "link-args=-z stack-size=3145728",
                "-C", "linker-plugin-lto",
                "-C", "opt-level=3",
                "-C", "debug-assertions=no",
                "-C", "debuginfo=0",
                "-C", "lto",
            ])
            .status()?;

        if !status.success() {
            return Err(Error::Build(format!(
                "Failed to build package {}", binary_name
            )));
        }

        let wasm_path = build_dir.join(format!("{}.wasm", binary_name));

        // Add candid
        let with_candid = build_dir.join(format!("{}_with_candid.wasm", binary_name));
        let status = Command::new("ic-wasm")
            .arg(&wasm_path)
            .args(["-o", &with_candid.to_string_lossy()])
            .args(["metadata", "candid:service", "-f"])
            .arg(candid_path)
            .args(["-v", "public"])
            .status()?;

        if !status.success() {
            return Err(Error::Candid("Failed to add Candid metadata".into()));
        }

        // Shrink
        let shrunk = build_dir.join(format!("{}_shrunk.wasm", binary_name));
        let status = Command::new("ic-wasm")
            .arg(&with_candid)
            .args(["-o", &shrunk.to_string_lossy()])
            .arg("shrink")
            .status()?;

        if !status.success() {
            return Err(Error::Build("Failed to shrink wasm".into()));
        }

        // Gzip
        let status = Command::new("gzip")
            .args(["-n", "--force"])
            .arg(&shrunk)
            .status()?;

        if !status.success() {
            return Err(Error::Build("Failed to gzip wasm".into()));
        }

        Ok(PathBuf::from(format!("{}.gz", shrunk.to_string_lossy())))
    }
}
