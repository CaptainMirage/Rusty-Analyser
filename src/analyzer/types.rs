use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DriveAnalysis {
    pub total_size: f64,
    pub used_space: f64,
    pub free_space: f64,
    pub free_space_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FolderSize {
    pub folder: String,
    pub size_gb: f64,
    pub file_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FileInfo {
    pub full_path: String,
    pub size_mb: f64,
    pub last_modified: Option<String>,
    pub last_accessed: Option<String>,
}

#[derive(Debug, Default)]
pub struct FileTypeStats {
    pub total_size: u64,
    pub count: usize,
}