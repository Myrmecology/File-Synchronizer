use log::{debug, info};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Get the relative path from base to path
pub fn get_relative_path(path: &Path, base: &Path) -> PathBuf {
    pathdiff::diff_paths(path, base).unwrap_or_else(|| path.to_path_buf())
}

/// Calculate SHA-256 hash of a file
pub fn calculate_file_hash(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024 * 1024]; // 1MB buffer

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Compare two files to see if they are the same
pub fn files_are_equal(source: &Path, destination: &Path) -> io::Result<bool> {
    // First check if file sizes are different
    let source_meta = fs::metadata(source)?;
    if let Ok(dest_meta) = fs::metadata(destination) {
        if source_meta.len() != dest_meta.len() {
            return Ok(false);
        }

        // If source is newer, consider files different
        if let (Ok(source_time), Ok(dest_time)) = (
            source_meta.modified(),
            dest_meta.modified(),
        ) {
            if source_time > dest_time {
                debug!("Source is newer than destination: {:?}", source);
                return Ok(false);
            }
        }

        // If sizes are the same, compare file hashes
        let source_hash = calculate_file_hash(source)?;
        let dest_hash = calculate_file_hash(destination)?;
        
        return Ok(source_hash == dest_hash);
    }
    
    Ok(false)
}

/// Copy a file with its metadata
pub fn copy_file(source: &Path, destination: &Path, dry_run: bool) -> io::Result<()> {
    if dry_run {
        info!("Would copy: {:?} -> {:?}", source, destination);
        return Ok(());
    }

    // Create parent directories if they don't exist
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    info!("Copying: {:?} -> {:?}", source, destination);
    // Create copy options with overwrite enabled
    let mut options = fs_extra::file::CopyOptions::new();
    options.overwrite = true;
    fs_extra::file::copy(source, destination, &options)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    
    // Copy modification time
    if let Ok(metadata) = fs::metadata(source) {
        if let Ok(mtime) = metadata.modified() {
            let filetime = filetime::FileTime::from_system_time(mtime);
            filetime::set_file_mtime(destination, filetime)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        }
    }

    Ok(())
}

/// Delete a file or directory
pub fn delete_path(path: &Path, dry_run: bool) -> io::Result<()> {
    if dry_run {
        info!("Would delete: {:?}", path);
        return Ok(());
    }

    info!("Deleting: {:?}", path);
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    
    Ok(())
}