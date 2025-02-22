use crate::analyzer::{
    StorageAnalyzer,
    constants::*
};
use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    io::{self, Write},
    process,
};
use whoami::fallible;
/*
    let mut analyzer: StorageAnalyzer = StorageAnalyzer::new();
    for drive in &analyzer.drives.clone() {
        analyzer.analyze_drive(drive)?;
    }
 */
fn prompter_fn() {
    let _user: String = whoami::username();
    let _host: String = fallible::hostname().unwrap();
    let prompt: String = format!(
        "\n{}{}{}\n{} ",
        "user".green(),
        "@".white(),
        "host".blue(),
        "$".cyan()
    );
    print!("{}", prompt);
    io::stdout().flush().unwrap();
}

pub fn bash_commands() {
    // Define the HashSet of commands
    lazy_static! {
        static ref BUILTIN_COMMANDS: HashSet<&'static str> = {
            vec![
                "exit", "echo", "type", "pwd", "drive-space",
                "file-type-dist", "largest-files", "largest-folder",
                "recent-large-files", "old-large-files", "drive-analysis"
            ]
                .into_iter()
                .collect()
        };
    }

    prompter_fn();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    let mut analyzer: StorageAnalyzer = StorageAnalyzer::new();
    loop {
        stdin.read_line(&mut input).unwrap();
        let command: Vec<String> = input
            .trim()
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();

        if command.is_empty() {
            input.clear();
            prompter_fn();
            continue;
        }

        match command.iter().map(|s| s.as_str()).collect::<Vec<_>>()[..] {
            // some default commands
            ["exit", ..] => match command.get(1) {
                Some(code) => process::exit(code.parse::<i32>().unwrap()),
                None => process::exit(0),  // Default exit code if none provided
            },
            ["echo", ..] => match command.get(1..) {
                Some(words) => println!("{}", words.join(" ")),
                None => println!(),  // Just print newline if no arguments given
            },
            ["pwd"] => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => println!("pwd: error getting current directory: {}", e),
            },
            
            // drive analysis commands
            ["drive-space", ..] => match command.get(1) {
                Some(drive) => {
                    let drive = drive.to_uppercase();

                    if drive.len() == 1 && drive.chars().all(|c| c.is_ascii_alphabetic()) {
                        // User entered just the letter (e.g., "C"), format it properly
                        let formatted_drive = format!("{}:/", drive);
                        if let Err(e) = analyzer.print_drive_analysis(&formatted_drive) {
                            eprintln!("Error: {}", e);
                        }
                    } else if drive.len() == 3 && drive.ends_with(":/") && drive.chars().next().unwrap().is_ascii_alphabetic() {
                        // User entered a valid full path (e.g., "C:/"), use it directly
                        if let Err(e) = analyzer.print_drive_analysis(&drive) {
                            eprintln!("Error: {}", e);
                        }
                    } else {
                        // Invalid input, show an error message
                        eprintln!("Invalid drive format. Please enter a single letter (e.g., 'C') or a valid drive path (e.g., 'C:/').");
                    }
                }
                None => println!("didnt put any inputs for DriveSpace"),
            },
            
            ["file-type-dist", ..] => match command.get(1) {
                Some(drive) => println!("{}", drive),
                None => println!("didnt put any inputs for FileTypeDist"),
            },
            ["largest-files", ..] => match command.get(1) {
                Some(drive) => println!("{}", drive),
                None => println!("didnt put any inputs for LargestFiles"),
            },
            ["largest-folder", ..] => match command.get(1) {
                Some(drive) => println!("{}", drive),
                None => println!("didnt put any inputs for LargestFolder"),
            },
            ["recent-large-files", ..] => match command.get(1) {
                Some(drive) => println!("{}", drive),
                None => println!("didnt put any inputs for RecentLargeFiles"),
            },
            ["old-large-files", ..] => match command.get(1) {
                Some(drive) => println!("{}", drive),
                None => println!("didnt put any inputs for OldLargeFiles"),
            },
            ["drive-analysis", ..] => match command.get(1) {
                Some(drive) => println!("{}", drive),
                None => println!("didnt put any inputs for DriveAnalysis"),
            },
            
            
            _ => {
                println!("{}: not found", command[0]);
            }
        }
        input.clear();
        prompter_fn();
    }
}
