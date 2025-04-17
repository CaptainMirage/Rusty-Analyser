mod analyser;
mod shell;
mod utility;
use crate::utility::{constants::*, funzy::display_boot_sequence};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(debug_assertions)]
    {
        println!("--- WARNING :: DEV PROFILE ACTIVE ---");
        println!("if you are a normal user, consider using cargo run --release\n\n\n");
    }

    let flag: bool = false;
    if flag {
        // where the main code will run
        display_boot_sequence();

        shell::bash_commands();
    }
    shell::ntfs_bash_commands();

    Ok(())
}
