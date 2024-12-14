use crate::external::{Config, LocalCanister};
use cargo_metadata::MetadataCommand;
use std::path::PathBuf;
use std::process::Command;
use thiserror::Error;

const DEFAULT_BUILD_DIR: &str = "target/canisters";

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Metadata error: {0}")]
    Metadata(#[from] cargo_metadata::Error),
    #[error("Package {0} not found in workspace")]
    PackageNotFound(String),
    #[error("Build failed: {0}")]
    Build(String),
    #[error("Candid processing failed: {0}")]
    Candid(String),
}

pub fn get_wasm(config: &Config, info: &LocalCanister) -> Result<Vec<u8>, crate::Error> {
    let builder = CargoWasmBuilder::new()?;
    let out_path = builder.compile_wasm(
        &info.binary,
        &info.candid,
        &get_build_dir()
    )?;
    Ok(std::fs::read(out_path)?)
}

fn get_build_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(DEFAULT_BUILD_DIR)
}

struct CargoWasmBuilder {
    metadata: cargo_metadata::Metadata,
}

impl CargoWasmBuilder {
    fn new() -> Result<Self, Error> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()?;
            
        Ok(Self { metadata })
    }
    
    fn compile_wasm(
        &self,
        binary_name: &str,
        candid_path: &PathBuf,
        build_dir: &PathBuf,
    ) -> Result<PathBuf, Error> {
        let package = self.metadata.packages
            .iter()
            .find(|p| p.name == binary_name)
            .ok_or_else(|| Error::PackageNotFound(binary_name.to_string()))?;
            
        let manifest_dir = package.manifest_path.parent().unwrap();
        std::fs::create_dir_all(build_dir)?;

        // Build WASM
        let status = Command::new("cargo")
            .current_dir(manifest_dir)
            .args([
                "build",
                "--target", "wasm32-unknown-unknown",
                "--release",
                "-p", binary_name,
            ])
            .args([
                "-C", "link-args=-z stack-size=3145728",
                "-C", "linker-plugin-lto",
                "-C", "opt-level=3",
                "-C", "debug-assertions=no",
                "-C", "debuginfo=0",
                "-C", "lto",
            ])
            .status()?;

        if !status.success() {
            return Err(Error::Build(format!(
                "Failed to build package {}", binary_name
            )));
        }

        let wasm_path = build_dir.join(format!("{}.wasm", binary_name));

        // Add candid
        let with_candid = build_dir.join(format!("{}_with_candid.wasm", binary_name));
        let status = Command::new("ic-wasm")
            .arg(&wasm_path)
            .args(["-o", &with_candid.to_string_lossy()])
            .args(["metadata", "candid:service", "-f"])
            .arg(candid_path)
            .args(["-v", "public"])
            .status()?;

        if !status.success() {
            return Err(Error::Candid("Failed to add Candid metadata".into()));
        }

        // Shrink
        let shrunk = build_dir.join(format!("{}_shrunk.wasm", binary_name));
        let status = Command::new("ic-wasm")
            .arg(&with_candid)
            .args(["-o", &shrunk.to_string_lossy()])
            .arg("shrink")
            .status()?;

        if !status.success() {
            return Err(Error::Build("Failed to shrink wasm".into()));
        }

        // Gzip
        let status = Command::new("gzip")
            .args(["-n", "--force"])
            .arg(&shrunk)
            .status()?;

        if !status.success() {
            return Err(Error::Build("Failed to gzip wasm".into()));
        }

        Ok(PathBuf::from(format!("{}.gz", shrunk.to_string_lossy())))
    }
}
