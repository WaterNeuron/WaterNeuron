use ic_wasm_utils::{get_wasm_path, CanisterName};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static! {
    static ref LOCAL_CANISTER_PATHS: Vec<(String, PathBuf)> = {
        let canister_configs = vec![
            (CanisterName::Local("boomerang"), false),
            (CanisterName::Local("water_neuron"), false),
            (CanisterName::Local("water_neuron"), true),
            (CanisterName::Local("sns_module"), false),
        ];
        let mut paths = Vec::with_capacity(canister_configs.len());
        for (canister_name, is_self_check) in canister_configs {
            let name = canister_name.to_string();
            let wasm_path = get_wasm_path(canister_name, is_self_check).unwrap_or_else(|e| {
                eprintln!("Error getting wasm path for {}: {}", name, e);
                panic!("Failed to get wasm path for {}", name);
            });
            paths.push((name, wasm_path));
        }
        paths
    };
    static ref DFINITY_CANISTER_PATHS: Vec<(String, PathBuf)> = {
        let canister_configs = vec![
            (CanisterName::Ledger, false),
            (CanisterName::NnsGovernance, false),
            (CanisterName::Cmc, false),
            (CanisterName::SnsGovernance, false),
            (CanisterName::SnsSwap, false),
            (CanisterName::Sns, false),
            (CanisterName::SnsRoot, false),
            (CanisterName::Icrc1Ledger, false),
            (CanisterName::Icrc1IndexNg, false),
        ];
        let mut paths = Vec::with_capacity(canister_configs.len());
        for (canister_name, is_self_check) in canister_configs {
            let name = canister_name.to_string();
            let wasm_path = get_wasm_path(canister_name, is_self_check).unwrap_or_else(|e| {
                eprintln!("Error getting wasm path for {}: {}", name, e);
                panic!("Failed to get wasm path for {}", name);
            });
            paths.push((name, wasm_path));
        }
        paths
    };
}

fn check_self_check(path: &Path) -> bool {
    let wasm_path = path.to_str().unwrap().strip_suffix(".gz").unwrap();

    let unzip_cmd = format!("gunzip -fk {}", path.to_str().unwrap());
    Command::new("sh")
        .args(["-c", &unzip_cmd])
        .status()
        .unwrap();

    let dump_cmd = format!("wasm-objdump -x {}", wasm_path);
    let output = Command::new("sh").args(["-c", &dump_cmd]).output().unwrap();

    output
        .stdout
        .windows("canister_query self_check".len())
        .any(|window| window == "canister_query self_check".as_bytes())
}

fn check_candid(name: &str, path: &Path) -> bool {
    let candid_cmd = Command::new("ic-wasm")
        .args([path.to_str().unwrap(), "metadata", "candid:service"])
        .output()
        .expect("Failed to execute ic-wasm command");

    let candid_output = String::from_utf8_lossy(&candid_cmd.stdout);
    let candid_file =
        std::fs::read_to_string(format!("{}/{}.did", name, name)).unwrap_or_else(|_| String::new());

    !candid_output.trim().is_empty() && candid_output.trim() == candid_file.trim()
}

fn check_git(path: &Path) -> bool {
    let git_cmd = Command::new("ic-wasm")
        .args([path.to_str().unwrap(), "metadata", "git_commit_id"])
        .output()
        .expect("Failed to execute ic-wasm command");

    let git_output = String::from_utf8_lossy(&git_cmd.stdout);
    let current_commit = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    git_output.trim() == current_commit.trim()
}

fn main() {
    let mut sums = vec![];
    for (name, path) in LOCAL_CANISTER_PATHS.iter() {
        println!("Building {}...", name);
        let data = std::fs::read(path).unwrap_or_else(|_| panic!("Could not read {:?}", path));
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sum = format!("{:x}", hasher.finalize());
        sums.push((name, path, sum));
    }

    println!("\nDFINITY canisters path");
    println!("─────────────────────────────────────────────");
    for (name, path) in DFINITY_CANISTER_PATHS.iter() {
        println!("{:<20} {}", name, path.display());
    }

    println!("\nSHA256 Checksums:");
    println!("─────────────────────────────────────────────");
    for (name, path, sum) in &sums {
        println!("{:<12} {}", name, sum);

        let name = if name.contains("water_neuron") {
            "water_neuron"
        } else {
            name
        };

        println!("           → {:?}", path);

        println!(
            "           {} git commit metadata",
            if check_git(path) { "✓" } else { "✗" }
        );

        println!(
            "           {} candid metadata",
            if check_candid(name, path) {
                "✓"
            } else {
                "✗"
            }
        );

        if name.contains("water_neuron") {
            let has_self_check = check_self_check(path);
            println!(
                "           {} does not have `self_check`",
                if !has_self_check { "✓" } else { "✗" }
            );
        }
        println!();
    }

    let commit = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    println!("Git commit:   {}", commit.trim());
}
