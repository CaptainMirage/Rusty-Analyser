use crate::analyzer::{
    StorageAnalyzer,
    constants::*  // Import constants from the module
};
use lazy_static::lazy_static;
use std::{
    collections::HashSet,
    env,
    io::{self, Write},
    process,
};

/*
    let mut analyzer: StorageAnalyzer = StorageAnalyzer::new();
    for drive in &analyzer.drives.clone() {
        analyzer.analyze_drive(drive)?;
    }
 */

pub fn bash_commands() {
    // Define the HashSet of commands
    lazy_static! {
        static ref BUILTIN_COMMANDS: HashSet<&'static str> = {
            vec!["exit", "echo", "type", "pwd"]
                .into_iter()
                .collect()
        };
    }

    print!("$ ");
    io::stdout().flush().unwrap();

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
            print!("$ ");
            io::stdout().flush().unwrap();
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
            ["oldlargefiles", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for OldLargeFiles"),
            },
            ["driveanalysis", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs for DriveAnalysis"),
            },

            
            _ => {
                println!("{}: not found", command[0]);
            }
        }
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
    }
}
