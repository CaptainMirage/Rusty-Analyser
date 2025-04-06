mod analyser;
mod shell;
mod utility;
use crate::{
    utility::{
        funzy::display_boot_sequence,
        constants::*
    },

};

fn main() -> Result<(), Box<dyn std::error::Error>>  {
    #[cfg(debug_assertions)]
    {
        println!("--- WARNING ---");
        println!("DEV PROFILE : Running dev profile!");
        println!("if you are a normal user, consider using cargo run --release\n\n\n");
    }

    let flag: bool = true;
    if flag {
        // where the main code will run
        display_boot_sequence();

        shell::bash_commands();
    }
    
    Ok(())
}
