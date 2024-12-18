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
        println!("\n{}", name);
        let data = std::fs::read(path).unwrap();
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sum = format!("{:x}", hasher.finalize());
        sums.push((path, sum));
        println!("{} canister compiled", name);
    }

    println!("\n=== Summary ===");
    for (path, sum) in sums {
        println!("{:?}: {}", path, sum);
    }

    let commit = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    println!("\ncommit: {}", commit.trim());
}
