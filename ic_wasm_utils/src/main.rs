use ic_wasm_utils::{get_wasm_path, CanisterName};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static! {
    static ref CANISTER_PATHS: Vec<(String, PathBuf)> = vec![
        (
            "boomerang".into(),
            get_wasm_path(CanisterName::Local("boomerang"), false).unwrap()
        ),
        (
            "water_neuron".into(),
            get_wasm_path(CanisterName::Local("water_neuron"), false).unwrap()
        ),
        (
            "water_neuron_self_check".into(),
            get_wasm_path(CanisterName::Local("water_neuron"), true).unwrap()
        ),
        (
            "sns_module".into(),
            get_wasm_path(CanisterName::Local("sns_module"), false).unwrap()
        ),
    ];
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

fn main() {
    let mut sums = vec![];
    for (name, path) in CANISTER_PATHS.iter() {
        println!("Building {}...", name);
        let data = std::fs::read(path).unwrap_or_else(|_| panic!("Could not read {:?}", path));
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let sum = format!("{:x}", hasher.finalize());
        sums.push((name, path, sum));
    }

    println!("\nSHA256 Checksums:");
    println!("─────────────────────────────────────────────");
    for (name, path, sum) in &sums {
        println!("{:<12} {}", name, sum);
        println!("           → {:?}", path);
        if name.contains("water_neuron") {
            let has_self_check = check_self_check(path);
            println!(
                "           self_check: {}",
                if has_self_check { "✓" } else { "✗" }
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
