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

#[derive(Debug, Serialize)]
struct FileInfo {
    full_path: String,
    size_mb: f64,
    last_modified: String,
    last_accessed: Option<String>,
}

// New type for file type statistics
#[derive(Debug)]
struct FileTypeStats {
    total_size: u64,
    count: usize,
}

pub struct StorageAnalyzer {
    drives: Vec<String>,
}

// Implementation of drive-related functionality
impl StorageAnalyzer {
    pub fn new() -> Self {
        let drives = Self::list_drives();
        StorageAnalyzer { drives }
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
                if slice.is_empty() {
                    return None;
                }
                let drive = OsString::from_wide(slice).to_string_lossy().into_owned();
                let drive_type = unsafe { GetDriveTypeW(slice.as_ptr()) };
                (drive_type == DRIVE_FIXED).then_some(drive)
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

        let wide_drive: Vec<u16> = OsStr::new(drive)
            .encode_wide()
            .chain(Some(0))
            .collect();

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
    
}
// Implementation of analysis functionality
impl StorageAnalyzer {
    pub fn analyze_drive(&self, drive: &str) -> io::Result<()> {
        println!("\n====== STORAGE DISTRIBUTION ANALYSIS ======");
        println!("Date: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
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

    fn print_file_type_distribution(&self, drive: &str) -> io::Result<()> {
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

    fn print_file_info(&self, files: &[&FileInfo]) {
        for file in files {
            println!("Path: {}", file.full_path);
            println!("Size (MB): {:.2}", file.size_mb);
            println!("Last Modified: {}", file.last_modified);
            if let Some(last_accessed) = &file.last_accessed {
                println!("Last Accessed: {}", last_accessed);
            }
            println!("---");
        }
    }
}

// Implementation of file collection and processing
impl StorageAnalyzer {
    fn get_largest_folders(&self, drive: &str) -> io::Result<Vec<FolderSize>> {
        let mut folders = WalkDir::new(drive)
            .min_depth(1)
            .max_depth(3)
            .into_iter()
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

        folders.sort_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap());
        Ok(folders)
    }

    fn calculate_folder_size(&self, path: &Path) -> io::Result<FolderSize> {
        let files: Vec<_> = WalkDir::new(path)
            .into_iter()
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

    fn get_file_type_distribution(&self, drive: &str) -> io::Result<Vec<(String, f64, usize)>> {
        let mut file_types: HashMap<String, FileTypeStats> = HashMap::new();

        WalkDir::new(drive)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
            .for_each(|entry| {
                let ext = entry
                    .path()
                    .extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .unwrap_or_else(|| "(No Extension)".to_string());

                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

                file_types
                    .entry(ext)
                    .and_modify(|stats| {
                        stats.total_size += size;
                        stats.count += 1;
                    })
                    .or_insert(FileTypeStats {
                        total_size: size,
                        count: 1,
                    });
            });

        let mut distribution: Vec<_> = file_types
            .into_iter()
            .map(|(ext, stats)| {
                (
                    ext,
                    stats.total_size as f64 / GB_TO_BYTES,
                    stats.count,
                )
            })
            .filter(|&(_, size, _)| size > MIN_FILE_TYPE_SIZE_GB)
            .collect();

        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(distribution)
    }

    fn collect_files(
        &self,
        drive: &str,
        after_date: Option<DateTime<Utc>>,
        min_size_mb: Option<f64>,
    ) -> io::Result<Vec<FileInfo>> {
        let mut files = Vec::new();

        for entry in WalkDir::new(drive)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len() as f64 / MB_TO_BYTES;

                if let Some(min_size) = min_size_mb {
                    if file_size < min_size {
                        continue;
                    }
                }

                let last_modified = metadata
                    .modified()
                    .ok()
                    .map(Self::system_time_to_string);

                let last_accessed = metadata
                    .accessed()
                    .ok()
                    .map(Self::system_time_to_string);

                if let Some(last_modified_str) = last_modified {
                    if let Some(after) = after_date {
                        if let Ok(modified) =
                            DateTime::parse_from_str(&last_modified_str, "%Y-%m-%d %H:%M:%S")
                        {
                            if modified.with_timezone(&Utc) < after {
                                continue;
                            }
                        }
                    }

                    files.push(FileInfo {
                        full_path: entry.path().to_string_lossy().to_string(),
                        size_mb: file_size,
                        last_modified: last_modified_str,
                        last_accessed,
                    });
                }
            }
        }

        Ok(files)
    }

    fn system_time_to_string(system_time: SystemTime) -> String {
        let datetime: DateTime<Utc> = system_time
            .duration_since(UNIX_EPOCH)
            .map(|duration| Utc.timestamp_opt(duration.as_secs() as i64, 0).unwrap())
            .unwrap_or_else(|_| Utc::now());
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn get_largest_files(&self, drive: &str) -> io::Result<Vec<FileInfo>> {
        let mut files = self.collect_files(drive, None, None)?;
        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
    }

    fn get_recent_large_files(&self, drive: &str) -> io::Result<Vec<FileInfo>> {
        let thirty_days_ago = Utc::now() - Duration::days(30);
        let mut files = self.collect_files(drive, Some(thirty_days_ago), Some(100.0))?;
        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
    }

    fn get_old_large_files(&self, drive: &str) -> io::Result<Vec<FileInfo>> {
        let six_months_ago = Utc::now() - Duration::days(180);
        let mut files = self.collect_files(drive, None, Some(100.0))?;

        files.retain(|file| {
            DateTime::parse_from_str(&file.last_modified, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.with_timezone(&Utc) < six_months_ago)
                .unwrap_or(false)
        });

        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
    }
}

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
        .expect("Error setting Ctrl-C handler");

    let analyzer = StorageAnalyzer::new();
    for drive in &analyzer.drives {
        if !running.load(Ordering::SeqCst) {
            println!("Exiting gracefully...");
            break;
        }
        analyzer.analyze_drive(drive)?;
    }

    Ok(())
}