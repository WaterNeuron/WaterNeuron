use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::BTreeMap;

const URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{}/canisters/{}";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Canister {
    sha256: String,
    version: String,
    wasm_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Config {
    canisters: BTreeMap<String, Canister>,
}

// this is a comment
fn main() {
    let content = fs::read_to_string("../external_canisters.toml").unwrap();
    let raw_data: Config = toml::from_str(&content).unwrap();
    dbg!(raw_data);
}
