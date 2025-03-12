use super::{constants::*, types::*, utils::*};
use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};
use rayon::{ThreadPoolBuilder, prelude::*};
#[cfg(target_os = "windows")]
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    io::{self, Error},
    path::Path,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;
#[cfg(target_os = "windows")]
use winapi::um::{
    fileapi::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW},
    winbase::DRIVE_FIXED,
};

pub struct StorageAnalyser {
    pub drives: Vec<String>,
    file_cache: HashMap<String, Vec<FileInfo>>,
    folder_cache: HashMap<String, Vec<FolderSize>>,
}

impl StorageAnalyser {
    pub fn new() -> Self {
        let drives = Self::list_drives();
        StorageAnalyser {
            drives,
            file_cache: HashMap::new(),
            folder_cache: HashMap::new(),
        }
    }

    fn print_file_info(file: &FileInfo) {
        println!("\n[*] Path: {}", file.full_path);
        println!(
            "    Size: {:.2} MB / {:.2} GB",
            file.size_mb,
            file.size_mb / 1000.0
        );
        println!(
            "    Last Modified: {}",
            file.last_modified.as_deref().unwrap_or("Unknown")
        );
        if let Some(last_accessed) = &file.last_accessed {
            println!("    Last Accessed: {}", last_accessed);
        }
    }

    // Windows-specific implementation to list fixed drives
    // filters for physical drives only, skips USB/network drives
    #[cfg(target_os = "windows")]
    fn list_drives() -> Vec<String> {
        let mut buffer = [0u16; 256];
        let len = unsafe { GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr()) };

        if len == 0 {
            return Vec::new();
        }

