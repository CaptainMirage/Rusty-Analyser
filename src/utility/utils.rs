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
use walkdir::WalkDir;
use num_cpus;

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

    println!("No cache found, scanning {}...", drive);

    // Skip patterns - common folders with many small files that slow scans
    let skip_patterns = [
        "Windows\\WinSxS",
        "Windows\\Installer",
        "Program Files\\WindowsApps",
        "$Recycle.Bin",
        "System Volume Information",
    ];

    // Create batched processing system
    const BATCH_SIZE: usize = 1000;
    let files_batches = Arc::new(Mutex::new(Vec::new()));
    let folder_sizes = Arc::new(Mutex::new(HashMap::<String, (u64, usize)>::new())); // (size, file_count)

    // Create a thread pool for processing batches
    let _pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();

    // Single-pass file system walk
    let entries: Vec<_> = WalkDir::new(drive)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
        .par_bridge()
        .filter_map(Result::ok)
        .filter(|entry| {
            let path_str = entry.path().to_string_lossy();
            !skip_patterns.iter().any(|pattern| path_str.contains(pattern))
        })
        .collect();

    // Process entries in parallel batches
    entries.par_chunks(BATCH_SIZE).for_each(|batch| {
        let mut local_files = Vec::with_capacity(batch.len());
        let mut local_folders = HashMap::new();

        for entry in batch {
            let path = entry.path();

            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    let file_size = metadata.len();
                    // Add file to local files collection
                    local_files.push(FileInfo {
                        full_path: path.to_string_lossy().to_string(),
                        size_mb: file_size as f64 / MB_TO_BYTES,
                        last_modified: metadata.modified().ok().map(system_time_to_string),
                        last_accessed: metadata.accessed().ok().map(system_time_to_string),
                    });

                    // Update folder sizes for all parent folders
                    let mut current = path;
                    while let Some(parent) = current.parent() {
                        if parent.as_os_str().is_empty() {
                            break;
                        }
                        let parent_path = parent.to_string_lossy().to_string();
                        let entry = local_folders.entry(parent_path).or_insert((0, 0));
                        entry.0 += file_size;
                        entry.1 += 1;
                        current = parent;
                    }
                }
            }
        }

        // Merge local results into global collections
        let mut files_lock = files_batches.lock().unwrap();
        files_lock.push(local_files);

        let mut folder_lock = folder_sizes.lock().unwrap();
        for (folder, (size, count)) in local_folders {
            let entry = folder_lock.entry(folder).or_insert((0, 0));
            entry.0 += size;
            entry.1 += count;
        }
    });

    // Combine all file batches
    let mut all_files = Vec::new();
    for batch in Arc::try_unwrap(files_batches).unwrap().into_inner().unwrap() {
        all_files.extend(batch);
    }

    // Convert folder size map to the expected format
    let folder_data = Arc::try_unwrap(folder_sizes)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .filter(|(path, _)| path.starts_with(drive))
        .map(|(folder, (size, count))| {
            FolderSize {
                folder,
                size_gb: size as f64 / GB_TO_BYTES,
                file_count: count,
            }
        })
        .collect();

    // Store results in cache
    file_cache.insert(drive.to_string(), all_files);
    folder_cache.insert(drive.to_string(), folder_data);

    println!("Scanning complete, cached files and folders");
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