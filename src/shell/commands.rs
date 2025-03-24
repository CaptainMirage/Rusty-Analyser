use super::help_cmd::*;
use crate::analyser::StorageAnalyser;
use crate::utility::utils::{save_empty_folders_to_file, validate_and_format_drive, time_command};
use colored::Colorize;
use std::{
    env,
    io::{self, Write},
    process,
};
use whoami::fallible;
use crate::analyser;

macro_rules! vfd {
    // pattern to accept an arbitrary closure block
    ($drive:expr, $action:expr) => {
        validate_and_format_drive($drive, $action)
    };
    // pattern for the simpler case of passing an instance and method name
    ($drive:expr, $instance:expr, $method:ident) => {
        validate_and_format_drive($drive, |d| $instance.$method(d))
    };
}

fn prompter_fn() {
    let _user: String = whoami::username();
    let _host: String = fallible::hostname().unwrap();
    let prompt: String = format!(
        "\n{}{}{}\n{} ",
        _user.bright_green(),
        "@".bright_white(),
        _host.bright_blue(),
        "$".bright_cyan()
    );
    print!("{}", prompt);
    io::stdout().flush().unwrap();
}

fn print_command_help(command: &String) {
    if let Some(info) = COMMAND_DESCRIPTIONS.get(command.as_str()) {
        print!(
            "\n\
            {}\n\
            {}\n\
            {}\n",
            //info.title.bright_white(),
            info.cmd_args.bright_blue(),
            "-------------".green().bold(),
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

    for (_, info) in COMMAND_DESCRIPTIONS.iter() {
        print!(
            "\n\
            {}\n\
            {}\n\
            {}\n",
            //info.title.bright_white(),
            info.cmd_args.bright_blue(),
            "-------------".green().bold(),
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
    let mut analyser: StorageAnalyser = StorageAnalyser::new();
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
                None => process::exit(0), // Default exit code if none provided
            },
            ["echo", ..] => match command.get(1..) {
                Some(words) => {
                    if words == ["i", "am", "an", "idiot"] {
                        println!("you are an idiot")
                    } else {
                        println!("{}", words.join(" "))
                    }
                }
                None => println!(), // prints a newline if no arguments given
            },
            ["pwd"] => match env::current_dir() {
                Ok(path) => println!("{}", path.display()),
                Err(e) => println!("pwd: error getting current directory: {}", e),
            },
            ["help", ..] => match command.get(1) {
                Some(cword) => print_command_help(cword),
                None => print_all_help(),
            },

            // drive analysis commands
            ["drive-space", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, analyser, print_drive_space_overview),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["file-type-dist", ..] => match command.get(1) {
                Some(drive) => time_command(|| { vfd!(drive, analyser, print_file_type_distribution)}),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["largest-files", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, analyser, print_largest_files),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["largest-folder", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, analyser, print_largest_folders),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["recent-large-files", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, analyser, print_recent_large_files),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["old-large-files", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, analyser, print_old_large_files),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["full-drive-analysis", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, analyser, analyze_drive),
                None => println!(
                    "drive letter required. Usage: {} [drive]",
                    command.get(0).unwrap()
                ),
            },

            ["empty-folders", ..] => {
                if command.contains(&"-delete".to_string()) {
                    // placeholder for later implementation
                    println!("Deletion functionality for empty folders is not yet implemented.");
                } else {
                    match command.get(1) {
                        Some(drive) => vfd!(drive, |d| {
                            let empty_folders = analyser.get_empty_folders(d)?;
                            println!("Found {} empty folders.", empty_folders.len());
                            for folder in &empty_folders {
                                println!(" - {}", folder);
                            }
                            save_empty_folders_to_file(&empty_folders)?;
                            Ok(())
                        }),
                        None => println!(
                            "drive letter required. Usage: {} [drive]",
                            command.get(0).unwrap()
                        ),
                    }
                }
            }

        ["rescan", ..] => match command.get(1) {
                Some(drive) => vfd!(drive, |d| {
                    time_command(|| {
                        analyser.rescan_drive(d)?;
                        println!("Rescan complete for drive {}", d);
                        Ok(())
                    })
                }),
                None => println!("Drive letter required. Usage: rescan [drive]"),
            },
            
            _ => {
                println!("{}: command not found", command[0]);
            }
        }   
        input.clear();
        prompter_fn();
    }
}
