mod analyzer;
mod shell;
use crate::analyzer::{
    constants::* 
};

#[cfg(feature = "DEBUG_MODE")]
fn debug_test() -> std::io::Result<()> {
    let mut analyzer = StorageAnalyzer::new();
    analyzer.print_recent_large_files("C:\\")?;
    analyzer.print_old_large_files("C:\\")?;
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
    shell::bash_commands();
    Ok(())
}