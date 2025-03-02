use crate::analyzer::{
    StorageAnalyzer,
    constants::*
};
use super::{
    help_cmd::*
};
use std::{
    env,
    io::{self, Write},
    process,
};
use colored::{ColoredString, Colorize};
use lazy_static::lazy_static;
use rayon::max_num_threads;
use whoami::fallible;


fn prompter_fn() {
    let _user: String = whoami::username();
    let _host: String = fallible::hostname().unwrap();
    let prompt: String = format!(
        "\n{}{}{}\n{} ",
        _user.green(),
        "@".white(),
        _host.blue(),
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
        // user entered just the letter (e.g., "C"), format it properly
        if let Err(e) = action(format!("{}:/", drive).as_str()) { 
            eprintln!("Error: {}", e);
        }
    } else if drive.len() == 3 && drive.ends_with(":/") &&
        drive.chars().next().unwrap().is_ascii_alphabetic() {
        // user entered a valid full path (e.g., "C:/"), use it directly
        if let Err(e) = action(drive.as_str()) {
            eprintln!("Error: {}", e);
        }
    } else {
        // invalid input
        eprintln!("Invalid drive format. Please enter a single letter (e.g., 'C')\
         or a valid drive path (e.g., 'C:/').");
    }
}

fn print_command_help(command: &String) {
        if let Some(info) = COMMAND_DESCRIPTIONS.get(command.as_str()) {
            print!("\n{}\n-------------\n{}\n",
                     info.title.bright_white(),
                     info.description
            );
        } else {
            println!("Command not found: {}", command);
        }
}

fn print_all_help() {
    // for if I want to sort it alphabetically (probably still works, probably) :
    // let mut commands: Vec<_> = COMMAND_DESCRIPTIONS.iter().collect();
    // commands.sort_by_key(|(cmd, _)| *cmd);

    for (_, info) in COMMAND_DESCRIPTIONS.iter()  {
        print!("\n{}\n-------------\n{}",
               info.title.bright_white(),
               info.description
        );
        println!(); // add an extra newline between commands
    }
}

pub fn bash_commands() {
    
    prompter_fn();

    // wait for user input
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
                Some(words) => if words == ["i", "am", "an", "idiot"] {
                    println!("you are an idiot")
                } else { println!("{}", words.join(" ")) },
                None => println!(),  // Just print newline if no arguments given
            },
            ["pwd"] => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => println!("pwd: error getting current directory: {}", e),
            },
            ["help", ..] => match command.get(1) {
                Some(cword) => print_command_help(cword),
                None => print_all_help(),
            }
            
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

            ["empty-folders", ..] => {
                if command.contains(&"-delete".to_string()) {
                    // placeholder for later implementation
                    println!("Deletion functionality for empty folders is not yet implemented.");
                } else {
                    match command.get(1) {
                        Some(drive) => validate_and_format_drive(drive, |d| analyzer.print_empty_folders(d)),
                        None => println!("No drive provided for empty-folders command."),
                    }
                }
            }
            
            _ => {
                println!("{}: not found", command[0]);
            }
        }
        input.clear();
        prompter_fn();
    }
}
