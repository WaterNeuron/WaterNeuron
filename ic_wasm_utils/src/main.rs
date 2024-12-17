use ic_wasm_utils::{get_wasm, CanisterName, Error, Result};

fn main() -> Result<()> {
    let canisters = [
        CanisterName::Ledger,
        CanisterName::NnsGovernance,
        CanisterName::Cmc,
        CanisterName::SnsGovernance,
        CanisterName::SnsSwap,
        CanisterName::Sns,
        CanisterName::SnsRoot,
        CanisterName::Icrc1Ledger,
        CanisterName::Icrc1IndexNg,
        CanisterName::Local("boomerang".to_string()),
        CanisterName::Local("sns_module".to_string()),
        CanisterName::Local("water_neuron".to_string()),
    ];

    for canister in &canisters {
        print!("{:?}... ", canister);
        match get_wasm(canister.clone()) {
            Ok(_) => println!("✓"),
            Err(e) => println!("✗ ({})", e),
        }
    }

    Ok(())
}
