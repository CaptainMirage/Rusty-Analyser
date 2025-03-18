mod analyser;
mod shell;
mod utility;
// use crate::Analyser::utils::tester_function;
use utility::constants::*;
use crate::utility::funzy::{display_boot_sequence};

#[cfg(feature = "DEBUG_MODE")]
fn debug_test() -> std::io::Result<()> {
    let mut Analyser = StorageAnalyser::new();
    Analyser.print_recent_large_files("C:\\")?;
    Analyser.print_old_large_files("C:\\")?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    #[cfg(debug_assertions)]
    {
        println!("--- WARNING ---");
        println!("DEV PROFILE : Running dev profile!");
        println!("if you are a normal user, consider using cargo run --release\n\n\n");
    }

    #[cfg(feature = "DEBUG_MODE")]
    {
        println!("--- WARNING ---");
        println!("DEBUG MODE : Running debug function!");
        return debug_test();
    }

    // where the main code will run
    display_boot_sequence();

    shell::bash_commands();
    Ok(())
}
