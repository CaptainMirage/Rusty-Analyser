use chrono::{DateTime, Duration, TimeZone, Utc};
use ctrlc;
use rayon::prelude::*;
use serde::Serialize;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::{self};
use std::os::windows::ffi::OsStrExt;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;
use winapi::um::fileapi::GetDiskFreeSpaceExW;
use winapi::um::fileapi::{GetDriveTypeW, GetLogicalDriveStringsW};
use winapi::um::winbase::DRIVE_FIXED;

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

pub struct StorageAnalyzer {
    drives: Vec<String>,
}

impl StorageAnalyzer {
    pub fn new() -> Self {
        let drives = StorageAnalyzer::list_drives();
        StorageAnalyzer { drives }
    }

    fn list_drives() -> Vec<String> {
        #[cfg(target_os = "windows")]
        {
            let mut buffer = [0u16; 256];
            let len = unsafe { GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr()) };

            if len == 0 {
                return Vec::new();
            }

            buffer[..len as usize]
                .split(|&c| c == 0)
                .filter_map(|slice| {
                    if slice.is_empty() {
                        None
                    } else {
                        let drive = OsString::from_wide(slice).to_string_lossy().into_owned();
                        let drive_type = unsafe { GetDriveTypeW(slice.as_ptr()) };
                        if drive_type == DRIVE_FIXED {
                            Some(drive)
                        } else {
                            None
                        }
                    }
                })
                .collect()
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Unix-based: Return a placeholder empty vector for now
            // (still not sure how to do it)
            Vec::new()
        }
    }

    fn analyze_drive(&self, drive: &str) -> io::Result<()> {
        println!("\n====== STORAGE DISTRIBUTION ANALYSIS ======");
        println!("Date: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        println!("Drive: {}", drive);
        println!("========================================\n");

        match self.get_drive_space(drive) {
            Ok(drive_analysis) => {
                println!("Drive Space Overview:");
                println!("Total Size (GB): {:.2}", drive_analysis.total_size);
                println!("Used Space (GB): {:.2}", drive_analysis.used_space);
                println!("Free Space (GB): {:.2}", drive_analysis.free_space);
                println!("Free Space (%): {:.2}", drive_analysis.free_space_percent);
            }
            Err(e) => {
                eprintln!("Failed to analyze drive '{}': {}", drive, e);
                return Ok(());
            }
        }

        println!("\nLargest Folders (Top 10):");
        let largest_folders = self.get_largest_folders(drive)?;
        for folder in largest_folders.iter().take(10) {
            println!("Folder: {}", folder.folder);
            println!("Size (GB): {:.2}", folder.size_gb);
            println!("File Count: {}", folder.file_count);
            println!("---");
        }

        println!("\nFile Type Distribution (Top 10):");
        let file_type_distribution = self.get_file_type_distribution(drive)?;
        for (ext, size, count) in file_type_distribution.iter().take(10) {
            println!(
                "Extension: {}, Count: {}, Size (GB): {:.2}",
                ext, count, size
            );
        }

        println!("\nLargest Individual Files (Top 10):");
        let largest_files = self.get_largest_files(drive)?;
        for file in largest_files.iter().take(10) {
            println!("Path: {}", file.full_path);
            println!("Size (MB): {:.2}", file.size_mb);
            println!("Last Modified: {}", file.last_modified);
            println!("---");
        }

        println!("\nRecently Modified Large Files (>100MB, Last 30 days):");
        let recent_large_files = self.get_recent_large_files(drive)?;
        for file in recent_large_files {
            println!("Path: {}", file.full_path);
            println!("Size (MB): {:.2}", file.size_mb);
            println!("Last Modified: {}", file.last_modified);
            println!("---");
        }

        println!("\nOld Large Files (>100MB, Not accessed in 6 months):");
        let old_large_files = self.get_old_large_files(drive)?;
        for file in old_large_files {
            println!("Path: {}", file.full_path);
            println!("Size (MB): {:.2}", file.size_mb);
            println!("Last Modified: {}", file.last_modified);
            if let Some(last_accessed) = file.last_accessed {
                println!("Last Accessed: {}", last_accessed);
            }
            println!("---");
        }

        Ok(())
    }

    fn get_drive_space(&self, drive: &str) -> io::Result<DriveAnalysis> {
        use winapi::um::winnt::ULARGE_INTEGER;
        let mut free_bytes_available: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };
        let mut total_free_bytes: ULARGE_INTEGER = unsafe { std::mem::zeroed() };

        // Convert drive to a wide string
        let wide_drive: Vec<u16> = OsStr::new(drive)
            .encode_wide()
            .chain(Some(0)) // Null-terminate the string
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
            return Err(io::Error::last_os_error());
        }

        // Convert ULARGE_INTEGER fields to f64
        let total_size = unsafe { *total_bytes.QuadPart() } as f64 / 1_073_741_824.0;
        let free_space = unsafe { *total_free_bytes.QuadPart() } as f64 / 1_073_741_824.0;
        let used_space = total_size - free_space;

        Ok(DriveAnalysis {
            total_size,
            used_space,
            free_space,
            free_space_percent: (free_space / total_size) * 100.0,
        })
    }

    fn get_largest_folders(&self, drive: &str) -> io::Result<Vec<FolderSize>> {
        let mut folders = Vec::new();
        let walker = WalkDir::new(drive)
            .min_depth(1)
            .max_depth(3)
            .into_iter()
            .filter_entry(|e| {
                !e.file_name()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            });

        for entry in walker
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            match self.calculate_folder_size(entry.path()) {
                Ok(folder_size) if folder_size.size_gb > 0.1 => folders.push(folder_size),
                _ => continue,
            }
        }

        folders.sort_by(|a, b| b.size_gb.partial_cmp(&a.size_gb).unwrap());
        Ok(folders)
    }

    fn calculate_folder_size(&self, path: &Path) -> io::Result<FolderSize> {
        let files: Vec<_> = WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        let total_size: u64 = files
            .par_iter()
            .map(|entry| entry.metadata().map(|m| m.len()).unwrap_or(0))
            .sum();

        Ok(FolderSize {
            folder: path.to_string_lossy().to_string(),
            size_gb: total_size as f64 / 1_073_741_824.0,
            file_count: files.len(),
        })
    }

    fn get_file_type_distribution(&self, drive: &str) -> io::Result<Vec<(String, f64, usize)>> {
        let mut file_types = std::collections::HashMap::new();

        for entry in WalkDir::new(drive)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let ext = entry
                .path()
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_else(|| "(No Extension)".to_string());

            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            file_types
                .entry(ext)
                .and_modify(|(total_size, count)| {
                    *total_size += size;
                    *count += 1;
                })
                .or_insert((size, 1));
        }

        let mut distribution: Vec<_> = file_types
            .into_iter()
            .map(|(ext, (size, count))| (ext, size as f64 / 1_073_741_824.0, count))
            .filter(|&(_, size, _)| size > 0.01)
            .collect();

        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(distribution)
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
            if let Ok(modified) = DateTime::parse_from_str(&file.last_modified, "%Y-%m-%d %H:%M:%S")
            {
                modified.with_timezone(&Utc) < six_months_ago
            } else {
                false
            }
        });

        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
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
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            if let Ok(metadata) = entry.metadata() {
                let file_size = metadata.len() as f64 / 1_048_576.0;

                if let Some(min_size) = min_size_mb {
                    if file_size < min_size {
                        continue;
                    }
                }

                let last_modified = metadata.modified().ok().map(Self::system_time_to_string);
                let last_accessed = metadata.accessed().ok().map(Self::system_time_to_string);

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
        analyzer.analyze_drive(&drive)?;
    }

    Ok(())
}
