#![allow(dead_code)]
use crate::utility::constants::GB_TO_BYTES;
use ntfs_reader::{file_info::FileInfo, mft::Mft, volume::Volume};
use std::{
    collections::HashMap,
    error::Error,
    ffi::OsStr,
    fmt::format,
    io::{Read, Seek},
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr::null_mut,
};
use time::{Duration, OffsetDateTime};

// unsafe shit, use properly or get a panic attack
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

pub struct NtfsExplorer {
}

impl NtfsExplorer {
    pub fn new() -> Self {
        NtfsExplorer {}
    }
    
    /// returns true if the file name appears to be a concatenation of GUIDs.
    fn is_guid_concat(&self, name: &str) -> bool {
        // Heuristic: if the name starts with '{', contains "}{", and ends with '}'
        // it likely is two GUIDs concatenated.
        name.starts_with('{') && name.contains("}{") && name.ends_with('}')
    }
    
    /// given a file name, returns a user-friendly name (filtering out GUID concatenations).
    pub fn filter_filename(&self, name: &str, empty: bool) -> String {
        if name.is_empty() && empty {
            "No Name".to_string()
        } else if self.is_guid_concat(name) {
            "GUID name".to_string()
        } else {
            name.to_string()
        }
    }
    
    /// given a number in bytes, returns a compressed version of it
    ///
    /// examples : `1505210368 --> 1.40 GB` | `815663130 --> 777.88 MB`
    fn format_size(&self, bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
    
        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }
    
    /// Returns a folder key (as a String) from a full file path.
    /// It takes up to 5 directory components from the root.
    /// Example:
    ///
    /// "C:\Folder1\Folder2\Folder3\Folder4\Folder5\Folder6\file.txt" becomes
    /// "C:\Folder1\Folder2\Folder3\Folder4\Folder5".
    fn folder_key_from_path(&self, full_path: &str, drive_letter: &str, depth: usize) -> Option<String> {
        let path = Path::new(full_path);
        // Collect components as Strings.
        let components: Vec<String> = path
            .components()
            .map(|comp| comp.as_os_str().to_string_lossy().into_owned())
            .collect();
    
        if components.is_empty() {
            return None;
        }
    
        // Limit the folder depth.
        let take_count = if components.len() > depth { depth } else { components.len() };
        let joined = components[..take_count].join("\\");
    
        // Optionally remove the drive prefix (if present).
        let drive_prefix = format!("\\\\.\\{}:\\", drive_letter);
        let without_prefix = if joined.starts_with(&drive_prefix) {
            joined.replacen(&drive_prefix, "", 1)
        } else {
            joined
        };
    
        // Split into components and transform each one.
        let mut transformed_components: Vec<String> = without_prefix
            .split('\\')
            .map(|comp| self.filter_filename(comp, false))
            .collect();
    
        // Remove any leading empty component if it exists.
        if let Some(first) = transformed_components.first() {
            if first.is_empty() {
                transformed_components.remove(0);
            }
        }
    
        // Join the components back together.
        let result = transformed_components.join("\\");
        // Prepend the drive letter with a colon
        Some(format!("{}:\\{}", drive_letter, result))
    }
    
    /// Checks if a folder is hidden.
    ///
    /// hidden being with dots or folders that have the hidden tag (totally)
    #[allow(dead_code)]
    fn is_hidden_folder(&self, folder: &str) -> bool {
        // Extract the final component and check if it starts with '.'
        if let Some(name) = Path::new(folder).file_name().and_then(|s| s.to_str()) {
            name.starts_with('.')
        } else {
            false
        }
    }
    
    
    // -- scanning functions -- //
    
    /// retrieves total, used, and free space (in bytes) for the given drive letter.
    ///     
    /// returns a tuple: (total_bytes, used_bytes, free_bytes).
    fn get_drive_space(&self, drive: &str) -> Result<(u64, u64, u64), Box<dyn Error>> {
        // Construct a device path like "C:\".
        let path = format!("{}:\\", drive);
        // Convert to a null-terminated wide string.
        let wide: Vec<u16> = OsStr::new(&path).encode_wide().chain(Some(0)).collect();
    
        let mut free_available = 0u64;
        let mut total_bytes = 0u64;
        let mut free_bytes = 0u64;
    
        let ret = unsafe {
            GetDiskFreeSpaceExW(
                wide.as_ptr(),
                &mut free_available,
                &mut total_bytes,
                &mut free_bytes,
            )
        };
    
        if ret == 0 {
            Err("GetDiskFreeSpaceExW failed".into())
        } else {
            let used_bytes = total_bytes.saturating_sub(free_bytes);
            Ok((total_bytes, used_bytes, free_bytes))
        }
    }
    
