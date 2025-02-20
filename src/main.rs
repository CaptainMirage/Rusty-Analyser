mod analyzer;
mod shell;

use analyzer::{
    StorageAnalyzer,
    constants::*  // Import constants from the module
};
use console::Term;

// Constants can stay here or move to analyzer/constants.rs if you prefer

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
        println!("DEV PROFILE : Running in debug mode!");
        println!("if you are a normal user, consider using cargo run --release");
    }

    #[cfg(not(debug_assertions))]
    {
        println!("-- INFO --");
        println!("  RELEASE PROFILE : Running in release mode!");
        println!("  Optimizations enabled, debug off, overflow checks off");
    }

    #[cfg(feature = "DEBUG_MODE")]
    {
        println!("--- WARNING ---");
        println!("DEBUG MODE : Running in debug mode!");
        return debug_test();
    }

    let mut analyzer: StorageAnalyzer = StorageAnalyzer::new();
    for drive in &analyzer.drives.clone() {
        analyzer.analyze_drive(drive)?;
    }

    let term = Term::stdout();
    println!("\nPress any key to exit...");
    term.read_char()?;

    Ok(())
}