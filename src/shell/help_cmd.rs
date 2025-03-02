use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use indexmap::IndexMap;
use super::{
    types::*
};

macro_rules! add_command {
    ($reg:ident, $name:expr, title: $title:expr, description: $desc:expr $(,)?) => {{
        $reg.0.insert($name);
        $reg.1.insert($name, CommandInfo { title: $title, description: $desc });
    }};
}

lazy_static! {
    // Create a tuple containing built-in command names (HashSet) and ordered command descriptions (IndexMap).
    static ref COMMANDS: (HashSet<&'static str>, IndexMap<&'static str, CommandInfo>) = {
        let mut m = (HashSet::new(), IndexMap::new());

        add_command!{
          m, "help",
          title      : "Help",
          description: "Displays all commands descriptions \n\
                        if an argument is given, it gives the command description of the said argument",
        }
        add_command!{
          m, "exit",
          title      : "Exit",
          description: "hey, you, yes you, if you can read this and understand it, \n\
                        then there is no need for an explanation of what this command does",
        }
        add_command!{
          m, "echo",
          title      : "Echo",
          description: "Repeats what you say, probably",
        }
        add_command!{
          m, "type",
          title      : "Type",
          description: "It just tells you if the command exists",
        }
        add_command!{
          m, "pwd",
          title      : "pwd",
          description: "Shows the location the program is ran in",
        }
        add_command!{
          m, "drive-space",
          title      : "Drive Space",
          description: "Shows the amount of space in a drive, what else do you want?",
        }
        add_command!{
          m, "file-type-dist",
          title      : "File Type Distribution",
          description: "Shows the distribution of the 10 file formats taking the largest space",
        }
        add_command!{
          m, "Error-680089",
          title      : "????????",
          description: "",
        }
        add_command!{
          m, "largest-files",
          title      : "Largest Files",
          description: "Shows the top 10 largest files",
        }
        add_command!{
          m, "largest folder",
          title      : "Largest Folder",
          description: "Shows the top 10 largest folders up to 3 levels deep \n\
                        Excludes hidden folders (those starting with '.')",
        }
        add_command!{
          m, "recent-large-files",
          title      : "Recent Large Files",
          description: "Shows most recent files within last 30 days that are large",
        }
        add_command!{
          m, "old-large-files",
          title      : "Old Large Files",
          description: "Shows older than 6 months files that are your m- i mean large",
        }
        add_command!{
          m, "full-drive-analysis",
          title      : "Full Drive Analysis",
          description: "cant you read?",
        }
        add_command!{
            m, "empty-folders",
            title: "Empty Folders",
            description: "searches for empty folders and lists them all\
                          (not all empty folders should be deleted) \n\
                          if you're not sure just search the folder path and see if you can delete it"
        }
        m
    };
    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> = COMMANDS.0.clone();
    pub static ref COMMAND_DESCRIPTIONS: IndexMap<&'static str, CommandInfo> = COMMANDS.1.clone();
}