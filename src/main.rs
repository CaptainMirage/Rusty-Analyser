use std::path::Path;
#[allow(unused_imports)]
use std::fs; // for unix based shit
use std::io;
use rayon::prelude::*;
use serde::Serialize;
#[allow(unused_imports)]
use chrono::{Duration, Utc, DateTime}; // for unix based shit pt.2
use walkdir::WalkDir;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::{System, Disk};

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
}

struct StorageAnalyzer {
    drives: Vec<String>,
}

impl StorageAnalyzer {
    fn new() -> Self {
        // We display all disks' information:
        println!("=> disks:");
        let mut system = System::new_all();
        system.refresh_all(); // Adjusted for API change

        let drives: Vec<String> = system
            .disks()
            .iter()
            .map(|disk| disk.mount_point().to_string_lossy().into_owned())
            .collect();
        StorageAnalyzer { drives }
    }

    fn analyze_drive(&self, drive: &str) -> io::Result<()> {
        println!("\n====== STORAGE DISTRIBUTION ANALYSIS ======");
        println!("Date: {}", Utc::now().format("%Y-%m-%d %H:%M:%S"));
        println!("Drive: {}", drive);
        println!("========================================\n");

        let drive_analysis = self.get_drive_space(drive)?;
        println!("Drive Space Overview:");
        println!("Total Size (GB): {:.2}", drive_analysis.total_size);
        println!("Used Space (GB): {:.2}", drive_analysis.used_space);
        println!("Free Space (GB): {:.2}", drive_analysis.free_space);
        println!("Free Space (%): {:.2}", drive_analysis.free_space_percent);

        println!("\nLargest Folders:");
        let largest_folders = self.get_largest_folders(drive)?;
        for folder in largest_folders.iter().take(10) {
            println!("{:?}", folder);
        }

        println!("\nFile Type Distribution:");
        let file_type_distribution = self.get_file_type_distribution(drive)?;
        for (ext, size, count) in file_type_distribution.iter().take(10) {
            println!("Extension: {}, Count: {}, Size (GB): {:.2}", ext, count, size);
        }

        println!("\nLargest Individual Files:");
        let largest_files = self.get_largest_files(drive)?;
        for file in largest_files.iter().take(10) {
            println!("{:?}", file);
        }

        Ok(())
    }

    fn get_drive_space(&self, drive: &str) -> io::Result<DriveAnalysis> {
        let mut system = System::new_all();
        system.refresh_all(); // Adjusted for API change

        if let Some(disk) = system.disks().iter().find(|&disk| disk.mount_point().to_str() == Some(drive)) {
            let total_size = disk.total_space() as f64 / 1_073_741_824.0;
            let free_size = disk.available_space() as f64 / 1_073_741_824.0;
            let used_size = total_size - free_size;

            Ok(DriveAnalysis {
                total_size,
                used_space: used_size,
                free_space: free_size,
                free_space_percent: (free_size / total_size) * 100.0,
            })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "Drive not found"))
        }
    }

    fn get_largest_folders(&self, drive: &str) -> io::Result<Vec<FolderSize>> {
        let mut folders = Vec::new();

        for entry in WalkDir::new(drive)
            .max_depth(3)  // Limit depth to prevent extremely long processing
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_dir())
        {
            match self.calculate_folder_size(entry.path()) {
                Ok(folder_size) => folders.push(folder_size),
                Err(_) => continue, // Skip folders with access issues
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
            let ext = entry.path()
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
            .filter(|&(_, size, _)| size > 0.01) // Filter out tiny file types
            .collect();

        distribution.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        Ok(distribution)
    }

    fn get_largest_files(&self, drive: &str) -> io::Result<Vec<FileInfo>> {
        let mut files: Vec<_> = WalkDir::new(drive)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|entry| {
                let metadata = entry.metadata().ok()?;
                let file_size = metadata.len();
                let last_modified = metadata.modified().ok()?;

                Some(FileInfo {
                    full_path: entry.path().to_string_lossy().to_string(),
                    size_mb: file_size as f64 / 1_048_576.0,
                    last_modified: Self::system_time_to_string(last_modified),
                })
            })
            .collect();

        files.sort_by(|a, b| b.size_mb.partial_cmp(&a.size_mb).unwrap());
        Ok(files)
    }

    fn system_time_to_string(system_time: SystemTime) -> String {
        let datetime: DateTime<Utc> = match system_time.duration_since(UNIX_EPOCH) {
            Ok(duration) => DateTime::from_timestamp(duration.as_secs() as i64, 0)
                .unwrap_or_else(|| Utc::now()),
            Err(_) => Utc::now(),
        };
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn run(&self) -> io::Result<()> {
        for drive in &self.drives {
            self.analyze_drive(drive)?;
        }
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let analyzer = StorageAnalyzer::new();
    analyzer.run()
}
