mod config;
mod file_utils;
mod sync;

use clap::{Parser, Subcommand};
use config::SyncConfig;
use log::{info, LevelFilter};
use std::path::PathBuf;
use sync::synchronize;

/// A file synchronization tool
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Synchronize files between source and destination
    Sync {
        /// Source directory
        #[arg(short, long)]
        source: PathBuf,

        /// Destination directory
        #[arg(short, long)]
        destination: PathBuf,

        /// Delete files in destination that don't exist in source
        #[arg(short = 'D', long, default_value_t = false)]
        delete: bool,

        /// Only simulate, don't actually copy/delete files
        #[arg(short = 'n', long, default_value_t = false)]
        dry_run: bool,

        /// Number of parallel jobs (default: number of CPU cores)
        #[arg(short, long)]
        jobs: Option<usize>,
        
        /// Patterns of files to ignore (can be specified multiple times)
        #[arg(short, long)]
        ignore: Vec<String>,
    },
}

fn main() {
    // Initialize logger
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .format_timestamp_secs()
        .init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Sync {
            source,
            destination,
            delete,
            dry_run,
            jobs,
            ignore,
        } => {
            info!("Starting synchronization");
            
            if !source.exists() {
                eprintln!("Error: Source directory does not exist: {:?}", source);
                std::process::exit(1);
            }

            if !destination.exists() {
                println!("Creating destination directory: {:?}", destination);
                if let Err(e) = std::fs::create_dir_all(destination) {
                    eprintln!("Error creating destination directory: {}", e);
                    std::process::exit(1);
                }
            }

            let config = SyncConfig {
                source: source.clone(),
                destination: destination.clone(),
                delete: *delete,
                dry_run: *dry_run,
                jobs: jobs.unwrap_or_else(|| num_cpus::get()),
                ignore_patterns: ignore.clone(),
            };

            match synchronize(&config) {
                Ok(_) => info!("Synchronization completed successfully"),
                Err(e) => {
                    eprintln!("Error during synchronization: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
