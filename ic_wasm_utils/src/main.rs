use ic_wasm_utils::*;

fn main() {
    println!("Testing boomerang...");
    println!("Size: {} bytes", boomerang_wasm().len());

    println!("\nTesting water_neuron...");
    println!("Size: {} bytes", water_neuron_wasm().len());

    println!("\nTesting icp_ledger...");
    println!("Size: {} bytes", icp_ledger_wasm().len());

    println!("\nTesting governance...");
    println!("Size: {} bytes", governance_wasm().len());

    println!("\nTesting ledger...");
    println!("Size: {} bytes", ledger_wasm().len());

    println!("\nTesting sns_governance...");
    println!("Size: {} bytes", sns_governance_wasm().len());

    println!("\nTesting sns_root...");
    println!("Size: {} bytes", sns_root_wasm().len());

    println!("\nTesting sns_swap...");
    println!("Size: {} bytes", sns_swap_wasm().len());
}
