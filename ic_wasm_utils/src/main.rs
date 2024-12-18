use ic_wasm_utils::{get_wasm_path, CanisterName};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Command;

lazy_static! {
    static ref CANISTER_PATHS: Vec<(String, PathBuf)> = vec![
        (
            "boomerang".into(),
            get_wasm_path(CanisterName::Local("boomerang".into()), false).unwrap()
        ),
        (
            "water_neuron".into(),
            get_wasm_path(CanisterName::Local("water_neuron".into()), false).unwrap()
        ),
        (
            "sns_module".into(),
            get_wasm_path(CanisterName::Local("sns_module".into()), false).unwrap()
        ),
    ];
}

fn main() {
    let mut sums = vec![];

    for (name, path) in CANISTER_PATHS.iter() {
        println!("Building {}...", name);
        let data = std::fs::read(path).expect(&format!("Could not read {:?}", path));
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sum = format!("{:x}", hasher.finalize());
        sums.push((name, path, sum));
    }

    println!("\nSHA256 Checksums:");
    println!("─────────────────────────────────────────────");
    for (name, path, sum) in sums {
        println!("{:<12} {}", name, sum);
        println!("           → {:?}", path);
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
