use ic_wasm_utils::{get_wasm, CanisterName, Error};

fn main() -> Result<(), Error> {
    let wasm = get_wasm(CanisterName::Sns)?;
    println!("Successfully downloaded {} bytes", wasm.len());
    
    Ok(())
}
