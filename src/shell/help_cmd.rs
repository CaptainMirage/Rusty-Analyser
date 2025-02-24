use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use super::{
    types::*
};

// Define the HashSet of commands & command descriptions
lazy_static! {
        pub static ref BUILTIN_COMMANDS: HashSet<&'static str> = {
            vec![
                "exit", "echo", "type", "pwd", "help", "drive-space",
                "file-type-dist", "largest-files", "largest-folder",
                "recent-large-files", "old-large-files", "full-drive-analysis"
            ]
                .into_iter()
                .collect()
        };
    // make this into what master said to do
        pub static ref COMMAND_DESCRIPTIONS: HashMap<&'static str, CommandInfo> = {
            let mut m = HashMap::new();
            m.insert("help", CommandInfo {
                title: "Help",
                description: "Displays all commands descriptions \n\
                if an argument is given, it gives the command description of the said argument"
            });
            m.insert("exit", CommandInfo {
                title: "Exit",
                description: "hey, you, yes you, if you can read this and understand it, \n\
                then there is no need for an explanation of what this command does"
            });
            m.insert("echo", CommandInfo {
            title: "Echo",
            description: "Repeats what you say, probably"
            });
            m.insert("type", CommandInfo {
            title: "Type",
            description: "It just tells you if the command exists"
            });
            m.insert("pwd", CommandInfo {
            title: "pwd",
            description: "Shows the location the program is ran in"
            });
            m.insert("drive-space", CommandInfo {
            title: "Drive Space",
            description: "Shows the amount of space in a drive, what else do you want?"
            });
            m.insert("file-type-dist", CommandInfo {
            title: "File Type Distribution",
            description: "Shows the distribution of the 10 file formats taking the largest space"
            });
            m.insert("largest-files", CommandInfo {
            title: "Largest Files",
            description: "Shows the top 10 largest files"
            }); 
            m.insert("largest folder", CommandInfo {
            title: "Largest Folder",
            description: "Shows the top 10 largest folders up to 3 levels deep \n\
            (Excludes hidden folders [those starting with '.'])"
            });
            m.insert("recent-large-files", CommandInfo {
            title: "Recent Large Files",
            description: "Shows most recent files within last 30 days that are large"
            });
            m.insert("old-large-files", CommandInfo {
            title: "Old Large Files",
            description: "Shows older than 6 months files that are your m- i mean large"
            });
            m.insert("full-drive-analysis", CommandInfo {
            title: "Full Drive Analysis",
            description: "cant you read?"
            });
            m.insert("temp-680089", CommandInfo {
            title: "????????",
            description: ""
            });
            m
        };
    }