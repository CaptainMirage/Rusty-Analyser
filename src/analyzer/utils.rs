use chrono::{DateTime, TimeZone, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::DATE_FORMAT;
use std::{collections::HashMap, io, path::Path, sync::{Arc, Mutex}};
use rayon::prelude::*;
use walkdir::WalkDir;
use super::{
    constants::*,
    types::*
};

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

pub fn collect_and_cache_files(
    drive: &str,
    file_cache: &mut HashMap<String, Vec<FileInfo>>,
    folder_cache: &mut HashMap<String, Vec<FolderSize>>
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
    file_cache.insert(drive.to_string(), Arc::try_unwrap(file_cache_arc).unwrap().into_inner().unwrap());
    folder_cache.insert(drive.to_string(), Arc::try_unwrap(folder_cache_arc).unwrap().into_inner().unwrap());
    println!("Caching files and folders..");

    Ok(())
}
