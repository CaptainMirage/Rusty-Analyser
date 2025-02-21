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
        let command: Vec<_> = input.trim().split_whitespace().collect();

        if command.is_empty() {
            input.clear();
            print!("$ ");
            io::stdout().flush().unwrap();
            continue;
        }

        match command[..] {
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
            ["get_drive_space", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs dumbass"),
            },
            ["get_file_type_distribution", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs dumbass"),
            },
            ["get_largest_files", ..] => match command.get(1) {
                Some(drive_space) => println!("{}", drive_space),
                None => println!("didnt put any inputs dumbass"),
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
