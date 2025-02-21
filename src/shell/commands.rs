use crate::analyzer::{
    StorageAnalyzer,
    constants::*  // Import constants from the module
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
    let user = whoami::username();
    let host = fallible::hostname().unwrap();
    let prompt= format!(
        "\n{}@{}\n{} ",
        user.green(),
        host.blue(),
        "$".yellow()
    );
    print!("{}", prompt);
    io::stdout().flush().unwrap();
}

pub fn bash_commands() {
    // Define the HashSet of commands
    lazy_static! {
        static ref BUILTIN_COMMANDS: HashSet<&'static str> = {
            vec!["exit", "echo", "type", "pwd"]
                .into_iter()
                .collect()
        };
    }

    prompter_fn();

    // Wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
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
            ["drivespace", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for DriveSpace"),
            },
            ["filetypedist", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for FileTypeDist"),
            },
            ["largestfiles", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for LargestFiles"),
            },
            ["largestfolder", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for LargestFolder"),
            },
            ["recentlargefiles", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for RecentLargeFiles"),
            },
            ["old-large-files", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for OldLargeFiles"),
            },
            ["drive-analysis", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
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
