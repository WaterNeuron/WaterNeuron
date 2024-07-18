use candid::{self, CandidType, Decode, Deserialize, Encode, Principal};
use ic_agent::{identity::AnonymousIdentity, Agent};
use sha2::Digest;
use std::str::FromStr;

pub mod types;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://icp0.io";
    let agent = Agent::builder()
        .with_url(url)
        .with_identity(AnonymousIdentity)
        .build()?;
    agent.fetch_root_key().await?;

    let canister_id = "jfnic-kaaaa-aaaaq-aadla-cai";

    let arg = types::GetProposal {
        proposal_id: Some(types::ProposalId { id: 6 }),
    };

    let arg_raw = Encode!(&arg)?;

    let response = agent
        .query(&Principal::from_text(canister_id).unwrap(), "get_proposal")
        .with_arg(arg_raw)
        .call()
        .await?;

    let result = Decode!(response.as_slice(), Option<types::GetProposalResponse>)?;

    // get new_canister_wasm  from result
    let new_canister_wasm = result.unwrap().result.unwrap();

    let (canister_id, wasm, canister_upgrade_arg) = match new_canister_wasm {
        types::Result1::Proposal(data) => match data.proposal.unwrap().action.unwrap() {
            types::Action::UpgradeSnsControlledCanister(new_data) => {
                let wasm = new_data.new_canister_wasm;
                let canister_id = new_data.canister_id.unwrap();
                let canister_upgrade_arg = new_data.canister_upgrade_arg.unwrap();

                (canister_id, wasm, canister_upgrade_arg)
            }
            _ => {
                panic!("Not a upgrade proposal");
            }
        },
        _ => {
            panic!("Not a wasm proposal");
        }
    };

    println!("Canister ID: {}", canister_id.to_text());
    println!("Wasm: {:?}", hex::encode(&wasm));
    println!(
        "Canister Upgrade Arg: {:?}",
        hex::encode(&canister_upgrade_arg)
    );

    // compute the sha256 hash of the wasm
    let wasm_hash = sha2::Sha256::digest(&wasm);
    println!("Wasm Hash: {:?}", hex::encode(&wasm_hash));

    let upgrade_arg_hash = sha2::Sha256::digest(&canister_upgrade_arg);
    println!("Upgrade Arg Hash: {:?}", hex::encode(&upgrade_arg_hash));

    println!("Wasm:");
    println!("  Length: {} bytes", wasm.len());
    println!("  SHA256 Hash: {}", hex::encode(&wasm[..32]));
    println!("  Leading 32 Bytes: {}", hex::encode(&wasm[..32]));
    println!(
        "  Trailing 32 Bytes: {}",
        hex::encode(&wasm[wasm.len().saturating_sub(32)..])
    );

    println!("Canister Upgrade Arg:");
    println!("  Length: {} bytes", canister_upgrade_arg.len());
    println!(
        "  SHA256 Hash: {:?}",
        format_hash(&canister_upgrade_arg[..32])
    );
    println!(
        "  Leading 32 Bytes: {}",
        hex::encode(&canister_upgrade_arg[..32])
    );
    println!(
        "  Trailing 32 Bytes: {}",
        hex::encode(&canister_upgrade_arg[canister_upgrade_arg.len().saturating_sub(32)..])
    );

    Ok(())
}

fn format_hash(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ")
}
