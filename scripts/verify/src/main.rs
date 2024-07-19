use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use clap::{Parser, ValueEnum};
use ic_agent::AgentError;
use ic_agent::{identity::AnonymousIdentity, Agent};
use log::{error, info};
use regex::Regex;
use sha2::Digest;
use sha2::Sha256;
use std::io::Read;
use std::os::unix::process;
use std::process::Command;
use std::str::FromStr;
use std::string::FromUtf8Error;
use thiserror::Error;
use types::GetProposalResponse;

mod types;

const WATERNEURON_GOVERNANCE_CANISTER: &'static str = "jfnic-kaaaa-aaaaq-aadla-cai";
const CANDID_DIDC_PATH: &'static str = "CANDID_DIDC_PATH";

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

    #[error("Hex error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("UTF-8 conversion error: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),

    #[error("Generic error: {0}")]
    Generic(String),
}

#[derive(ValueEnum, Debug, Clone)]
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
    WaterNeuronCanister,
}

impl CanisterType {
    fn get_wasm_env(&self) -> &'static str {
        match self {
            Self::IcIcrc1Ledger => "IC_ICRC1_LEDGER_WASM_PATH",
            Self::LedgerCanister => "LEDGER_CANISTER_WASM_PATH",
            Self::CyclesMintingCanister => "CYCLES_MINTING_CANISTER_WASM_PATH",
            Self::SnsGovernanceCanister => "SNS_GOVERNANCE_CANISTER_WASM_PATH",
            Self::SnsSwapCanister => "SNS_SWAP_CANISTER_WASM_PATH",
            Self::SnsWasmCanister => "SNS_WASM_CANISTER_WASM_PATH",
            Self::SnsRootCanister => "SNS_ROOT_CANISTER_WASM_PATH",
            Self::GovernanceCanister => "GOVERNANCE_CANISTER_WASM_PATH",
            Self::IndexCanister => "IC_ICRC1_INDEX_WASM_PATH",
            Self::WaterNeuronCanister => "WATERNEURON_CANISTER_WASM_PATH",
        }
    }

    fn get_candid_env(&self) -> &'static str {
        match self {
            Self::IcIcrc1Ledger => "IC_ICRC1_LEDGER_CANDID_PATH",
            Self::LedgerCanister => "LEDGER_CANISTER_CANDID_PATH",
            Self::CyclesMintingCanister => "CYCLES_MINTING_CANISTER_CANDID_PATH",
            Self::SnsGovernanceCanister => "SNS_GOVERNANCE_CANISTER_CANDID_PATH",
            Self::SnsSwapCanister => "SNS_SWAP_CANISTER_CANDID_PATH",
            Self::SnsWasmCanister => "SNS_WASM_CANISTER_CANDID_PATH",
            Self::SnsRootCanister => "SNS_ROOT_CANISTER_CANDID_PATH",
            Self::GovernanceCanister => "GOVERNANCE_CANISTER_CANDID_PATH",
            Self::IndexCanister => "IC_ICRC1_INDEX_CANDID_PATH",
            Self::WaterNeuronCanister => "WATERNEURON_CANISTER_CANDID_PATH",
        }
    }
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
    wasm_hash: String,
    /// Canister upgrade arg
    #[arg(short, long, num_args = 0..)]
    upgrade_args: Vec<String>,
    /// Canister did file
    #[arg(short, long)]
    canister: CanisterType,
    /// Git commit
    #[arg(short, long)]
    git_commit: String,
    //// Target canister
    #[arg(short, long)]
    target_canister: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let proposal_id = args.proposal_id;
    let wasm_hash = args.wasm_hash;
    let upgrade_args = args.upgrade_args;
    let canister = args.canister;
    let git_commit = args.git_commit;

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // TODO check all the env variables used in CanisterType exist
    // if not, return error

    match run(proposal_id, wasm_hash, upgrade_args, canister, git_commit).await {
        Ok(_) => {}
        Err(err) => {
            error!("Error: {:?}", err);
            std::process::exit(1);
        }
    }
}

