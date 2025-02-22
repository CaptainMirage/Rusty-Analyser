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
    F: FnOnce(&str) -> Result<(), std::io::Error>,
{
    let drive = drive.to_uppercase();

    if drive.len() == 1 && drive.chars().all(|c| c.is_ascii_alphabetic()) {
        let formatted_drive = format!("{}:/", drive);
        if let Err(e) = action(formatted_drive.as_str()) { 
            eprintln!("Error: {}", e);
        }
    } else if drive.len() == 3 && drive.ends_with(":/") &&
        drive.chars().next().unwrap().is_ascii_alphabetic() {
        if let Err(e) = action(drive.as_str()) {
            eprintln!("Error: {}", e);
        }
    } else {
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
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_drive_analysis(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            ["file-type-dist", ..] => match command.get(1) {
                    Some(drive) => validate_and_format_drive
                        (drive, |d| analyzer.print_drive_analysis(d)),
                    None => println!("didnt put any inputs for DriveSpace"),
                }
            
            ["largest-files", ..] => match command.get(1) {
                    Some(drive) => validate_and_format_drive
                        (drive, |d| analyzer.print_drive_analysis(d)),
                    None => println!("didnt put any inputs for DriveSpace"),
                }
            
            ["largest-folder", ..] => match command.get(1) {
                    Some(drive) => validate_and_format_drive
                        (drive, |d| analyzer.print_drive_analysis(d)),
                    None => println!("didnt put any inputs for DriveSpace"),
                }
            
            ["recent-large-files", ..] => match command.get(1) {
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_drive_analysis(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            ["old-large-files", ..] => match command.get(1) {
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_drive_analysis(d)),
                None => println!("didnt put any inputs for DriveSpace"),
            }
            
            ["drive-analysis", ..] => match command.get(1) {
                Some(drive) => validate_and_format_drive
                    (drive, |d| analyzer.print_drive_analysis(d)),
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
