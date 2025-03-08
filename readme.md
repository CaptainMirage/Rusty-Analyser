# Rusty Analyser

![Static Badge](https://img.shields.io/badge/Version-Alpha-%23e81919?style=flat&color=%23e81919)
![Static Badge](https://img.shields.io/badge/Development_Stage-InDev-%234be819?style=flat)
![Static Badge](https://img.shields.io/badge/Latest_Update-¯%5C__%28ツ%29__/¯-%2318a5a3?)
![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads-pre/CaptainMirage/Rusty-Analyser/latest/total?style=flat&label=Total%20Downloads&color=%2322c2a0)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## Overview

Rusty Analyser is a Rust-based tool that performs a comprehensive analysis of your fixed drives (aka built in drives).
It helps you get detailed insights into storage usage patterns and file distributions using totally efficient I/O parallel processing.
currently its kinda slow, taking a few minutes to scan a full drive with over around 2 million files even with limits,
but that depends on the drive speed since it uses I/O scanning.

## Features
- **Full drive analysis:** it just scans it all and does it all
- **Bash system:** the whole project is in a bash like system, custom-made

## Coming Features
- a more talkative terminal
- faster scans with NTFS scanning 
(likes of [everything](https://www.voidtools.com/), [wiztree](https://diskanalyzer.com/), etc.)
- smart commands
- auto complete commands

## Commands

Below is a list of available commands along with their usage and a brief description:

**Help**  
`help [command]`
-------------  
Displays descriptions for all commands. If you specify a command, it shows details only for that command.

**Exit**  
`exit [code]`
-------------  
Exits the application. Optionally accepts an exit code.

**Echo**  
`echo [message]`
-------------  
Repeats the provided message back to you.

**Type**  
`type [command]`
-------------  
Checks whether a given command exists.

**pwd**  
`pwd`
-------------  
Displays the current working directory.

**Drive Space**  
`drive-space [drive]`
-------------  
Shows the drive’s total, used, and free space.

**File Type Distribution**  
`file-type-dist [drive]`
-------------  
Displays the distribution of the top 10 file types by space usage.

**Largest Files**  
`largest-files [drive]`
-------------  
Lists the top 10 largest files on the specified drive.

**Largest Folder**  
`largest-folder [drive]`
-------------  
Shows the top 10 largest folders (up to 3 levels deep), excluding hidden folders.

**Recent Large Files**  
`recent-large-files [drive]`
-------------  
Lists large files that were modified within the last 30 days.

**Old Large Files**  
`old-large-files [drive]`
-------------  
Lists large files that are older than 6 months.

**Full Drive Analysis**  
`full-drive-analysis [drive]`
-------------  
Performs a comprehensive analysis of the entire drive.

**Empty Folders**  
`empty-folders [drive] [-delete]`
-------------  
Searches for empty folders on the specified drive. The `-delete` flag is reserved for future deletion functionality.

## How To Use

### Download & Run

1. **Download the release:**  
   Grab the latest zip from the [Releases](https://github.com/CaptainMirage/Rusty-Analyser/releases) page.
2. **Unzip and Run:**  
   Simply unzip the package and run the included `.exe` file.

### Build from Source

1. **Clone the repository:**

   ```bash
   git clone https://github.com/CaptainMirage/Rusty-Analyser.git
   ```
2. **Install Rust:**  
   Follow the instructions on [rustup.rs](https://rustup.rs/) (Windows).
3. **Build and Run:**

   ```bash
   cargo run --release
   ```

## Technologies Used

- just check the [cargo.toml](https://github.com/CaptainMirage/Rusty-Analyser/blob/master/Cargo.toml) file :T

## License

This project is licensed under the [MIT License](LICENSE).

## Attribution
While the MIT License doesn't require it, if you use this tool or its code, a credit would be appreciated! You can provide attribution in any of these ways:

Example attribution:
```markdown
This project uses/was inspired by [Rusty Analyser](https://github.com/CaptainMirage/Rusty-Analyser) by Captain Mirage.
```

## Contact
For inquiries or contributions, feel free to reach out!

(my info is in my profile, cant be bothered to add it here)