# Rusty System Analyser

A simple Rust program to analyze storage drives, calculate space usage, and provide insights into file and folder distributions.

![Static Badge](https://img.shields.io/badge/Version-Prototype-%23e81919?style=flat&color=%23e81919)
![Static Badge](https://img.shields.io/badge/Development_Stage-InDev-%234be819?style=flat)
![Static Badge](https://img.shields.io/badge/Latest_Update-06%2F01%2F2025-%2318a5a3)

---

## Features

- Drive space metrics (total, used, free)
- Largest folders identification
- File type distribution analysis
- Largest files listing [Coming soon?]
- Recently modified large files >100MB [Coming Soon]
- Old unused large files >100MB [Coming Soon]

---

## How to Use

1. Clone the repository and navigate to the project directory.
    ```bash
     git clone https://github.com/CaptainMirage/Rusty-System-Analyser
2. Ensure Rust is installed. You can install it via [rustup on windows](https://rustup.rs/).

The program will display analysis results for all available drives (probably).

---

## Technologies

- **Rust**: Programming language.
- **sysinfo**: For drive space and system information.
- **walkdir**: For traversing directories.
- **rayon**: For parallelized computations.

---

## License

yes

---

## Author Info

For any inquiries, feel free to reach out or contribute to the project!
