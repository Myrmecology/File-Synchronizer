use crate::config::SyncConfig;
use crate::file_utils::{copy_file, delete_path, files_are_equal, get_relative_path};
use globset::{Glob, GlobSet, GlobSetBuilder};
use indicatif::{ProgressBar, ProgressStyle, HumanBytes, HumanDuration};
use log::{debug, info};
use rayon::prelude::*;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use walkdir::WalkDir;

/// Synchronize files from source to destination
pub fn synchronize(config: &SyncConfig) -> io::Result<()> {
    info!(
        "Synchronizing from {:?} to {:?}",
        config.source, config.destination
    );
    
    if config.dry_run {
        info!("Dry run mode: no files will be modified");
    }

    // Build glob set for ignore patterns
    let mut builder = GlobSetBuilder::new();
    for pattern in &config.ignore_patterns {
        match Glob::new(pattern) {
            Ok(glob) => {
                builder.add(glob);
                info!("Added ignore pattern: {}", pattern);
            },
            Err(e) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid ignore pattern '{}': {}", pattern, e),
                ));
            }
        }
    }
    let glob_set = match builder.build() {
        Ok(set) => set,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to build glob set: {}", e),
            ));
        }
    };

    // Configure thread pool
    rayon::ThreadPoolBuilder::new()
        .num_threads(config.jobs)
        .build_global()
        .unwrap();

    // Scan source and destination directories
    let source_files = scan_directory(&config.source, &glob_set)?;
    let destination_files = scan_directory(&config.destination, &GlobSet::empty())?;

    if source_files.is_empty() {
        info!("No files to synchronize after applying ignore patterns");
        return Ok(());
    } else {
        info!("Found {} files to synchronize", source_files.len());
    }

    // Calculate total size of all files
    let total_bytes = source_files.iter().fold(0, |acc, path| {
        acc + std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    });

    let processed_bytes = Arc::new(AtomicU64::new(0));

    // Create progress bar with enhanced style
    let progress_bar = ProgressBar::new(total_bytes);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("##-"),
    );

    // Start time for calculating transfer speed
    let start_time = Instant::now();

    // Process files in parallel
    source_files
        .par_iter()
        .try_for_each(|source_path| -> io::Result<()> {
            let rel_path = get_relative_path(source_path, &config.source);
            let dest_path = config.destination.join(&rel_path);

            // Check if file needs to be copied
            let should_copy = if dest_path.exists() {
                !files_are_equal(source_path, &dest_path)?
            } else {
                true
            };

            let file_size = std::fs::metadata(source_path).map(|m| m.len()).unwrap_or(0);

            if should_copy {
                copy_file(source_path, &dest_path, config.dry_run)?;
            } else {
                debug!("Skipping unchanged file: {:?}", rel_path);
            }

            // Update progress
            processed_bytes.fetch_add(file_size, Ordering::Relaxed);
            progress_bar.set_position(processed_bytes.load(Ordering::Relaxed));
            progress_bar.set_message(format!("File: {}", rel_path.display()));

            Ok(())
        })?;

    // Calculate overall transfer speed
    let elapsed = start_time.elapsed();
    let bytes_processed = processed_bytes.load(Ordering::Relaxed);
    let speed = if elapsed.as_secs() > 0 {
        bytes_processed / elapsed.as_secs()
    } else {
        bytes_processed
    };

    progress_bar.finish_with_message(format!(
        "Synchronized {} files ({}) in {}. Avg speed: {}/s",
        source_files.len(),
        HumanBytes(bytes_processed),
        HumanDuration(elapsed),
        HumanBytes(speed)
    ));

    // Delete files in destination that don't exist in source
    if config.delete {
        info!("Checking for files to delete in destination");

        let source_rel_paths: HashSet<_> = source_files
            .iter()
            .map(|p| get_relative_path(p, &config.source))
            .collect();

        let files_to_delete: Vec<_> = destination_files
            .iter()
            .filter(|dest_path| {
                let rel_path = get_relative_path(dest_path, &config.destination);
                !source_rel_paths.contains(&rel_path)
            })
            .collect();

        if files_to_delete.is_empty() {
            info!("No files to delete");
        } else {
            // Calculate total size of files to delete
            let total_delete_bytes = files_to_delete.iter().fold(0, |acc, path| {
                acc + std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
            });

            let delete_bytes = Arc::new(AtomicU64::new(0));

            let delete_progress = ProgressBar::new(total_delete_bytes);
            delete_progress.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.red} [{elapsed_precise}] [{bar:40.red/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta}) {msg}")
                    .expect("Invalid progress bar template")
                    .progress_chars("##-"),
            );

            let delete_start_time = Instant::now();

            files_to_delete.par_iter().try_for_each(|path| {
                let file_size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
                let rel_path = get_relative_path(path, &config.destination);
                
                let result = delete_path(path, config.dry_run);
                
                // Update progress
                delete_bytes.fetch_add(file_size, Ordering::Relaxed);
                delete_progress.set_position(delete_bytes.load(Ordering::Relaxed));
                delete_progress.set_message(format!("Deleting: {}", rel_path.display()));
                
                result
            })?;

            let delete_elapsed = delete_start_time.elapsed();
            let delete_bytes_processed = delete_bytes.load(Ordering::Relaxed);
            let delete_speed = if delete_elapsed.as_secs() > 0 {
                delete_bytes_processed / delete_elapsed.as_secs()
            } else {
                delete_bytes_processed
            };

            delete_progress.finish_with_message(format!(
                "Deleted {} files ({}) in {}. Avg speed: {}/s",
                files_to_delete.len(),
                HumanBytes(delete_bytes_processed),
                HumanDuration(delete_elapsed),
                HumanBytes(delete_speed)
            ));
        }
    }

    Ok(())
}

/// Scan a directory and return a list of file paths, applying ignore patterns
fn scan_directory(dir: &Path, glob_set: &GlobSet) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    
    let walker = WalkDir::new(dir).follow_links(true).into_iter();
    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        
        // Skip directories
        if path.is_dir() {
            continue;
        }
        
        // Check if path should be ignored
        let rel_path = get_relative_path(&path, dir);
        let rel_path_str = rel_path.to_string_lossy();
        
        if glob_set.is_match(rel_path_str.to_string()) {
            debug!("Ignoring file: {:?}", rel_path);
            continue;
        }
        
        files.push(path);
    }
    
    Ok(files)
}