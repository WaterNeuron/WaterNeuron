use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use clap::Parser;
use ic_agent::AgentError;
use ic_agent::{identity::AnonymousIdentity, Agent};
use log::info;
use regex::Regex;
use sha2::Digest;
use std::str::FromStr;
use thiserror::Error;
use types::GetProposalResponse;

mod types;

const WATERNEURON_GOVERNANCE_CANISTER: &'static str = "jfnic-kaaaa-aaaaq-aadla-cai";

type Result<T> = std::result::Result<T, CustomError>;

#[derive(Error, Debug)]
pub enum CustomError {
    #[error("Invalid neuron ID")]
    InvalidNeuronId,

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Candid error: {0}")]
    CandidError(#[from] candid::Error),

    #[error("Decode error: {0}")]
    DecodeError(String),

    #[error("Principal error: {0}")]
    PrincipalError(#[from] ic_agent::export::PrincipalError),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

enum CanisterType {
    IcIcrc1Ledger,
    GovernanceCanister,
    SnsRootCanister,
    SnsWasmCanister,
    SnsSwapCanister,
    SnsGovernanceCanister,
    CyclesMintingCanister,
    LedgerCanister,
    IndexCanister,
}

/// Proposal verifier
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Proposal ID
    #[arg(short, long)]
    proposal_id: u64,
    /// WASH hash
    #[arg(short, long)]
    wash_hash: String,
    /// Canister upgrade arg hash
    #[arg(short, long)]
    canister_upgrade_args: String,
    /// Canister did file
    #[arg(short, long)]
    canister: CanisterType,
    /// Git commit
    #[arg(short, long)]
    git_commit: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let proposal_id = args.proposal_id;
    let wash_hash = args.wash_hash;
    let canister_upgrade_arg_hash = args.canister_upgrade_arg_hash;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    match run(proposal_id).await {
        Ok(_) => {}
        Err(err) => {
            eprintln!("Error: {:?}", err);
            std::process::exit(1);
        }
    }
}

async fn fetch_proposal(proposal_id: u64) -> Result<Option<GetProposalResponse>> {
    let url = "https://icp0.io";
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(AnonymousIdentity)
        .build()?;
    agent.fetch_root_key().await?;

    let arg = types::GetProposal {
        proposal_id: Some(types::ProposalId { id: 6 }),
    };

    let arg_raw = Encode!(&arg)?;

    let response = agent
        .query(
            &Principal::from_text(WATERNEURON_GOVERNANCE_CANISTER)?,
            "get_proposal",
        )
        .with_arg(arg_raw)
        .call()
        .await?;

    Ok(Decode!(
        response.as_slice(),
        Option<types::GetProposalResponse>
    )?)
}

async fn get_shasum(proposal_id: u64) -> Result<(String, String)> {
    // get new_canister_wasm  from result
    let new_canister_wasm = fetch_proposal(proposal_id)
        .await?
        .ok_or(CustomError::Generic(
            "No proposal found for the given proposal ID".to_string(),
        ))?
        .result
        .ok_or(CustomError::Generic(
            "No result found for the given proposal ID".to_string(),
        ))?;

    let (canister_id, wasm, canister_upgrade_arg) = match new_canister_wasm {
        types::Result1::Proposal(data) => match data
            .proposal
            .ok_or(CustomError::DecodeError(
                "No proposal found in the result".to_string(),
            ))?
            .action
        {
            Some(types::Action::UpgradeSnsControlledCanister(new_data)) => {
                let wasm = new_data.new_canister_wasm;
                let canister_id = new_data.canister_id.ok_or(CustomError::Generic(
                    "No canister ID found in the proposal".to_string(),
                ))?;
                let canister_upgrade_arg =
                    new_data.canister_upgrade_arg.ok_or(CustomError::Generic(
                        "No canister upgrade arg found in the proposal".to_string(),
                    ))?;

                (canister_id, wasm, canister_upgrade_arg)
            }
            _ => Err(CustomError::Generic("Not an upgrade proposal".to_string()))?,
        },
        _ => Err(CustomError::Generic("Not a proposal result".to_string()))?,
    };

    info!("Canister ID: {}", canister_id);
    info!("Wasm: {:?}", wasm);
    info!("Canister Upgrade Arg: {:?}", canister_upgrade_arg);

    let wasm_utf8 = std::str::from_utf8(&wasm)?;
    info!("Wasm (UTF-8): {}", wasm_utf8);

    let wasm_sha256_hash = extract_sha256_hash(wasm_utf8).ok_or(CustomError::Generic(
        "No SHA256 hash found in the Wasm".to_string(),
    ))?;
    info!("Wasm SHA256 Hash: {}", wasm_sha256_hash);

    let canister_upgrade_arg_utf8 = std::str::from_utf8(&canister_upgrade_arg)?;
    info!(
        "Canister Upgrade Arg (UTF-8): {}",
        canister_upgrade_arg_utf8
    );
    let canister_upgrade_arg_sha256_hash = extract_sha256_hash(canister_upgrade_arg_utf8).ok_or(
        CustomError::Generic("No SHA256 hash found in the Canister Upgrade Arg".to_string()),
    )?;
    info!(
        "Canister Upgrade Arg SHA256 Hash: {}",
        canister_upgrade_arg_sha256_hash
    );

    Ok((wasm_sha256_hash, canister_upgrade_arg_sha256_hash))
}

async fn run(proposal_id: u64) -> Result<()> {
    let (wasm_sha256_hash, canister_upgrade_arg_sha256_hash) = get_shasum(proposal_id).await?;

    let wasm_path = std::env::var("WASM_PATH")
        .map_err(|_| CustomError::Generic("WASM_PATH not set".to_string()))?;

    let wasm_bytes = std::fs::read(&wasm_path)?;

    let wasm_sha256_hash_local = sha2::Sha256::digest(&wasm_bytes)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    info!("Local Wasm SHA256 Hash: {}", wasm_sha256_hash_local);

    if wasm_sha256_hash_local != wasm_sha256_hash {
        return Err(CustomError::Generic(
            "Local Wasm SHA256 hash does not match the proposal Wasm SHA256 hash".to_string(),
        ));
    } else {
        info!("Local Wasm SHA256 hash matches the proposal Wasm SHA256 hash");
    }

    // check the wasm hash is the same as the one in the proposal

    // compute the didc hash of the args given

    // check the upgrade hash is the same as the one in the proposal

    // check with ic-wasm the git-commit is indeed the correct one

    Ok(())
}

fn extract_sha256_hash(wasm_utf8: &str) -> Option<String> {
    let re = Regex::new(r"SHA256 Hash:\s+((?:[0-9A-Fa-f]{2}\s*){32})").unwrap();
    re.captures(wasm_utf8).and_then(|cap| {
        cap.get(1).map(|m| {
            m.as_str()
                .split_whitespace()
                .collect::<String>()
                .to_lowercase()
        })
    })
}
