use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use super::{
    types::*
};

// Define the HashSet of commands & command descriptions
lazy_static! {
        pub static ref BUILTIN_COMMANDS: HashSet<&'static str> = {
            vec![
                "exit", "echo", "type", "pwd", "drive-space",
                "file-type-dist", "largest-files", "largest-folder",
                "recent-large-files", "old-large-files", "full-drive-analysis"
            ]
                .into_iter()
                .collect()
        };
        pub static ref COMMAND_DESCRIPTIONS: HashMap<&'static str, CommandInfo> = {
            let mut m = HashMap::new();
            m.insert("help", CommandInfo {
                title: "Help",
                description: "Lorem ipsum odor amet, consectetuer adipiscing elit. \n\
                Pellentesque porttitor finibus donec facilisi montes, tristique cras mauris?"
            });
            m.insert("exit", CommandInfo {
                title: "Exit",
                description: "Lorem ipsum odor amet, consectetuer adipiscing elit. \n\
                 Amet cras hendrerit aenean elementum platea curabitur pellentesque!"
            });
            m
        };
    }