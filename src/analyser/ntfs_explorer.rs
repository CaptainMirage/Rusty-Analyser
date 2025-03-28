use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::format;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use crate::utility::constants::GB_TO_BYTES;
use std::io::{Read, Seek};
use std::path::Path;
use ntfs_reader::file_info::FileInfo;
use ntfs_reader::mft::Mft;
use ntfs_reader::volume::Volume;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetDiskFreeSpaceExW(
        lp_directory_name: *const u16,
        lp_free_bytes_available: *mut u64,
        lp_total_number_of_bytes: *mut u64,
        lp_total_number_of_free_bytes: *mut u64,
    ) -> i32;
}

/// Returns true if the file name appears to be a concatenation of GUIDs.
fn is_guid_concat(name: &str) -> bool {
    // Heuristic: if the name starts with '{', contains "}{", and ends with '}'
    // it likely is two GUIDs concatenated.
    name.starts_with('{') && name.contains("}{") && name.ends_with('}')
}

/// Given a file name, returns a user-friendly name (filtering out GUID concatenations).
fn filter_filename(name: &str) -> &str {
    if name.is_empty() {
        "No Name"
    } else if is_guid_concat(name) {
        "Unknown"
    } else {
        name
    }
}

fn format_size(bytes: u64) -> String {
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

/// Retrieves total, used, and free space (in bytes) for the given drive letter.
///     
/// Returns a tuple: (total_bytes, used_bytes, free_bytes).
fn get_drive_space(drive: &str) -> Result<(u64, u64, u64), Box<dyn Error>> {
    // Construct a device path like "C:\".
    let path = format!("{}:\\", drive);
    // Convert to a null-terminated wide string.
    let wide: Vec<u16> = OsStr::new(&path)
        .encode_wide()
        .chain(Some(0))
        .collect();

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

/// Scans the NTFS drive and returns a HashMap with file extensions and their total sizes.
fn scan_file_type_dist(drive_letter: &str) -> HashMap<String, u64> {
    let drive_path = format!("\\\\.\\{}:", drive_letter);
    let volume = Volume::new(&drive_path).expect(&format!("Failed to open volume at {}", drive_path));
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

fn scan_largest_files(drive_letter: &str) -> Vec<FileInfo> {
    let drive_path = format!("\\\\.\\{}:", drive_letter);
    let volume = Volume::new(&drive_path)
        .expect(&format!("Failed to open volume at {}", drive_path));
    let mft = Mft::new(volume)
        .expect("Failed to create MFT from the volume");

    let mut files: Vec<FileInfo> = Vec::new();
    mft.iterate_files(|file| {
        let mut info = FileInfo::new(&mft, file);
        if !info.is_directory {
            // Convert from clusters to bytes if needed:
            files.push(info);
        }
    });
    files.sort_by(|a, b| b.size.cmp(&a.size));
    files
}


/// Prints the top 10 largest files on the specified drive.
/// This function takes a drive letter (e.g. "C"), scans the drive for files,
/// sorts them by size, and prints the file name (filtered) and size (formatted).
pub fn print_largest_files(drive_letter: &str) {
    let files = scan_largest_files(drive_letter);

    println!("Largest Files on Drive {} (Top 10):", drive_letter);
    for file in files.into_iter().take(10) {
        // Filter the file name if it's a GUID concatenation.
        let display_name = filter_filename(&file.name);
        println!("{:<30} {}", display_name, format_size(file.size));
    }
}

// -- public printing functions -- //

pub fn print_drive_space(drive: &str) -> Result<(), Box<dyn Error>> {
    let (total, used, free) = get_drive_space(drive)?;

    println!("Drive {}:", drive);
    println!("Total space: {} GB", format_size(total));
    println!("Used space : {} GB", format_size(used));
    println!("Free space : {} GB", format_size(free));
    Ok(())
}

pub fn print_file_type_dist(drive_letter: &str) -> Result<(), Box<dyn Error>> {
    let distribution = scan_file_type_dist(drive_letter);
    let mut items: Vec<(&String, &u64)> = distribution.iter().collect();
    items.sort_by(|a, b| b.1.cmp(a.1));

    println!("File Type Distribution for Drive {} (Top 10 by space usage):", drive_letter);
    for (ext, size) in items.into_iter().take(10) {
        let display_ext = if ext.is_empty() { "No Extension" } else { ext };
        println!("{:<15}: {}", display_ext, format_size(*size));
    }
    Ok(())
}