        buffer[..len as usize]
            .split(|&c| c == 0)
            .filter_map(|slice| {
                (!slice.is_empty())
                    .then(|| {
                        let drive = OsString::from_wide(slice);
                        let drive_type = unsafe { GetDriveTypeW(slice.as_ptr()) };
                        (drive_type == DRIVE_FIXED).then(|| drive.to_string_lossy().into_owned())
                    })
                    .flatten()
            })
            .collect()
    }

    // placeholder for non-Windows platforms, no bloody idea what to do
    #[cfg(not(target_os = "windows"))]
    fn list_drives() -> Vec<String> {
        Vec::new()
    }

    // a full scan fn that calls all other ones
    pub fn analyze_drive(&mut self, drive: &str) -> io::Result<()> {
        if !self.drives.contains(&drive.to_string()) {
            println!(
                "Drive {} is not a valid fixed drive. Valid drives are: {:?}",
                drive, self.drives
            );
            return Ok(());
        }

        println!("\n=== Storage Distribution Analysis ===");
        println!("Date: {}", Utc::now().format(DATE_FORMAT));
        println!("Drive: {}", drive);

        self.print_drive_space_overview(drive)?;
        self.print_largest_folders(drive)?;
        self.print_empty_folders(drive)?;
        self.print_file_type_distribution(drive)?;
        self.print_largest_files(drive)?;
        self.print_recent_large_files(drive)?;
        self.print_old_large_files(drive)?;

        Ok(())
    }

    // -- private calculation functions -- //

    // uses Windows API to get drive space information
    fn get_drive_space(&self, drive: &str) -> io::Result<DriveAnalysis> {
        use winapi::um::winnt::ULARGE_INTEGER;
        let mut free_bytes_available: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_free_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };

        // convert drive path to wide string for Windows API
        let wide_drive: Vec<u16> = OsStr::new(drive).encode_wide().chain(Some(0)).collect();

        let success = unsafe {
            GetDiskFreeSpaceExW(
                wide_drive.as_ptr(),
                &mut free_bytes_available as *mut _ as *mut _,
                &mut total_bytes as *mut _ as *mut _,
                &mut total_free_bytes as *mut _ as *mut _,
            )
        };

        if success == 0 {
            return Err(Error::last_os_error());
        }

        let total_size = unsafe { *total_bytes.QuadPart() } as f64 / GB_TO_BYTES;
        let free_space = unsafe { *total_free_bytes.QuadPart() } as f64 / GB_TO_BYTES;
        let used_space = total_size - free_space;

        Ok(DriveAnalysis {
            total_size,
            used_space,
            free_space,
            free_space_percent: (free_space / total_size) * 100.0,
        })
    }

    fn get_file_type_distribution(&mut self, drive: &str) -> io::Result<Vec<(String, f64, usize)>> {
        collect_and_cache_files(drive, &mut self.file_cache, &mut self.folder_cache)?;

        let file_types: HashMap<String, FileTypeStats> =
            if let Some(files) = self.file_cache.get(drive) {
                files
                    .par_iter()
                    .fold(
                        || HashMap::new(),
                        |mut acc, file_info| {
                            let ext = Path::new(&file_info.full_path)
                                .extension()
                                .map(|e| e.to_string_lossy().to_lowercase())
                                .unwrap_or_else(|| "(No Extension)".to_string());

                            let size = (file_info.size_mb * MB_TO_BYTES) as u64;

                            let stats: &mut FileTypeStats = acc.entry(ext).or_default();
                            stats.total_size += size;
                            stats.count += 1;
                            acc
                        },
                    )
                    .reduce(
                        || HashMap::new(),
                        |mut acc1, acc2| {
                            for (ext, stats2) in acc2 {
                                let stats1 = acc1.entry(ext).or_default();
                                stats1.total_size += stats2.total_size;
                                stats1.count += stats2.count;
                            }
                            acc1
                        },
                    )
            } else {
                HashMap::new()
            };

        let mut distribution: Vec<_> = file_types
            .into_iter()
            .map(|(ext, stats)| (ext, stats.total_size as f64 / GB_TO_BYTES, stats.count))
            .filter(|&(_, size, _)| size > MIN_FILE_TYPE_SIZE_GB)
            .collect();

        distribution.par_sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(distribution)
    }

    fn get_largest_files(&mut self, drive: &str) -> io::Result<Vec<FileInfo>> {
        collect_and_cache_files(drive, &mut self.file_cache, &mut self.folder_cache)?;

        if let Some(files) = self.file_cache.get(drive) {
            let mut result = files.clone();
            result.par_sort_unstable_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    fn get_largest_folders(&self, drive: &str) -> io::Result<Vec<FolderSize>> {
        if let Some(cached_folders) = self.folder_cache.get(drive) {
            // Use the cached folder sizes, filtering out folders that are too small.
            let mut folders: Vec<FolderSize> = cached_folders
                .iter()
                .cloned()
                .filter(|folder| folder.size_gb > MIN_FOLDER_SIZE_GB)
                .collect();
            // Sort descending by size.
            folders.sort_unstable_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap());
            return Ok(folders);
        }
        // Fallback in the unlikely event the cache is missing.
        let mut folders = WalkDir::new(drive)
            .min_depth(1)
            .max_depth(3)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
            .filter(|e| {
                !e.file_name()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            })
            .filter_map(|entry| {
                calculate_folder_size(entry.path())
                    .ok()
                    .filter(|size| size.size_gb > MIN_FOLDER_SIZE_GB)
            })
            .collect::<Vec<_>>();
        folders.par_sort_unstable_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap());
        Ok(folders)
    }

    // gets anything older than 6 months
    fn get_old_large_files(&mut self, drive: &str) -> io::Result<Vec<FileInfo>> {
        collect_and_cache_files(drive, &mut self.file_cache, &mut self.folder_cache)?;

        let mut files = if let Some(files) = self.file_cache.get(drive) {
            files.clone()
        } else {
            return Ok(Vec::new());
        };

        let six_months_ago = Utc::now().naive_utc() - Duration::days(180);

        files.retain(|file| {
            NaiveDateTime::parse_from_str(
                &file.last_modified.as_deref().unwrap_or("Unknown"),
                DATE_FORMAT,
            )
            .map(|dt| dt < six_months_ago)
            .unwrap_or(false)
        });

        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
    }

    // gets recently modified large files (within last 30 days)
    fn get_recent_large_files(&mut self, drive: &str) -> io::Result<Vec<FileInfo>> {
        collect_and_cache_files(drive, &mut self.file_cache, &mut self.folder_cache)?;

        let mut files = if let Some(files) = self.file_cache.get(drive) {
            files.clone()
        } else {
            return Ok(Vec::new());
        };
        let thirty_days_ago = Utc::now().naive_utc() - Duration::days(30);
        files.retain(|file| {
            NaiveDateTime::parse_from_str(
                &file.last_modified.as_deref().unwrap_or("Unknown"),
                DATE_FORMAT,
            )
            .map(|dt| dt > thirty_days_ago)
            .unwrap_or(false)
        });
        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
    }

    pub fn get_empty_folders(&mut self, drive: &str) -> io::Result<Vec<String>> {
        collect_and_cache_files(drive, &mut self.file_cache, &mut self.folder_cache)?;

        // filtering for folders with 0 files.
        if let Some(cached_folders) = self.folder_cache.get(drive) {
            let empty_folders: Vec<String> = cached_folders
                .iter()
                .filter(|folder| folder.file_count == 0)
                .map(|folder| folder.folder.clone())
                .collect();
            Ok(empty_folders)
        } else {
            Ok(vec![])
        }
    }

    // -- public printing functions -- //

    pub fn print_drive_space_overview(&self, drive: &str) -> io::Result<()> {
        match self.get_drive_space(drive) {
            Ok(analysis) => {
                println!("\n--- Drive Space Overview ---");
                println!("Total Size: {:.2} GB", analysis.total_size);
                println!("Used Space: {:.2} GB", analysis.used_space);
                println!(
                    "Free Space: {:.2} GB ({:.2}%)",
                    analysis.free_space, analysis.free_space_percent
                );
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to analyze drive '{}': {}", drive, e);
                Err(e)
            }
        }
    }

    pub fn print_file_type_distribution(&mut self, drive: &str) -> io::Result<()> {
        println!("\n--- File Type Distribution (Top 10) ---");
        let distribution = self.get_file_type_distribution(drive)?;
        for (ext, size, count) in distribution.iter().take(10) {
            println!(
                "\n[>] {} \n  Count: {} \n  Size: {:.2} GB",
                ext, count, size
            );
        }
        Ok(())
    }

    pub fn print_largest_files(&mut self, drive: &str) -> io::Result<()> {
        println!("\n--- Largest Files ---");
        let files = self.get_largest_files(drive)?;
        for file in files.iter().take(10) {
            Self::print_file_info(file)
        }
        Ok(())
    }

    // returns largest folders up to 3 levels deep
    // excludes hidden folders (those starting with '.')
    pub fn print_largest_folders(&mut self, drive: &str) -> io::Result<()> {
        println!("\n--- Largest Folders (Top 10) ---");

        if !self.folder_cache.contains_key(drive) {
            collect_and_cache_files(drive, &mut self.file_cache, &mut self.folder_cache)?;
        }

        let folders = self.get_largest_folders(drive)?;

        let mut cnt: i8 = 0;
        for folder in folders.iter().take(10) {
            cnt += 1;
            println!("\n[{}] {}", cnt, folder.folder);
            println!("  Size: {:.2} GB", folder.size_gb);
            println!("  Files: {}", folder.file_count);
        }

        Ok(())
    }

    pub fn print_old_large_files(&mut self, drive: &str) -> io::Result<()> {
        println!("\n--- Old Large Files (>6 months old) ---");
        let files = self.get_old_large_files(drive)?;
        for file in files.iter().take(10) {
            Self::print_file_info(file)
        }
        Ok(())
    }

    pub fn print_recent_large_files(&mut self, drive: &str) -> io::Result<()> {
        println!("\n--- Recent Large Files ---");
        let files = self.get_recent_large_files(drive)?;
        for file in files.iter().take(10) {
            Self::print_file_info(file)
        }
        Ok(())
    }

    pub fn print_empty_folders(&mut self, drive: &str) -> io::Result<()> {
        println!("\n--- Empty Folders ---");
        let empty_folders = self.get_empty_folders(drive)?;
        println!("Found {} empty folders.", empty_folders.len());
        for folder in empty_folders.iter() {
            println!(" - {}", folder);
        }
        Ok(())
    }
}
