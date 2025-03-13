use super::types::*;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};

macro_rules! add_command {
    ($reg:ident, $name:expr, title: $title:expr, cmd_args: $args:expr, description: $desc:expr $(,)?) => {{
        $reg.0.insert($name);
        $reg.1.insert($name, CommandInfo {
            title: $title,
            cmd_args: $args,
            description: $desc,
        });
    }};
}

lazy_static! {
    // Create a tuple containing built-in command names (HashSet) and ordered command descriptions (IndexMap).
    static ref COMMANDS: (HashSet<&'static str>, IndexMap<&'static str, CommandInfo>) = {
        let mut m = (HashSet::new(), IndexMap::new());

        add_command!{
            m, "help",
            title      : "Help",
            cmd_args   : "",
            description: "Displays all commands descriptions \n\
                        if an argument is given, it gives the command description of the said argument",
        }
        add_command!{
            m, "exit",
            title      : "Exit",
            cmd_args   : "",
            description: "hey, you, yes you, if you can read this and understand it, \n\
                        then there is no need for an explanation of what this command does",
        }
        add_command!{
            m, "echo",
            title      : "Echo",
            cmd_args   : "",
            description: "Repeats what you say, probably",
        }
        add_command!{
            m, "type",
            title      : "Type",
            cmd_args   : "",
            description: "It just tells you if the command exists",
        }
        add_command!{
            m, "pwd",
            title      : "pwd",
            cmd_args   : "",
            description: "Shows the location the program is ran in",
        }
        add_command!{
            m, "drive-space",
            title      : "Drive Space",
            cmd_args   : "",
            description: "Shows the amount of space in a drive, what else do you want?",
        }
        add_command!{
            m, "file-type-dist",
            title      : "File Type Distribution",
            cmd_args   : "",
            description: "Shows the distribution of the 10 file formats taking the largest space",
        }
        add_command!{
            m, "Error-680089",
            title      : "????????",
            cmd_args   : "",
            description: "",
        }
        add_command!{
            m, "largest-files",
            title      : "Largest Files",
            cmd_args   : "",
            description: "Shows the top 10 largest files",
        }
        add_command!{
            m, "largest folder",
            title      : "Largest Folder",
            cmd_args   : "",
            description: "Shows the top 10 largest folders up to 3 levels deep \n\
                        Excludes hidden folders (those starting with '.')",
        }
        add_command!{
            m, "recent-large-files",
            title      : "Recent Large Files",
            cmd_args   : "",
            description: "Shows most recent files within last 30 days that are large",
        }
        add_command!{
            m, "old-large-files",
            title      : "Old Large Files",
            cmd_args   : "",
            description: "Shows older than 6 months files that are your m- i mean large",
        }
        add_command!{
            m, "full-drive-analysis",
            title      : "Full Drive Analysis",
            cmd_args   : "",
            description: "cant you read?",
        }
        add_command!{
            m, "empty-folders",
            title      : "Empty Folders",
            cmd_args   : "",
            description: "searches for empty folders and lists them all\
                        (not all empty folders should be deleted) \n\
                        if you're not sure just search the folder path and see if you can delete it"
        }
        m
    };
    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> = COMMANDS.0.clone();
    pub static ref COMMAND_DESCRIPTIONS: IndexMap<&'static str, CommandInfo> = COMMANDS.1.clone();
}
