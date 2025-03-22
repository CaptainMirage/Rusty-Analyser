mod analyser;
mod shell;
mod utility;
mod scanner;
// use crate::utility::funzy::display_boot_sequence;
use utility::constants::*;

#[cfg(feature = "DEBUG_MODE")]
fn debug_test() -> std::io::Result<()> {
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
    //display_boot_sequence();

    shell::bash_commands();
    Ok(())
}
