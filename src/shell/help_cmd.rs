use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use super::{
    types::*
};

// Define the macro to add a command to the registry tuple.
macro_rules! add_command {
    ($reg:ident, $name:expr, title: $title:expr, description: $desc:expr $(,)?) => {{
        $reg.0.insert($name);
        $reg.1.insert($name, CommandInfo { title: $title, description: $desc });
    }};
}


lazy_static! {
    static ref COMMANDS: (HashSet<&'static str>, HashMap<&'static str, CommandInfo>) = {
        // 'm' is our registry: a tuple of (HashSet, HashMap)
        let mut m = (HashSet::new(), HashMap::new());

        add_command!{
          m, "help",
          title      : "Help",
          description: "Displays all commands descriptions \n\
                        if an argument is given, it gives the command description of the said argument",
        }
        add_command!{
          m, "exit",
          title      : "Exit",
          description: "Exits the program. No explanation needed.",
        }
        m
    };

    pub static ref BUILTIN_COMMANDS: HashSet<&'static str> = COMMANDS.0.clone();
    pub static ref COMMAND_DESCRIPTIONS: HashMap<&'static str, CommandInfo> = COMMANDS.1.clone();
}