async fn run(
    proposal_id: u64,
    wasm_hash: String,
    upgrade_args: Vec<String>,
    canister: CanisterType,
    git_commit: String,
) -> Result<()> {
    let (wasm_sha256_hash, canister_upgrade_arg_sha256_hash) = get_shasum(proposal_id).await?;

    let wasm_path = std::env::var(canister.get_wasm_env())
        .map_err(|_| CustomError::Generic(format!("{} not set", canister.get_wasm_env())))?;

    let wasm_bytes = std::fs::read(&wasm_path)?;

    let wasm_sha256_hash_local = sha2::Sha256::digest(&wasm_bytes)
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    info!("Local Wasm SHA256 Hash: {}", wasm_sha256_hash_local);

    if wasm_sha256_hash_local != wasm_sha256_hash {
        return Err(CustomError::Generic(format!(
            "Local Wasm SHA256 hash does not match the proposal Wasm SHA256 hash: {} != {}",
            wasm_sha256_hash_local, wasm_sha256_hash
        )));
    } else {
        info!("Local Wasm SHA256 hash matches the proposal Wasm SHA256 hash");
    }

    // check the wasm hash is the same as the one in the proposal
    if wasm_hash != wasm_sha256_hash {
        return Err(CustomError::Generic(
            "Wasm hash does not match the proposal Wasm hash: {wasm_hash} != {wasm_sha256_hash}"
                .to_string(),
        ));
    } else {
        info!("Wasm hash matches the proposal Wasm hash");
    }

    let candid_file_path = std::env::var(canister.get_candid_env())
        .map_err(|_| CustomError::Generic(format!("{} not set", canister.get_candid_env())))?;

    let candid_binary = std::env::var(CANDID_DIDC_PATH)
        .map_err(|_| CustomError::Generic(format!("{} not set", CANDID_DIDC_PATH)))?;

    // split the upgrade args into a vector between ' '
    let encoded_candid_arg = Command::new(candid_binary)
        .arg("encode")
        .arg("-d")
        .arg(candid_file_path)
        .arg("-t")
        .args(upgrade_args)
        .output()?;

    let stdout = encoded_candid_arg.stdout;
    debug!("Raw stdout: {:?}", stdout);

    let encoded_candid_str = String::from_utf8(stdout)?;
    debug!("Encoded Candid string: {}", encoded_candid_str);

    let cleaned_candid_str = encoded_candid_str.replace(|c: char| c.is_whitespace(), "");
    debug!("Cleaned Candid string: {}", cleaned_candid_str);

    let bytes = hex::decode(&cleaned_candid_str)?;
    debug!("Decoded bytes: {:?}", bytes);

    let sha256_encoded_candid_arg = format!("{:x}", Sha256::digest(&bytes));
    info!("Computed hash: {}", sha256_encoded_candid_arg);

    info!(
        "SHA256 hash of the encoded upgrade args: {}",
        sha256_encoded_candid_arg
    );

    // compare them with the sha256 hash in the proposal
    if sha256_encoded_candid_arg != canister_upgrade_arg_sha256_hash {
        return Err(CustomError::Generic(
            "Encoded upgrade args hash does not match the proposal encoded upgrade args hash"
                .to_string(),
        ));
    } else {
        info!("Encoded upgrade args hash matches the proposal encoded upgrade args hash");
    }

    // check the upgrade hash is the same as the one in the proposal

    // check with ic-wasm the git-commit is indeed the correct one

    Ok(())
}

async fn fetch_proposal(proposal_id: u64) -> Result<Option<GetProposalResponse>> {
    let url = "https://icp0.io";
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(AnonymousIdentity)
        .build()?;
    agent.fetch_root_key().await?;

    let arg = types::GetProposal {
        proposal_id: Some(types::ProposalId { id: proposal_id }),
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
    info!("Wasm: {}", hex::encode(&wasm));
    info!(
        "Canister Upgrade Arg: {}",
        hex::encode(&canister_upgrade_arg)
    );

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
