use super::{constants::*, types::*};
use crate::DATE_FORMAT;
use chrono::{DateTime, TimeZone, Utc};
use rand::Rng;
use rayon::prelude::*;
use std::io::stdout;
use std::thread::sleep;
use std::time::Duration;
use std::{
    collections::HashMap,
    io,
    io::Write,
    path::Path,
    sync::{Arc, Mutex},
};
use std::{
    fs::{OpenOptions, create_dir_all},
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

// helper function to convert system time to formatted string
pub fn system_time_to_string(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time
        .duration_since(UNIX_EPOCH)
        .map(|duration| Utc.timestamp_opt(duration.as_secs() as i64, 0).unwrap())
        .unwrap_or_else(|_| Utc::now());
    datetime.format(DATE_FORMAT).to_string()
}

pub fn calculate_folder_size(path: &Path) -> io::Result<FolderSize> {
    let files: Vec<_> = WalkDir::new(path)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .collect();

    let total_size: u64 = files
        .par_iter()
        .map(|entry| entry.metadata().map(|m| m.len()).unwrap_or(0))
        .sum();

    Ok(FolderSize {
        folder: path.to_string_lossy().to_string(),
        size_gb: total_size as f64 / GB_TO_BYTES,
        file_count: files.len(),
    })
}

pub fn save_empty_folders_to_file(empty_folders: &[String]) -> io::Result<()> {
    // ensure the outputs folder exists.
    let output_dir = Path::new("outputs");
    if !output_dir.exists() {
        create_dir_all(output_dir)?;
        println!("Created outputs directory.");
    } else {
        println!("Outputs directory already exists.");
    }

    // define the output file path.
    let report_file_path = output_dir.join("EmptyFolderReport.txt");

    // open the file in write mode
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&report_file_path)?;

    // write the report header and the empty folders list.
    writeln!(file, "Empty Folders Report:")?;
    writeln!(file, "Found {} empty folders.", empty_folders.len())?;
    for folder in empty_folders {
        writeln!(file, " - {}", folder)?;
    }

    println!(
        "Saved empty folders report to: {}",
        report_file_path.display()
    );
    Ok(())
}

pub fn collect_and_cache_files(
    drive: &str,
    file_cache: &mut HashMap<String, Vec<FileInfo>>,
    folder_cache: &mut HashMap<String, Vec<FolderSize>>,
) -> io::Result<()> {
    if file_cache.contains_key(drive) || folder_cache.contains_key(drive) {
        println!("Cached scan found! Proceeding..");
        return Ok(());
    }

    println!("No cache found, scanning..");

    let file_cache_arc = Arc::new(Mutex::new(Vec::new()));
    let folder_cache_arc = Arc::new(Mutex::new(Vec::new()));

    let files: Vec<FileInfo> = WalkDir::new(drive)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|entry| {
            let metadata = entry.metadata().ok()?;
            Some(FileInfo {
                full_path: entry.path().to_string_lossy().to_string(),
                size_mb: metadata.len() as f64 / MB_TO_BYTES,
                last_modified: metadata.modified().ok().map(system_time_to_string),
                last_accessed: metadata.accessed().ok().map(system_time_to_string),
            })
        })
        .flatten()
        .collect();

    {
        let mut cache = file_cache_arc.lock().unwrap();
        cache.extend(files);
    }

    let folders: Vec<FolderSize> = WalkDir::new(drive)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
        .filter_map(|entry| calculate_folder_size(entry.path()).ok())
        .collect();

    {
        let mut cache = folder_cache_arc.lock().unwrap();
        cache.extend(folders);
    }

    println!("Scanning complete..");
    file_cache.insert(
        drive.to_string(),
        Arc::try_unwrap(file_cache_arc)
            .unwrap()
            .into_inner()
            .unwrap(),
    );
    folder_cache.insert(
        drive.to_string(),
        Arc::try_unwrap(folder_cache_arc)
            .unwrap()
            .into_inner()
            .unwrap(),
    );
    println!("Caching files and folders..");

    Ok(())
}

pub fn validate_and_format_drive<F>(drive: &str, action: F)
where
    F: FnOnce(&str) -> Result<(), io::Error>,
{
    let drive = drive.to_uppercase();

    if drive.len() == 1 && drive.chars().all(|c| c.is_ascii_alphabetic()) {
        // user entered just the letter (e.g., "C"), format it properly
        if let Err(e) = action(format!("{}:/", drive).as_str()) {
            eprintln!("Error: {}", e);
        }
    } else if drive.len() == 3
        && drive.ends_with(":/")
        && drive.chars().next().unwrap().is_ascii_alphabetic()
    {
        // user entered a valid full path (e.g., "C:/"), use it directly
        if let Err(e) = action(drive.as_str()) {
            eprintln!("Error: {}", e);
        }
    } else {
        // invalid input
        eprintln!(
            "Invalid drive format. Please enter a single letter (e.g., 'C')\
         or a valid drive path (e.g., 'C:/')."
        );
    }
}

