use ic_wasm_utils::{get_wasm, CanisterName, Error};

fn main() -> Result<(), Error> {
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
    ];

    println!("Checking all canisters...\n");

    for canister in &canisters {
        println!("Canister: {:?}", canister);
        
        print!("Downloading... ");
        match get_wasm(canister.clone()) {
            Ok(bytes) => println!("✓ ({} bytes)", bytes.len()),
            Err(e) => println!("✗\n  Error: {}", e),
        }
        println!();
    }

    println!("All downloads completed");
    Ok(())
}
