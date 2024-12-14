use serde::{Deserialize, Serialize};
use std::fs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use reqwest::blocking::Client;

const CANISTER_URL_TEMPLATE: &str = "https://download.dfinity.systems/ic/{version}/canisters/{wasm_file}";

/*
 * TODO list:
 * - make sure every key in the config is unique
 * - everytime we run the script check the size of the cache
 * - check the sha256 matches
 *
 */

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
enum CanisterName {
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

    fn download(&self, cache: &PathBuf) {
        let client = Client::new();

        let file_name = {
            let parts: Vec<&str> = self.wasm_file.split(".wasm").collect();
            format!("{}_{}.wasm{}", parts[0], self.version, parts[1])
        };
        dbg!(&file_name);

        let dest = cache.join(&file_name);
        dbg!(&dest);

        let response = client.get(self.url()).send().unwrap().bytes().unwrap();

        fs::write(&dest, &response).unwrap();
    }

}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Config {
    canisters: BTreeMap<CanisterName, CanisterInfo>,
    cache_directory: PathBuf,
}

// this is a comment
fn main() {
    let content = fs::read_to_string("../external_canisters.toml").unwrap();
    let mut raw_data: Config = toml::from_str(&content).unwrap();
    dbg!(&raw_data);

    let cache_dir = PathBuf::from(std::env::var("HOME").unwrap()).join(raw_data.cache_directory);
    dbg!(&cache_dir);
    dbg!(fs::create_dir_all(&cache_dir).unwrap());

    dbg!(raw_data.canisters.get(&CanisterName::Sns).unwrap().download(&cache_dir));
}
