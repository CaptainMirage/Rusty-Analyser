use chrono::{DateTime, Duration, TimeZone, Utc};
use rayon::prelude::*;
use serde::Serialize;
use std::{
    collections::HashMap,
    ffi::{OsStr, OsString},
    io::{self, Error},
    os::windows::ffi::{OsStrExt, OsStringExt},
    path::{Path},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;
use winapi::um::{
    fileapi::{GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW},
    winbase::DRIVE_FIXED,
};

// Constants
const GB_TO_BYTES: f64 = 1_073_741_824.0;
const MB_TO_BYTES: f64 = 1_048_576.0;
const MIN_FOLDER_SIZE_GB: f64 = 0.1;
const MIN_FILE_TYPE_SIZE_GB: f64 = 0.01;
const DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Serialize)]
struct DriveAnalysis {
    total_size: f64,
    used_space: f64,
    free_space: f64,
    free_space_percent: f64,
}

#[derive(Debug, Serialize)]
struct FolderSize {
    folder: String,
    size_gb: f64,
    file_count: usize,
}

#[derive(Debug, Serialize, Clone)]
struct FileInfo {
    full_path: String,
    size_mb: f64,
    last_modified: String,
    last_accessed: Option<String>,
}

#[derive(Debug, Default)]
struct FileTypeStats {
    total_size: u64,
    count: usize,
}

pub struct StorageAnalyzer {
    drives: Vec<String>,
    file_cache: HashMap<String, Vec<FileInfo>>,
}

