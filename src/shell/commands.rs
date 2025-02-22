use crate::analyzer::{
    StorageAnalyzer,
    constants::*
};
use super::{
    types::*
};
use std::{
    collections::HashSet,
    env,
    collections::HashMap,
    io::{self, Write},
    process,
};
use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;
use whoami::fallible;


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

fn validate_and_format_drive<F>(drive: &str, action: F)
where
    F: FnOnce(&str) -> Result<(), io::Error>,
{
    let drive = drive.to_uppercase();
    
    if drive.len() == 1 && drive.chars().all(|c| c.is_ascii_alphabetic()) {
        // User entered just the letter (e.g., "C"), format it properly
        if let Err(e) = action(format!("{}:/", drive).as_str()) { 
            eprintln!("Error: {}", e);
        }
    } else if drive.len() == 3 && drive.ends_with(":/") &&
        drive.chars().next().unwrap().is_ascii_alphabetic() {
        // User entered a valid full path (e.g., "C:/"), use it directly
        if let Err(e) = action(drive.as_str()) {
            eprintln!("Error: {}", e);
        }
    } else {
        // Invalid input
        eprintln!("Invalid drive format. Please enter a single letter (e.g., 'C')\
         or a valid drive path (e.g., 'C:/').");
    }
}


pub fn bash_commands() {
    // Define the HashSet of commands
    lazy_static! {
        static ref BUILTIN_COMMANDS: HashSet<&'static str> = {
            vec![
                "exit", "echo", "type", "pwd", "drive-space",
                "file-type-dist", "largest-files", "largest-folder",
                "recent-large-files", "old-large-files", "full-drive-analysis"
            ]
                .into_iter()
                .collect()
        };
        // Define the HashSet of command descs
        static ref COMMAND_DESCRIPTIONS: HashMap<&'static str, CommandInfo> = {
            let mut m = HashMap::new();
            m.insert("help", CommandInfo {
                title: "Lorem ipsum",
                description: ""
            });
            m.insert("exit", CommandInfo {
                title: "Oder amet",
                description: ""
            });
            m
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
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_drive_space_overview(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            ["file-type-dist", ..] => match command.get(1) {
                    Some(drive) => validate_and_format_drive
                        (drive, |d| analyzer.print_file_type_distribution(d)),
                    None => println!("didnt put any inputs for DriveSpace"),
                }
            
            ["largest-files", ..] => match command.get(1) {
                    Some(drive) => validate_and_format_drive
                        (drive, |d| analyzer.print_largest_files(d)),
                    None => println!("didnt put any inputs for DriveSpace"),
                }
            
            ["largest-folder", ..] => match command.get(1) {
                    Some(drive) => validate_and_format_drive
                        (drive, |d| analyzer.print_largest_folders(d)),
                    None => println!("didnt put any inputs for DriveSpace"),
                }
            
            ["recent-large-files", ..] => match command.get(1) {
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_recent_large_files(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            ["old-large-files", ..] => match command.get(1) {
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_old_large_files(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            ["full-drive-analysis", ..] => match command.get(1) {
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.analyze_drive(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            _ => {
                println!("{}: not found", command[0]);
            }
        }
        input.clear();
        prompter_fn();
    }
}