pub fn type_text(text: &str, base_speed_ms: u64, end_delay_ms: Option<u64>, natural: bool) {
    let stdout = stdout();
    let mut handle = stdout.lock();
    let mut rng = rand::rng();

    // characters that typically cause a slight natural pause when typing
    let pause_chars = ['.', '!', '?', ',', ';', ':', '-', ')', '}', ']'];

    let mut prev_char = ' ';

    for c in text.chars() {
        // write the current character
        write!(handle, "{}", c).unwrap();
        handle.flush().unwrap();

        // calculate delay for this character
        let mut char_delay = base_speed_ms;

        if natural {
            // add slight randomness to typing speed (using positive range and subtracting after)
            let variation = rng.random_range(0..=30);
            if variation <= 10 {
                // subtract up to 10ms (similar to -10..=xx range which cant be normally)
                char_delay = char_delay.saturating_sub(variation);
            } else {
                // add up to 20ms for the remaining range
                char_delay = char_delay.saturating_add(variation - 10);
            }

            // add natural pauses after certain punctuation
            if pause_chars.contains(&prev_char) {
                char_delay = char_delay.saturating_add(rng.random_range(100..400));
            }

            // simulate faster typing for common character sequences
            if (prev_char == 't' && c == 'h')
                || (prev_char == 'i' && c == 'n')
                || (prev_char == 'a' && c == 'n')
            {
                char_delay = char_delay.saturating_sub(10);
            }
        }

        // sleep for the calculated delay
        sleep(Duration::from_millis(char_delay));

        // remember this character for next iteration
        prev_char = c;
    }

    // add a newline at the end
    writeln!(handle).unwrap();

    // apply the end delay (default to 500ms if None provided)
    let delay = end_delay_ms.unwrap_or(500);
    sleep(Duration::from_millis(delay));
}

// simplified version of type_text with default parameters
pub fn type_text_simple(text: &str, speed_ms: u64) {
    type_text(text, speed_ms, Some(500), true);
}

#[allow(dead_code)]
pub fn tester_function() {
    // Natural typing effect with default end delay
    type_text(
        "Hello, this is a demonstration of the natural typing effect! It mimics how a real person would type.",
        70,
        None,
        true,
    );

    // Using the simplified function for quick usage
    println!("\nUsing the simplified function:");
    type_text_simple("This uses the simplified function with natural typing.", 70);

    // Compare natural vs mechanical typing
    println!("\nNatural typing (with randomness and pauses):");
    type_text(
        "The quick brown fox jumps over the lazy dog. How natural does this feel?",
        60,
        Some(700),
        true,
    );

    println!("\nMechanical typing (constant speed):");
    type_text(
        "The quick brown fox jumps over the lazy dog. Notice the difference?",
        60,
        Some(700),
        false,
    );
}

pub fn display_boot_sequence() {
    type_text("Initializing Analyzer...", 35, Some(400), true);

    // Progress bar animation
    let steps = ["[     ]", "[=    ]", "[==   ]", "[===  ]", "[==== ]", "[=====]"];
    let messages = [
        "booting Iconic Defibulizer",
        "Connecting to Interdimensional Cable",
        "Calibrating Detox Machine",
        "Optimizing algorithms",
        "Preparing Mind Wiper",
        "Starting Time Device"
    ];

    for (step, message) in steps.iter().zip(messages.iter()) {
        // Clear the current line before printing
        print!("\r\x1B[K"); // ANSI escape code to clear the line
        print!("{} {}", step, message);
        stdout().flush().unwrap();
        sleep(Duration::from_millis(600));
    }

    println!("\n");
    sleep(Duration::from_millis(300));

    // Status messages
    let status_prefixes = [
        "Iconic Defibulizer booted    [OK]",
        "IC cable connected           [OK]",
        "Detox Machine calibrated     [OK]",
        "Algorithms optimized         [OK]",
        "Mind Wiper ready             [OK]",
        "Time Device running          [OK]"
    ];

    for prefix in status_prefixes {
        // Print the prefix without a newline
        type_text(prefix, 35, Some(400), true);
        stdout().flush().unwrap();
    }

    type_text("\nAnalyzer ready.", 35, Some(400), true);
}