use crate::DATE_FORMAT;
use crate::analyser::types::*;
use crate::utility::constants::*;
use chrono::{DateTime, TimeZone, Utc};
use rayon::prelude::*;
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
use std::error::Error;
use walkdir::WalkDir;

// helper function to convert system time to formatted string
pub fn system_time_to_string(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time
        .duration_since(UNIX_EPOCH)
        .map(|duration| Utc.timestamp_opt(duration.as_secs() as i64, 0).unwrap())
        .unwrap_or_else(|_| Utc::now());
    datetime.format(DATE_FORMAT).to_string()
}

use std::time::Instant;

pub fn time_command<F, R>(command: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = command();
    let elapsed = start.elapsed();
    println!("Execution time: {:?}", elapsed);
    result
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


/// A thin wrapper around `validate_and_format_drive` that:
/// 1. lets you write actions returning `Result<(), Box<dyn Error>>`  
/// 2. still runs through all of the drive‑format/validation logic  
/// 3. prints the same `eprintln!`s on bad format or I/O
pub fn validate_drive<F>(
    drive: &str,
    action: F,
) -> Result<(), Box<dyn Error>>
where
// now accepts your explorer‑style closures that return Box<dyn Error>
    F: FnOnce(&str) -> Result<(), Box<dyn Error>>,
{
    // stash whatever your action returns
    let mut result: Option<Result<(), Box<dyn Error>>> = None;

    // run the core validator/formatter
    validate_and_format_drive(drive, |formatted_drive| {
        match action(formatted_drive) {
            Ok(()) => {
                // action succeeded
                result = Some(Ok(()));
                Ok(()) // tell the validator "no I/O error"
            }
            Err(e) => {
                // action failed — stash the real Box<dyn Error>
                result = Some(Err(e));
                // but return an io::Error so `validate_and_format_drive` will
                // hit its `if let Err(e) = action(...) { eprintln!(...) }` branch
                Err(io::Error::new(io::ErrorKind::Other, "action failed"))
            }
        }
    });
    
    // todo - make the None thing actually do smth and not a lazy shit
    // now look at what actually happened
    match result {
        Some(Ok(()))  => Ok(()),      // drive OK + action OK
        Some(Err(e))  => Err(e),      // drive OK + action Err(e)
        None          => Ok(()),  // validator never even called us
    }
}