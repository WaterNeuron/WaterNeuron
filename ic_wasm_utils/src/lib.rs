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
    pub external: BTreeMap<CanisterName, ExternalCanister>,
}

impl ExternalCanister {
    fn url(&self) -> String {
        CANISTER_URL_TEMPLATE
            .replace("{version}", &self.version)
            .replace("{wasm_file}", &self.wasm_file)
    }
    
    fn versioned_filename(&self) -> String {
        let parts: Vec<&str> = self.wasm_file.split(".wasm").collect();
        format!("{}_{}.wasm{}", parts[0], self.version, parts[1])
    }

}

impl Config {
    pub fn load() -> Result<Self> {
        let metadata = MetadataCommand::new().no_deps().exec()?;
        let config_path = metadata.workspace_root.join("canisters.toml");
        let content = std::fs::read_to_string(config_path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }
}

/// Get the WASM binary for a given canister.
pub fn get_wasm(name: CanisterName) -> Result<Vec<u8>> {
    let config = Config::load()?;
    let metadata = MetadataCommand::new().no_deps().exec()?;
    let artifacts_dir = metadata.workspace_root.join("artifacts");
    std::fs::create_dir_all(&artifacts_dir)?;

    if let Some(info) = config.external.get(&name) {
        let wasm_file = artifacts_dir.join(info.versioned_filename());

        if wasm_file.exists() {
            let data = std::fs::read(&wasm_file)?;
            if verify_hash(&data, &info.sha256)? {
                return Ok(data);
            }
        }

        let data = reqwest::blocking::get(&info.url())?.bytes()?.to_vec();
        verify_hash(&data, &info.sha256)?;
        std::fs::write(&wasm_file, &data)?;

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
        let _package = self
            .metadata
            .packages
            .iter()
            .find(|p| p.name == binary_name)
            .ok_or_else(|| Error::PackageNotFound(binary_name.to_string()))?;

        let workspace_root = &self.metadata.workspace_root;
        std::fs::create_dir_all(workspace_root.join("artifacts"))?;

        // Step 1: Build with cargo
        let status = Command::new("cargo")
            .current_dir(workspace_root)
            .args(["canister", "-p", binary_name, "--release", "--bin", binary_name])
            .status()?;

        if !status.success() {
            return Err(Error::Build(format!("Failed to build package {}", binary_name)));
        }

        // Step 2: Add candid
        let status = Command::new("ic-wasm")
            .current_dir(workspace_root)
            .arg(format!("target/wasm32-unknown-unknown/release/{}.wasm", binary_name))
            .args(["-o", &format!("artifacts/{}_with_candid.wasm", binary_name)])
            .args(["metadata", "candid:service", "-f"])
            .arg(format!("{}/{}.did", binary_name, binary_name))
            .args(["-v", "public"])
            .status()?;

        if !status.success() {
            return Err(Error::Build("Failed to add Candid metadata".into()));
        }

        // Step 3: Add git commit
        let git_commit = Command::new("git")
            .current_dir(workspace_root)
            .args(["rev-parse", "HEAD"])
            .output()?;

        let status = Command::new("ic-wasm")
            .current_dir(workspace_root)
            .arg(format!("artifacts/{}_with_candid.wasm", binary_name))
            .args(["-o", &format!("artifacts/{}_with_candid_and_git.wasm", binary_name)])
            .args(["metadata", "git_commit_id", "-d"])
            .arg(String::from_utf8_lossy(&git_commit.stdout).trim())
            .args(["-v", "public"])
            .status()?;
        
        std::thread::sleep(std::time::Duration::from_secs(3));

        if !status.success() {
            return Err(Error::Build("Failed to add git metadata".into()));
        }

        // Step 4: Shrink
        let status = Command::new("ic-wasm")
            .current_dir(workspace_root)
            .arg(format!("artifacts/{}_with_candid_and_git.wasm", binary_name))
            .args(["-o", &format!("artifacts/{}_candid_git_shrink.wasm", binary_name)])
            .arg("shrink")
            .status()?;
        
        std::thread::sleep(std::time::Duration::from_secs(3));

        if !status.success() {
            return Err(Error::Build("Failed to shrink wasm".into()));
        }

        // Step 5: Gzip
        let status = Command::new("gzip")
            .current_dir(workspace_root)
            .args(["-n", "--force"])
            .arg(format!("artifacts/{}_candid_git_shrink.wasm", binary_name))
            .status()?;

        std::thread::sleep(std::time::Duration::from_secs(3));

        if !status.success() {
            println!("Exit code: {:?}", status.code());
            println!("Full status: {:?}", status);
            return Err(Error::Build(format!("Failed to gzip wasm {:?}", status)));
        }

        Ok(workspace_root.join(format!("artifacts/{}_candid_git_shrink.wasm.gz", binary_name)).into())
    }
}
