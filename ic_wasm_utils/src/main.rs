use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::BTreeMap;
use std::path::PathBuf;


const CANISTER_URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}";


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum CanisterName {
    Ledger,
    NnsGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct CanisterInfo {
    sha256: String,
    version: String,
    wasm_file: String,
}

impl CanisterInfo {
    fn download_url(&self) -> String {
        CANISTER_URL_TEMPLATE
            .replace("{version}", &self.version)
            .replace("{wasm_file}", &self.wasm_file)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Config {
    canisters: BTreeMap<CanisterName, CanisterInfo>,
}

// this is a comment
fn main() {
    let content = fs::read_to_string("../external_canisters.toml").unwrap();
    let raw_data: Config = toml::from_str(&content).unwrap();
    dbg!(&raw_data);

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("ic-canisters");

    dbg!(&cache_dir);
}
