# Rusty Analyser

A Rust program that performs comprehensive drive analysis, providing detailed insights into storage usage patterns and file distributions.

![Static Badge](https://img.shields.io/badge/Version-Alpha-%23e81919?style=flat&color=%23e81919)
![Static Badge](https://img.shields.io/badge/Development_Stage-InDev-%234be819?style=flat)
![Static Badge](https://img.shields.io/badge/Latest_Update-¯%5C__%28ツ%29__/¯-%2318a5a3)

## Description

This tool analyzes Windows fixed drives to provide detailed storage insights and help identify space usage patterns. It uses parallel processing for efficient analysis of large directory structures.

## Features

- Drive space analysis (total, used, free space with percentages)
- Largest folders identification (up to 3 levels deep)
- File type distribution analysis with size thresholds
- Largest files listing with metadata
- Recent large files analysis (last 30 days)
- Old large files identification (older than 6 months)
- Parallel processing for improved performance
- Non-blocking operation with graceful interruption handling

## How To Use

1. Clone the repository
```bash
git clone https://github.com/CaptainMirage/Rusty-System-Analyser
```
2. Install Rust via [rustup](https://rustup.rs/) (Windows)
3. Build and run:
```bash
cargo run --release
```

The program automatically analyzes all fixed drives, excluding USB and network drives.

## Technologies

- **Rust**: Core programming language
- **rayon**: Parallel computation framework
- **walkdir**: Directory traversal
- **chrono**: Time and date handling
- **serde**: Data serialization
- **winapi**: Windows API integration
- **ctrlc**: Signal handling

## Thresholds

- Minimum folder size: 0.1 GB
- Minimum file type size: 0.01 GB

## License

yes

## Author Info

For inquiries or contributions, feel free to reach out!