    /// scans the NTFS drive and returns a HashMap with file extensions and their total sizes.
    fn scan_file_type_dist(&self, drive_letter: &str) -> HashMap<String, u64> {
        let drive_path = format!("\\\\.\\{}:", drive_letter);
        let volume =
            Volume::new(&drive_path).expect(&format!("Failed to open volume at {}", drive_path));
        let mft = Mft::new(volume).expect("Failed to create MFT from the volume");
    
        let mut distribution: HashMap<String, u64> = HashMap::new();
    
        mft.iterate_files(|file| {
            let info = FileInfo::new(&mft, file);
            if !info.is_directory {
                let extension = Path::new(&info.name)
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_lowercase();
                *distribution.entry(extension).or_insert(0) += info.size;
            }
        });
    
        distribution
    }
    
    fn scan_largest_files(&self, drive_letter: &str) -> Vec<FileInfo> {
        let drive_path = format!("\\\\.\\{}:", drive_letter);
        let volume =
            Volume::new(&drive_path).expect(&format!("Failed to open volume at {}", drive_path));
        let mft = Mft::new(volume).expect("Failed to create MFT from the volume");
    
        let mut files: Vec<FileInfo> = Vec::new();
        mft.iterate_files(|file| {
            #[allow(unused_mut)]
            let mut info = FileInfo::new(&mft, file);
            if !info.is_directory {
                // Convert from clusters to bytes if needed:
                files.push(info);
            }
        });
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files
    }
    
    /// Scans the NTFS drive and returns a HashMap of folder paths (up to 5 levels deep)
    /// and their total file sizes, excluding hidden folders.
    fn scan_largest_folders(&self, drive_letter: &str) -> HashMap<String, u64> {
        let drive_path = format!("\\\\.\\{}:", drive_letter);
        let volume =
            Volume::new(&drive_path).expect(&format!("Failed to open volume at {}", drive_path));
        let mft = Mft::new(volume).expect("Failed to create MFT from the volume");
    
        let mut folder_sizes: HashMap<String, u64> = HashMap::new();
    
        mft.iterate_files(|file| {
            let info = FileInfo::new(&mft, file);
            if !info.is_directory {
                // Convert the file's PathBuf to &str.
                if let Some(path_str) = info.path.to_str() {
                    if let Some(folder) = self.folder_key_from_path(path_str, drive_letter, 5) {
                        // Skip hidden folders if needed, e.g., folders starting with a dot.
                        *folder_sizes.entry(folder).or_insert(0) += info.size;
                    }
                }
            }
        });
    
        folder_sizes
    }
    
    /// Scans the NTFS drive and returns a vector of FileInfo for non‑directory files
    /// that have a modification time (OffsetDateTime) satisfying the filter closure.
    fn scan_files_by_modified<F>(&self, drive_letter: &str, filter: F) -> Vec<FileInfo>
    where
        F: Fn(OffsetDateTime) -> bool,
    {
        let drive_path = format!("\\\\.\\{}:", drive_letter);
        let volume =
            Volume::new(&drive_path).expect(&format!("Failed to open volume at {}", drive_path));
        let mft = Mft::new(volume).expect("Failed to create MFT from the volume");
    
        let mut files: Vec<FileInfo> = Vec::new();
        mft.iterate_files(|file| {
            let info = FileInfo::new(&mft, file);
            if !info.is_directory {
                if let Some(modified) = info.modified {
                    if filter(modified) {
                        files.push(info);
                    }
                }
            }
        });
        files.sort_by(|a, b| b.size.cmp(&a.size));
        files
    }
    
    
    // -- printing functions -- //
    
