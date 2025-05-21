use std::path::PathBuf;

/// Configuration for file synchronization
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Source directory
    pub source: PathBuf,
    
    /// Destination directory
    pub destination: PathBuf,
    
    /// Whether to delete files in destination that don't exist in source
    pub delete: bool,
    
    /// Whether to perform a dry run (no actual changes)
    pub dry_run: bool,
    
    /// Number of parallel jobs
    pub jobs: usize,
    
    /// Patterns of files to ignore
    pub ignore_patterns: Vec<String>,
}