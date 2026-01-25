use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Metadata error: {0}")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("Hash mismatch")]
    HashMismatch,
    #[error("Unknown canister")]
    UnknownCanister,
    #[error("Build failed: {0}")]
    BuildFailed(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Hash, Eq, PartialOrd, Ord, PartialEq)]
pub enum CanisterName {
    Ledger,
    NnsGovernance,
    Icrc1Ledger,
    SnsGovernance,
    SnsSwap,
    Sns,
    SnsRoot,
    Cmc,
    Icrc1IndexNg,
    Local(String),
}

struct WasmBinary {
    hash: &'static str,
    ic_version: &'static str,
    name: &'static str,
}

lazy_static! {
    // REF: https://github.com/dfinity/ic/releases/tag/release-2026-01-08_03-31-base
    static ref DFINITY_CANISTERS: BTreeMap<CanisterName, WasmBinary> = {
        let ic_version = "035a2c7a2b19bc7ce7c4d977169583eb64b0e3cb";

        let mut map = BTreeMap::new();
        map.insert(
            CanisterName::Ledger,
            WasmBinary {
                hash: "67762b42631ac8c18069a6124e6c825096a50bb887e67dc3562aac12860ee564",
                ic_version,
                name: "ledger-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::NnsGovernance,
            WasmBinary {
                hash: "37fdeb4fe9c32135e6b7b405ee2171734d936abb1a7f923eef1794bf9140ef1c",
                ic_version,
                name: "governance-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Cmc,
            WasmBinary {
                hash: "33e398859dd3c988414f57596310a7ac6d12a809aa6907c7ad686e852d04f86e",
                ic_version,
                name: "cycles-minting-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::SnsGovernance,
            WasmBinary {
                hash: "4f8787135324597ed3c83b5e2083d108652589b448d0ac2dec70eaedb78d60d6",
                ic_version,
                name: "sns-governance-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::SnsSwap,
            WasmBinary {
                hash: "c6567eab9ac13859c844d8f82e76121f8ca13c3ceb0fd703f60535eb55351f66",
                ic_version,
                name: "sns-swap-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Sns,
            WasmBinary {
                hash: "ffe5262ff812c955cb63369f0bf4ed08d86ad0c18f2aa58c43076fcb460e8976",
                ic_version,
                name: "sns-wasm-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::SnsRoot,
            WasmBinary {
                hash: "dbef4e7af438505feca53f8c130dcd8c645e8f0894c3bd50634c1c6f070c8ffa",
                ic_version,
                name: "sns-root-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Icrc1Ledger,
            WasmBinary {
                hash: "860a14eadb4ce06f1ba9096522798db57f0c1fa3f8886cf803621a7e3e36385f",
                ic_version,
                name: "ic-icrc1-ledger.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Icrc1IndexNg,
            WasmBinary {
                hash: "cac207cf438df8c9fba46d4445c097f05fd8228a1eeacfe0536b7e9ddefc5f1c",
                ic_version,
                name: "ic-icrc1-index-ng.wasm.gz",
            },
        );
        map
    };
    static ref WORKSPACE_ROOT: PathBuf = cargo_metadata::MetadataCommand::new()
        .no_deps()
        .exec()
        .expect("Failed to get workspace root")
        .workspace_root
        .into();
    static ref BOOMERANG_WASM: Vec<u8> =
        get_wasm_sync(CanisterName::Local("boomerang".to_string()), false).unwrap();
    static ref WATER_NEURON_WASM: Vec<u8> =
        get_wasm_sync(CanisterName::Local("water_neuron".to_string()), true).unwrap();
}

pub async fn get_wasm_path(name: CanisterName, self_check: bool) -> Result<PathBuf> {
    match name {
        CanisterName::Local(name) => build_local_wasm(&name, self_check),
        remote => fetch_remote_wasm(remote).await,
    }
}

pub fn get_wasm_path_sync(name: CanisterName, self_check: bool) -> Result<PathBuf> {
    match name {
        CanisterName::Local(name) => build_local_wasm(&name, self_check),
        _remote => unreachable!(),
    }
}

async fn get_wasm(name: CanisterName, self_check: bool) -> Result<Vec<u8>> {
    Ok(std::fs::read(get_wasm_path(name, self_check).await?)?)
}

fn get_wasm_sync(name: CanisterName, self_check: bool) -> Result<Vec<u8>> {
    Ok(std::fs::read(get_wasm_path_sync(name, self_check)?)?)
}

fn build_local_wasm(name: &str, self_check: bool) -> Result<PathBuf> {
    std::fs::create_dir_all(WORKSPACE_ROOT.join("artifacts"))?;

    let home_dir = std::env::var("HOME")
        .map_err(|e| Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, e)))?;
    let cargo_dir = PathBuf::from(home_dir).join(".cargo");

    let rustflags = format!(
        "RUSTFLAGS=\"--remap-path-prefix={}= --remap-path-prefix={}=\"",
        WORKSPACE_ROOT.display(),
        cargo_dir.display()
    );

    let file_name = format!("{0}{1}", name, if self_check { "_self_check" } else { "" });

    let build_steps = [
        format!(
            "{0} cargo canister -p {1} --release --bin {1} --locked {2}",
            rustflags,
            name,
            if self_check {
                "--features=self_check"
            } else {
                ""
            }
        ),
        format!(
            "ic-wasm target/wasm32-unknown-unknown/release/{0}.wasm -o artifacts/{0}.wasm metadata candid:service -f {0}/{0}.did -v public",
            name
        ),
        format!(
            "ic-wasm artifacts/{0}.wasm -o artifacts/{1}.wasm metadata git_commit_id -d $(git rev-parse HEAD) -v public",
            name, file_name
        ),
        format!("ic-wasm artifacts/{0}.wasm shrink", file_name),
        format!(
            "gzip -cnf9 artifacts/{0}.wasm > artifacts/{0}.wasm.gz",
            file_name
        ),
        format!("rm artifacts/{0}.wasm", file_name),
    ];

    for cmd in &build_steps {
        if !std::process::Command::new("sh")
            .current_dir(&*WORKSPACE_ROOT)
            .args(["-c", cmd])
            .status()?
            .success()
        {
            return Err(Error::BuildFailed(cmd.to_string()));
        }
    }

    Ok(WORKSPACE_ROOT.join(format!("artifacts/{}.wasm.gz", file_name)))
}

async fn fetch_remote_wasm(canister: CanisterName) -> Result<PathBuf> {
    let wasm = DFINITY_CANISTERS
        .get(&canister)
        .ok_or(Error::UnknownCanister)?;
    let cache_path = WORKSPACE_ROOT.join("artifacts").join(wasm.name);

    if let Ok(data) = std::fs::read(&cache_path) {
        let mut hasher = Sha256::new();
        hasher.update(&data);
        if format!("{:x}", hasher.finalize()) == wasm.hash {
            return Ok(cache_path);
        }
    }

    std::fs::create_dir_all(cache_path.parent().unwrap())?;
    let url = format!(
        "https://download.dfinity.systems/ic/{}/canisters/{}",
        wasm.ic_version, wasm.name
    );

    let response = reqwest::get(&url).await?;
    let data = response.bytes().await?.to_vec();

    let mut hasher = Sha256::new();
    hasher.update(&data);
    if format!("{:x}", hasher.clone().finalize()) != wasm.hash {
        return Err(Error::HashMismatch);
    }

    std::fs::write(&cache_path, &data)?;
    Ok(cache_path)
}

pub fn boomerang_wasm() -> Vec<u8> {
    BOOMERANG_WASM.to_vec()
}
pub fn water_neuron_wasm() -> Vec<u8> {
    WATER_NEURON_WASM.to_vec()
}

pub async fn icp_ledger_wasm() -> Vec<u8> {
    get_wasm(CanisterName::Ledger, false).await.unwrap()
}

pub async fn governance_wasm() -> Vec<u8> {
    get_wasm(CanisterName::NnsGovernance, false).await.unwrap()
}

pub async fn ledger_wasm() -> Vec<u8> {
    get_wasm(CanisterName::Icrc1Ledger, false).await.unwrap()
}

pub async fn sns_governance_wasm() -> Vec<u8> {
    get_wasm(CanisterName::SnsGovernance, false).await.unwrap()
}

pub async fn sns_root_wasm() -> Vec<u8> {
    get_wasm(CanisterName::SnsRoot, false).await.unwrap()
}

pub async fn sns_swap_wasm() -> Vec<u8> {
    get_wasm(CanisterName::SnsSwap, false).await.unwrap()
}

pub async fn cmc_wasm() -> Vec<u8> {
    get_wasm(CanisterName::Cmc, false).await.unwrap()
}