    /// Displays information about a drive's storage space.
    ///
    /// # Arguments
    ///
    /// * `drive` - The drive letter to analyze (e.g., "C", "D")
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an error if the drive information cannot be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// // Display space information for drive C:
    /// print_drive_space("C").unwrap();
    /// // Output:
    /// // Drive C:
    /// // Total space: 500 GB
    /// // Used space : 350 GB
    /// // Free space : 150 GB
    /// ```
    pub fn print_drive_space(&self, drive: &str) -> Result<(), Box<dyn Error>> {
        let (total, used, free) = self.get_drive_space(drive)?;
    
        println!("Drive {}:", drive);
        println!("Total space: {} GB", self.format_size(total));
        println!("Used space : {} GB", self.format_size(used));
        println!("Free space : {} GB", self.format_size(free));
        Ok(())
    }
    
    /// Displays a distribution of file types on a drive, sorted by total size.
    ///
    /// # Arguments
    ///
    /// * `drive_letter` - The drive letter to analyze (e.g., "C", "D")
    /// * `count` - The number of file types to display in the results
    ///
    /// # Examples
    ///
    /// ```
    /// // Display top 5 file types by size on drive D:
    /// print_file_type_dist("D", 5).unwrap();
    /// // Output:
    /// // File Type Distribution for Drive D: (Top 5 by space usage):
    /// // .mp4            150 GB
    /// // .zip            120 GB
    /// // .pdf            85 GB
    /// // .docx           45 GB
    /// // No Extension    32 GB
    /// ```
    pub fn print_file_type_dist(&self, drive_letter: &str, count: usize) -> Result<(), Box<dyn Error>> {
        let distribution = self.scan_file_type_dist(drive_letter);
        let mut items: Vec<(&String, &u64)> = distribution.iter().collect();
        items.sort_by(|a, b| b.1.cmp(a.1));
    
        println!(
            "File Type Distribution for Drive {} (Top {} by space usage):",
            drive_letter, count
        );
        for (ext, size) in items.into_iter().take(count) {
            let display_ext = if ext.is_empty() { "No Extension" } else { ext };
            println!("{:<15}: {}", display_ext, self.format_size(*size));
        }
        Ok(())
    }
    
    /// Displays the largest files on a drive, sorted by size.
    ///
    /// # Arguments
    ///
    /// * `drive_letter` - The drive letter to analyze (e.g., "C", "D")
    /// * `count` - The number of files to display in the results
    ///
    /// # Examples
    ///
    /// ```
    /// // Display top 3 largest files on drive E:
    /// print_largest_files("E", 3).unwrap();
    /// // Output:
    /// // Largest Files on Drive E (Top 3):
    /// // movie.mkv                       8.5 GB
    /// // backup.iso                      4.2 GB
    /// // dataset.csv                     1.8 GB
    /// ```
    pub fn print_largest_files(&self, drive_letter: &str, count: usize) -> Result<(), Box<dyn Error>> {
        let files = self.scan_largest_files(drive_letter);
    
        println!("Largest Files on Drive {} (Top {}):", drive_letter, count);
        for file in files.into_iter().take(count) {
            // Filter the file name if it's a GUID concatenation.
            let display_name = self.filter_filename(&file.name, true);
            println!("{:<30} {}", display_name, self.format_size(file.size));
        }
        Ok(())
    }
    
    /// Displays the largest folders on a drive, sorted by total size.
    ///
    /// # Arguments
    ///
    /// * `drive_letter` - The drive letter to analyze (e.g., "C", "D")
    /// * `count` - The number of folders to display in the results
    ///
    /// # Examples
    ///
    /// ```
    /// // Display top 5 largest folders on drive C:
    /// print_largest_folders("C", 5).unwrap();
    /// // Output:
    /// // Largest Folders on Drive C: (Top 5):
    /// // C:\Users\username\Videos                              350 GB
    /// // C:\Program Files                                      120 GB
    /// // C:\Users\username\Downloads                           85 GB
    /// // C:\Windows                                            65 GB
    /// // C:\Program Files (x86)                                45 GB
    /// ```
    pub fn print_largest_folders(&self, drive_letter: &str, count: usize) -> Result<(), Box<dyn Error>> {
        let folder_sizes = self.scan_largest_folders(drive_letter);
    
        // Convert to vector and sort descending by size.
        let mut folders: Vec<(&String, &u64)> = folder_sizes.iter().collect();
        folders.sort_by(|a, b| b.1.cmp(a.1));
    
        println!("Largest Folders on Drive {} (Top {}):", drive_letter, count);
        for (folder, size) in folders.into_iter().take(count) {
            println!("{:<50} {}", folder, self.format_size(*size));
        }
        Ok(())
    }
    
