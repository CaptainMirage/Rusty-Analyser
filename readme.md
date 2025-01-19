# Rusty Analyser

A Rust program that performs comprehensive drive analysis, providing detailed insights into storage usage patterns and file distributions.

![Static Badge](https://img.shields.io/badge/Version-Alpha-%23e81919?style=flat&color=%23e81919)
![Static Badge](https://img.shields.io/badge/Development_Stage-OnHold-%234be819?style=flat)
![Static Badge](https://img.shields.io/badge/Latest_Update-¯%5C__%28ツ%29__/¯-%2318a5a3?)
![GitHub Downloads (all assets, latest release)](https://img.shields.io/github/downloads-pre/CaptainMirage/Rusty-Analyser/latest/total?style=flat&label=Total%20Downloads&color=%2322c2a0)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)


## Description

This tool analyzes Windows fixed drives to provide detailed storage insights and help identify space usage patterns. It uses parallel processing for efficient analysis of large directory structures.

## Features

- Drive space analysis (total, used, free space with percentages)
- Largest folders identification (up to 3 levels deep)
- File type distribution analysis with size thresholds
- Largest files listing with metadata
- Recent large files analysis (last 30 days)
- Old large files identification (older than 6 months)

## How To Use

### Download the built project 
1. download the zip file from [releases](https://github.com/CaptainMirage/Rusty-Analyser/releases) page
2. unzip and run the `.exe` file

### Build from Source
1. Clone the repository
```bash
git clone https://github.com/CaptainMirage/Rusty-Analyser
```
2. Install Rust via [rustup](https://rustup.rs/) (Windows)
3. Build and run:
```bash
cargo run --release
```

## Technologies

- **Rust**: Core programming language
- **rayon**: Parallel computation framework
- **walkdir**: Directory traversal
- **chrono**: Time and date handling
- **serde**: Data serialization
- **winapi**: Windows API integration
- **ctrlc**: Signal handling

## Thresholds

- The program automatically analyzes all fixed drives, excluding USB and network drives.

## License & Attribution

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

### Attribution?
While the MIT License doesn't require it, if you use this tool or its code, a credit would be appreciated! You can provide attribution in any of these ways:

- 🔗 Link back to this repository in your project
- 📝 Mention it in your documentation
- 💡 Share how you used or modified it

Example attribution:
```markdown
This project uses/was inspired by [Rusty Analyser](https://github.com/CaptainMirage/Rusty-Analyser) by Captain Mirage
```

## Author Info

For inquiries or contributions, feel free to reach out!

(my info is in my profile)