impl StorageAnalyzer {
    pub fn new() -> Self {
        let drives = Self::list_drives();
        StorageAnalyzer {
            drives,
            file_cache: HashMap::new(),
        }
    }

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
                (!slice.is_empty()).then(|| {
                    let drive = OsString::from_wide(slice);
                    let drive_type = unsafe { GetDriveTypeW(slice.as_ptr()) };
                    (drive_type == DRIVE_FIXED).then(|| drive.to_string_lossy().into_owned())
                }).flatten()
            })
            .collect()
    }

    #[cfg(not(target_os = "windows"))]
    fn list_drives() -> Vec<String> {
        Vec::new()
    }

    fn get_drive_space(&self, drive: &str) -> io::Result<DriveAnalysis> {
        use winapi::um::winnt::ULARGE_INTEGER;
        let mut free_bytes_available: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_free_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };

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

    fn collect_and_cache_files(&mut self, drive: &str) -> io::Result<()> {
        if self.file_cache.contains_key(drive) {
            return Ok(());
        }

        let files: Vec<FileInfo> = WalkDir::new(drive)
            .into_iter()
            .par_bridge()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .filter_map(|entry| {
                entry.metadata().ok().map(|metadata| {
                    let file_size = metadata.len() as f64 / MB_TO_BYTES;
                    let last_modified = metadata
                        .modified()
                        .ok()
                        .map(Self::system_time_to_string);
                    let last_accessed = metadata
                        .accessed()
                        .ok()
                        .map(Self::system_time_to_string);

                    last_modified.map(|modified| FileInfo {
                        full_path: entry.path().to_string_lossy().to_string(),
                        size_mb: file_size,
                        last_modified: modified,
                        last_accessed,
                    })
                }).flatten()
            })
            .collect();

        self.file_cache.insert(drive.to_string(), files);
        Ok(())
    }

    fn get_file_type_distribution(&mut self, drive: &str) -> io::Result<Vec<(String, f64, usize)>> {
        self.collect_and_cache_files(drive)?;

        let file_types: HashMap<String, FileTypeStats> = if let Some(files) = self.file_cache.get(drive) {
            files.par_iter()
                .fold(
                    || HashMap::new(),
                    |mut acc: HashMap<String, FileTypeStats>, file_info| {
                        let ext = Path::new(&file_info.full_path)
                            .extension()
                            .map(|e| e.to_string_lossy().to_lowercase())
                            .unwrap_or_else(|| "(No Extension)".to_string());

                        let size = (file_info.size_mb * MB_TO_BYTES) as u64;
                        acc.entry(ext.clone())
                            .or_default()
                            .total_size += size;
                        acc.entry(ext.clone())
                            .or_default()
                            .count += 1;
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
        self.collect_and_cache_files(drive)?;

        if let Some(files) = self.file_cache.get(drive) {
            let mut result = files.clone();
            result.par_sort_unstable_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }

    fn system_time_to_string(system_time: SystemTime) -> String {
        system_time
            .duration_since(UNIX_EPOCH)
            .ok()
            .and_then(|duration| Utc.timestamp_opt(duration.as_secs() as i64, 0).single())
            .unwrap_or_else(Utc::now)
            .format(DATE_FORMAT)
            .to_string()
    }

    pub fn analyze_drive(&mut self, drive: &str) -> io::Result<()> {
        println!("\n====== STORAGE DISTRIBUTION ANALYSIS ======");
        println!("Date: {}", Utc::now().format(DATE_FORMAT));
        println!("Drive: {}", drive);
        println!("========================================\n");

        self.print_drive_analysis(drive)?;
        self.print_largest_folders(drive)?;
        self.print_file_type_distribution(drive)?;
        self.print_largest_files(drive)?;
        self.print_recent_large_files(drive)?;
        self.print_old_large_files(drive)?;

        Ok(())
    }

    fn print_drive_analysis(&self, drive: &str) -> io::Result<()> {
        match self.get_drive_space(drive) {
            Ok(analysis) => {
                println!("Drive Space Overview:");
                println!("Total Size (GB): {:.2}", analysis.total_size);
                println!("Used Space (GB): {:.2}", analysis.used_space);
                println!("Free Space (GB): {:.2}", analysis.free_space);
                println!("Free Space (%): {:.2}", analysis.free_space_percent);
                Ok(())
            }
            Err(e) => {
                eprintln!("Failed to analyze drive '{}': {}", drive, e);
                Err(e)
            }
        }
    }

    fn print_largest_folders(&self, drive: &str) -> io::Result<()> {
        println!("\nLargest Folders (Top 10):");
        let largest_folders = self.get_largest_folders(drive)?;
        for folder in largest_folders.iter().take(10) {
            println!("Folder: {}", folder.folder);
            println!("Size (GB): {:.2}", folder.size_gb);
            println!("File Count: {}", folder.file_count);
            println!("---");
        }
        Ok(())
    }

    fn print_file_type_distribution(&mut self, drive: &str) -> io::Result<()> {
        println!("\nFile Type Distribution (Top 10):");
        let distribution = self.get_file_type_distribution(drive)?;
        for (ext, size, count) in distribution.iter().take(10) {
            println!(
                "Extension: {}, Count: {}, Size (GB): {:.2}",
                ext, count, size
            );
        }
        Ok(())
    }

    fn get_largest_folders(&self, drive: &str) -> io::Result<Vec<FolderSize>> {
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
                self.calculate_folder_size(entry.path())
                    .ok()
                    .filter(|size| size.size_gb > MIN_FOLDER_SIZE_GB)
            })
            .collect::<Vec<_>>();

        folders.par_sort_unstable_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap());
        Ok(folders)
    }

    fn calculate_folder_size(&self, path: &Path) -> io::Result<FolderSize> {
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

    fn print_largest_files(&mut self, drive: &str) -> io::Result<()> {
        println!("\nLargest Files (Top 10):");
        let files = self.get_largest_files(drive)?;
        for file in files.iter().take(10) {
            println!("Path: {}", file.full_path);
            println!("Size (MB): {:.2}", file.size_mb);
            println!("Last Modified: {}", file.last_modified);
            if let Some(last_accessed) = &file.last_accessed {
                println!("Last Accessed: {}", last_accessed);
            }
            println!("---");
        }
        Ok(())
    }

    fn print_recent_large_files(&mut self, drive: &str) -> io::Result<()> {
        println!("\nRecent Large Files (>100MB, last 30 days):");
        let thirty_days_ago = Utc::now() - Duration::days(30);

        if let Some(files) = self.file_cache.get(drive) {
            let mut recent_files: Vec<_> = files.iter()
                .filter(|file| {
                    file.size_mb >= 100.0 &&
                        DateTime::parse_from_str(&file.last_modified, DATE_FORMAT)
                            .map(|dt| dt.with_timezone(&Utc) >= thirty_days_ago)
                            .unwrap_or(false)
                })
                .cloned()
                .collect();

            recent_files.par_sort_unstable_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
            
            #[allow(unused_labels)]
            'NewLargeFiles: for file in recent_files.iter().take(10) {
                println!("Path: {}", file.full_path);
                println!("Size (MB): {:.2}", file.size_mb);
                println!("Last Modified: {}", file.last_modified);
                if let Some(last_accessed) = &file.last_accessed {
                    println!("Last Accessed: {}", last_accessed);
                }
                println!("---");
            }
        }
        Ok(())
    }

    fn print_old_large_files(&mut self, drive: &str) -> io::Result<()> {
        println!("\nOld Large Files (>100MB, older than 6 months):");
        let six_months_ago = Utc::now() - Duration::days(180);

        if let Some(files) = self.file_cache.get(drive) {
            let mut old_files: Vec<_> = files.iter()
                .filter(|file| {
                    file.size_mb >= 100.0 &&
                        DateTime::parse_from_str(&file.last_modified, DATE_FORMAT)
                            .map(|dt| dt.with_timezone(&Utc) < six_months_ago)
                            .unwrap_or(false)
                })
                .cloned()
                .collect();

            old_files.par_sort_unstable_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
            
            #[allow(unused_labels)]
            'OldLargeFiles: for file in old_files.iter().take(10) {
                println!("Path: {}", file.full_path);
                println!("Size (MB): {:.2}", file.size_mb);
                println!("Last Modified: {}", file.last_modified);
                if let Some(last_accessed) = &file.last_accessed {
                    println!("Last Accessed: {}", last_accessed);
                }
                
                println!("---");
            }
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
        .expect("Error setting Ctrl-C handler");

    let mut analyzer = StorageAnalyzer::new();
    let drives: Vec<String> = analyzer.drives.clone();  // Clone the drives vector first

    for drive in drives {
        if !running.load(Ordering::SeqCst) {
            println!("Exiting gracefully...");
            break;
        }
        analyzer.analyze_drive(&drive)?;
    }

    Ok(())
}