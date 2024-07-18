use ic_agent::{Agent, identity::AnonymousIdentity};
use candid::{self, CandidType,  Deserialize, Principal, Encode, Decode};
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
        proposal_id: Some(types::ProposalId { id: 5 }),
    };

    let arg_raw = Encode!(&arg)?;

    let response = agent.query(&Principal::from_text(canister_id).unwrap(), "get_proposal")
        .with_arg(arg_raw)
        .call()
        .await?;

    let result = Decode!(response.as_slice(), Option<types::GetProposalResponse>)?;

    // get new_canister_wasm  from result
    let new_canister_wasm = result.unwrap().proposal.unwrap().new_canister_wasm;

    println!("Proposal: {:?}", result);

    Ok(())
}
