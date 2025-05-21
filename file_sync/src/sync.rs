use crate::config::SyncConfig;
use crate::file_utils::{copy_file, delete_path, files_are_equal, get_relative_path};
use globset::{Glob, GlobSet, GlobSetBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, info};
use rayon::prelude::*;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
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
    } else {
        info!("Found {} files to synchronize", source_files.len());
    }

    // Create progress bar
    let progress_bar = ProgressBar::new(source_files.len() as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
            .expect("Invalid progress bar template")
            .progress_chars("##-"),
    );

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

            if should_copy {
                copy_file(source_path, &dest_path, config.dry_run)?;
            } else {
                debug!("Skipping unchanged file: {:?}", rel_path);
            }

            progress_bar.inc(1);
            Ok(())
        })?;

    progress_bar.finish_with_message("File synchronization completed");

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
            let delete_progress = ProgressBar::new(files_to_delete.len() as u64);
            delete_progress.set_style(
                ProgressStyle::default_bar()
                    .template("[{elapsed_precise}] {bar:40.red/blue} {pos}/{len} {msg}")
                    .expect("Invalid progress bar template")
                    .progress_chars("##-"),
            );

            files_to_delete.par_iter().try_for_each(|path| {
                let result = delete_path(path, config.dry_run);
                delete_progress.inc(1);
                result
            })?;

            delete_progress.finish_with_message("Deletion completed");
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
        
        // Fix: Convert to a String first, then use that as it implements AsRef<Path>
        if glob_set.is_match(rel_path_str.to_string()) {
            debug!("Ignoring file: {:?}", rel_path);
            continue;
        }
        
        files.push(path);
    }
    
    Ok(files)
}