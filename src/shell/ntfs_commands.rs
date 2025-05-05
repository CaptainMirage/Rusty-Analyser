use super::help_cmd::*;
use crate::analyser::ntfs_explorer::NtfsExplorer;
use crate::utility::utils::{save_empty_folders_to_file, time_command, validate_and_format_drive};
use colored::Colorize;
use std::{
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

pub fn ntfs_bash_commands() {
    prompter_fn();

    // wait for user input
    let stdin = io::stdin();
    let mut input = String::new();
    let explorer = NtfsExplorer::new();
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
                Some(drive) => match explorer.print_drive_space(drive) {
                    Ok(()) => { /* all good, nothing else to do, or is there? */ }
                    Err(e) => { eprintln!("{}", e) },
                },
                None => println!(
                    "drive letter required. Usage: drive-space [drive]"),
            },

            ["file-type-dist", ..] => match command.get(1) {
                Some(drive) => explorer.print_file_type_dist(drive, 20).unwrap(),
                None => println!(
                    "drive letter required. Usage: file-type-dist [drive]"),
            },

            ["largest-files", ..] => match command.get(1) {
                Some(drive) => explorer.print_largest_files(drive, 20).unwrap(),
                None => println!(
                    "drive letter required. Usage: largest-files [drive]"),
            },

            ["largest-folder", ..] => match command.get(1) {
                Some(drive) => explorer.print_largest_folders(drive, 20).unwrap(),
                None => println!(
                    "drive letter required. Usage: largest-folder [drive]"),
            },

            ["recent-large-files", ..] => match command.get(1) {
                Some(drive) => explorer.print_recent_large_files(drive, 20).unwrap(),
                None => println!(
                    "drive letter required. Usage: recent-large-files [drive]"
                ),
            },

            ["old-large-files", ..] => match command.get(1) {
                Some(drive) => explorer.print_old_large_files(drive, 20).unwrap(),
                None => println!(
                    "drive letter required. Usage: old-large-files [drive]"
                ),
            },

            ["full-drive-analysis", ..] => match command.get(1) {
                Some(_drive) => println!("not implemented yet even tho it takes only a few minutes"),
                None => println!(
                    "drive letter required. Usage: full-drive-analysis [drive]"),
            },

            ["empty-folders", ..] => {
                if command.contains(&"-delete".to_string()) {
                    // placeholder for later implementation
                    println!("Deletion functionality for empty folders is not yet implemented.");
                } else {
                    match command.get(1) {
                        Some(drive) => explorer.print_empty_folders(drive, 20).unwrap(),
                        None => println!(
                            "drive letter required. Usage: empty-folders [drive]"),
                    }
                }
            }

            _ => {
                println!("{}: command not found", command[0]);
            }
        }
        input.clear();
        prompter_fn();
    }
}