    /// Prints the largest files modified within the last 30 days.
    ///
    /// it is gonna be a little too accurate, for now
    ///
    /// # Arguments
    ///
    /// * `drive_letter` - The drive letter to analyze (e.g., "C", "D")
    /// * `count` - The number of files to display in the results
    ///
    /// # Examples
    ///
    /// ```
    /// // Display top 4 recent large files on drive D:
    /// print_recent_large_files("D", 4).unwrap();
    /// // Output:
    /// // Recent Large Files on Drive D: (Modified within last 30 days):
    /// // project_backup.zip             2.5 GB  Modified: 2023-05-10T14:32:15Z
    /// // meeting_recording.mp4          1.8 GB  Modified: 2023-05-15T09:45:30Z
    /// // system_logs.tar                1.2 GB  Modified: 2023-05-18T22:10:05Z
    /// // virtual_machine.vhdx           0.9 GB  Modified: 2023-05-20T16:25:40Z
    /// ```
    pub fn print_recent_large_files(&self, drive_letter: &str, count: usize) -> Result<(), Box<dyn Error>> {
        let now = OffsetDateTime::now_utc();
        let threshold = Duration::days(30);
    
        let files = self.scan_files_by_modified(drive_letter, |mod_time| now - mod_time <= threshold);
    
        println!(
            "Recent Large Files on Drive {} (Modified within last 30 days):",
            drive_letter
        );
        for file in files.into_iter().take(count) {
            println!(
                "{:<30} {}  Modified: {}",
                self.filter_filename(&*file.name, true),
                self.format_size(file.size),
                file.modified.unwrap()
            );
        }
        Ok(())
    }
    
    /// Prints the largest files modified more than 6 months ago.
    ///
    /// it is gonna be a little too accurate, for now
    ///
    /// # Arguments
    ///
    /// * `drive_letter` - The drive letter to analyze (e.g., "C", "D")
    /// * `count` - The number of files to display in the results
    ///
    /// # Examples
    ///
    /// ```
    /// // Display top 3 old large files on drive C:
    /// print_old_large_files("C", 3).unwrap();
    /// // Output:
    /// // Old Large Files on Drive C: (Modified more than 6 months ago):
    /// // old_backup_2022.zip            4.5 GB  Modified: 2022-08-12T18:20:45Z
    /// // archive_data.tar               3.2 GB  Modified: 2022-05-30T11:15:22Z
    /// // legacy_application.iso         2.8 GB  Modified: 2021-11-05T14:40:15Z
    /// ```
    pub fn print_old_large_files(&self, drive_letter: &str, count: usize) -> Result<(), Box<dyn Error>> {
        let now = OffsetDateTime::now_utc();
        let threshold = Duration::days(6 * 30); // approximate 6 months
    
        let files = self.scan_files_by_modified(drive_letter, |mod_time| now - mod_time >= threshold);
    
        println!(
            "Old Large Files on Drive {} (Modified more than 6 months ago):",
            drive_letter
        );
        for file in files.into_iter().take(count) {
            println!(
                "{:<30} {}  Modified: {}",
                self.filter_filename(&*file.name, true),
                self.format_size(file.size),
                file.modified.unwrap()
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scanner() {
        let files = scan_largest_folders("C");
        assert!(!files.is_empty(), "Expected some files from drive C");
    }

    #[test]
    fn test_printer() {
        println!("\n\n");
        print_largest_folders("C", 13).unwrap();
    }

    #[test]
    fn test_all() {
        let drivel = "C";
        println!("\n\n");
        print_drive_space(drivel).unwrap();
        println!("\n\n");
        print_file_type_dist(drivel, 12).unwrap();
        println!("\n\n");
        print_largest_files(drivel, 12).unwrap();
        println!("\n\n");
        print_largest_folders(drivel, 12).unwrap();
        println!("\n\n");
        print_old_large_files(drivel, 12).unwrap();
        println!("\n\n");
        print_recent_large_files(drivel, 12).unwrap();
        println!("\n\n");
    }
}
