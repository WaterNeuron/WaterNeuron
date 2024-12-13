use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::BTreeMap;

const URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{}/canisters/{}";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Config {
    canisters: BTreeMap<CanisterName, CanisterInfo>,
}

// this is a comment
fn main() {
    let content = fs::read_to_string("../external_canisters.toml").unwrap();
    let raw_data: Config = toml::from_str(&content).unwrap();
    dbg!(raw_data);
}
