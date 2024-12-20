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
pub enum CanisterName<'a> {
    Ledger,
    NnsGovernance,
    Icrc1Ledger,
    SnsGovernance,
    SnsSwap,
    Sns,
    SnsRoot,
    Cmc,
    Icrc1IndexNg,
    Local(&'a str),
}

struct WasmBinary {
    hash: &'static str,
    ic_version: &'static str,
    name: &'static str,
}

lazy_static! {
    static ref DFINITY_CANISTERS: BTreeMap<CanisterName<'static>, WasmBinary> = {
        let mut map = BTreeMap::new();
        map.insert(
            CanisterName::Ledger,
            WasmBinary {
                hash: "e31a3b38bbb3704876d8825bb826101d6f1f1843ad99c21a0d563e80bdd6e2f6",
                ic_version: "de29a1a55b589428d173b31cdb8cec0923245657",
                name: "ledger-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::NnsGovernance,
            WasmBinary {
                hash: "8f76b2de37197b3ff0ae188f1ef99ddd5bd75cb8f83fb87c2889822ece0b5576",
                ic_version: "ad5629caa17ac8a4545bc2e3cf0ecc990c9f681e",
                name: "governance-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Cmc,
            WasmBinary {
                hash: "d33b381e3de4cb3a35493ba0398b3c7f7b7165306400b25fe9129b9f28d08774",
                ic_version: "0abc8efa13a533576dbd9b652e37e4a817e6051c",
                name: "cycles-minting-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::SnsGovernance,
            WasmBinary {
                hash: "e6b285a50237a46d7cf72eb27ae4840222b98ecc02c20954a7946d039cab59f0",
                ic_version: "80e0363393ea26a36b77e8c75f7f183cb521f67f",
                name: "sns-governance-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::SnsSwap,
            WasmBinary {
                hash: "2bbaf53b7cbb8f20cdd6b30bf709f461a47d10b02b38cb1d54d52789c907f202",
                ic_version: "80e0363393ea26a36b77e8c75f7f183cb521f67f",
                name: "sns-swap-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Sns,
            WasmBinary {
                hash: "a6ffc60e50d7c59ce5b3bfbfa1a234287891e9396c85be312c8e725a2510fb35",
                ic_version: "80e0363393ea26a36b77e8c75f7f183cb521f67f",
                name: "sns-wasm-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::SnsRoot,
            WasmBinary {
                hash: "dd0b6dfe7a25852ed6d421ce71382f30f7275046aed7c64d870c8e0bb4bba6ea",
                ic_version: "80e0363393ea26a36b77e8c75f7f183cb521f67f",
                name: "sns-root-canister.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Icrc1Ledger,
            WasmBinary {
                hash: "4264ce2952c4e9ff802d81a11519d5e3ffdaed4215d5831a6634e59efd72f7d8",
                ic_version: "a3831c87440df4821b435050c8a8fcb3745d86f6",
                name: "ic-icrc1-ledger.wasm.gz",
            },
        );
        map.insert(
            CanisterName::Icrc1IndexNg,
            WasmBinary {
                hash: "cac207cf438df8c9fba46d4445c097f05fd8228a1eeacfe0536b7e9ddefc5f1c",
                ic_version: "a3831c87440df4821b435050c8a8fcb3745d86f6",
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
        get_wasm(CanisterName::Local("boomerang".into()), true).unwrap();
    static ref WATER_NEURON_WASM: Vec<u8> =
        get_wasm(CanisterName::Local("water_neuron".into()), true).unwrap();
    static ref SNS_MODULE_WASM: Vec<u8> =
        get_wasm(CanisterName::Local("sns_module".into()), true).unwrap();
}

pub fn get_wasm_path(name: CanisterName, self_check: bool) -> Result<PathBuf> {
    match name {
        CanisterName::Local(name) => build_local_wasm(&name, self_check),
        remote => fetch_remote_wasm(&remote),
    }
}

fn get_wasm(name: CanisterName, self_check: bool) -> Result<Vec<u8>> {
    Ok(std::fs::read(get_wasm_path(name, self_check)?)?)
}

fn build_local_wasm(name: &str, self_check: bool) -> Result<PathBuf> {
    std::fs::create_dir_all(WORKSPACE_ROOT.join("artifacts"))?;

    let self_check_flag = if self_check {
        "--features=self_check"
    } else {
        ""
    };

    let build_steps = [
        format!("cargo canister -p {0} --release --bin {0} --locked {1}", name, self_check_flag),
        format!("ic-wasm target/wasm32-unknown-unknown/release/{0}.wasm -o artifacts/{0}_candid.wasm metadata candid:service -f {0}/{0}.did -v public", name),
        format!("ic-wasm artifacts/{0}_candid.wasm -o artifacts/{0}_candid_git.wasm metadata git_commit_id -d $(git rev-parse HEAD) -v public", name),
        format!("ic-wasm artifacts/{0}_candid_git.wasm -o artifacts/{0}_candid_git_shrink.wasm shrink", name),
        format!("gzip -nf9v artifacts/{0}_candid_git_shrink.wasm", name),
        format!("mv artifacts/{0}_candid_git_shrink.wasm.gz artifacts/{0}.wasm.gz", name),
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

    Ok(WORKSPACE_ROOT.join(format!("artifacts/{}.wasm.gz", name)))
}

fn fetch_remote_wasm(canister: &CanisterName) -> Result<PathBuf> {
    let wasm = DFINITY_CANISTERS
        .get(canister)
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
    let data = reqwest::blocking::get(format!(
        "https://download.dfinity.systems/ic/{}/canisters/{}",
        wasm.ic_version, wasm.name
    ))?
    .bytes()?
    .to_vec();

    let mut hasher = Sha256::new();
    hasher.update(&data);
    if format!("{:x}", hasher.finalize()) != wasm.hash {
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
pub fn sns_module_wasm() -> Vec<u8> {
    SNS_MODULE_WASM.to_vec()
}

pub fn icp_ledger_wasm() -> Vec<u8> {
    get_wasm(CanisterName::Ledger, false).unwrap()
}
pub fn governance_wasm() -> Vec<u8> {
    get_wasm(CanisterName::NnsGovernance, false).unwrap()
}
pub fn ledger_wasm() -> Vec<u8> {
    get_wasm(CanisterName::Icrc1Ledger, false).unwrap()
}
pub fn sns_governance_wasm() -> Vec<u8> {
    get_wasm(CanisterName::SnsGovernance, false).unwrap()
}
pub fn sns_root_wasm() -> Vec<u8> {
    get_wasm(CanisterName::SnsRoot, false).unwrap()
}
pub fn sns_swap_wasm() -> Vec<u8> {
    get_wasm(CanisterName::SnsSwap, false).unwrap()
}
pub fn cmc_wasm() -> Vec<u8> {
    get_wasm(CanisterName::Cmc, false).unwrap()
}
