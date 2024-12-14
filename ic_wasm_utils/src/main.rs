use ic_wasm_utils::{get_wasm, CanisterName, Error};
use std::time::Instant;

fn main() -> Result<(), Error> {
    let start = Instant::now();
    
    // All canister types to process
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

    println!("\nProcessing canisters...\n");

    let mut success_count = 0;
    let mut error_count = 0;

    for canister in &canisters {
        print!("{}... ", format!("{:?}", canister).cyan());
        match get_wasm(canister.clone()) {
            Ok(bytes) => {
                success_count += 1;
                println!("{} ({} bytes)", "✓".green(), bytes.len().to_string().yellow());
            }
            Err(e) => {
                error_count += 1;
                println!("{}", "✗".red());
                println!("  {}", format!("Error: {}", e).red());
            }
        }
    }

    let elapsed = start.elapsed();
    println!("\nSummary:");
    println!("  Time: {:.2}s", elapsed.as_secs_f64());
    println!("  Successful: {}", success_count.to_string().green());
    if error_count > 0 {
        println!("  Failed: {}", error_count.to_string().red());
    }
    println!();

    // Return error if any canister failed
    if error_count > 0 {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("{} canister(s) failed to process", error_count)
        )))
    } else {
        Ok(())
    }
}

// Simple color helpers
trait ColorString {
    fn red(&self) -> String;
    fn green(&self) -> String;
    fn yellow(&self) -> String;
    fn cyan(&self) -> String;
}

impl<T: std::fmt::Display> ColorString for T {
    fn red(&self) -> String {
        format!("\x1b[31m{}\x1b[0m", self)
    }
    fn green(&self) -> String {
        format!("\x1b[32m{}\x1b[0m", self)
    }
    fn yellow(&self) -> String {
        format!("\x1b[33m{}\x1b[0m", self)
    }
    fn cyan(&self) -> String {
        format!("\x1b[36m{}\x1b[0m", self)
    }
}
