📂 Rust File Synchronizer

A fast and efficient file synchronization tool written in Rust. This utility helps you keep directories in sync by copying files from a source to a destination with various options like dry-run, ignore patterns, and parallel processing.
🚀 Features

For a video demo: https://www.youtube.com/watch?v=RUxZoChOa8c    🦀

⚡ Fast Synchronization - Quickly copy files from source to destination
🧵 Parallel Processing - Utilize multiple CPU cores for faster transfers
🧠 Smart Detection - Only copy files that are different or new
🗑️ Deletion Support - Optionally remove files from destination that no longer exist in source
🙈 Ignore Patterns - Skip files matching specified patterns (like .gitignore)
🔍 Dry Run Mode - Preview changes without modifying files
📊 Detailed Progress - See transfer speeds, file sizes, and estimated completion times

📋 Table of Contents

Installation
Usage
Command Options
Ignore Patterns
Examples
Future Enhancements
License

📥 Installation
Prerequisites

Rust and Cargo (latest stable version)

Building from Source

Clone this repository:

bashgit clone https://github.com/Myrmecology/File-Synchronizer.git
cd rust-file-sync

Build the project:

bashcargo build --release

The executable will be available at target/release/file_sync

🔧 Usage
Basic Synchronization
To copy files from a source directory to a destination:
bashcargo run -- sync --source ./docs --destination ./backup
Preview Changes (Dry Run)
To see what files would be copied without actually making changes:
bashcargo run -- sync --source ./docs --destination ./backup --dry-run
Delete Files Not in Source
To remove files from the destination that don't exist in the source:
bashcargo run -- sync --source ./docs --destination ./backup --delete
Ignore Specific Files
To skip certain files or patterns:
bashcargo run -- sync --source ./docs --destination ./backup --ignore "*.log" --ignore "temp/*"
Control Parallel Processing
To set the number of parallel jobs:
bashcargo run -- sync --source ./docs --destination ./backup --jobs 4
Combine Multiple Options
You can combine options as needed:
bashcargo run -- sync --source ./docs --destination ./backup --delete --dry-run --ignore "*.tmp" --jobs 8
🎛️ Command Options
OptionShortDescription--source-sSource directory to copy files from--destination-dDestination directory to copy files to--delete-DDelete files from destination that don't exist in source--dry-run-nSimulate the operation without making changes--ignore-iPattern of files to ignore (can be used multiple times)--jobs-jNumber of parallel jobs (default: number of CPU cores)
🙈 Ignore Patterns
The File Synchronizer supports glob patterns for ignoring files:

*.log - Ignore all files with .log extension
temp/* - Ignore all files in the temp directory
**/*.bak - Ignore .bak files in any directory
node_modules/** - Ignore the entire node_modules directory and contents
data/cache_*.dat - Ignore specific filename patterns

🧪 Examples
Let's walk through some examples to see how the synchronizer works:
Example 1: Basic Synchronization
bash# Create test directories and files
mkdir -p test_source/subfolder
echo "Test content" > test_source/file1.txt
echo "More content" > test_source/subfolder/file2.txt

# Perform synchronization
cargo run -- sync --source ./test_source --destination ./test_output
Example 2: Using Ignore Patterns
bash# Create some files to ignore
echo "Log entry" > test_source/debug.log
echo "Temporary data" > test_source/temp.dat

# Sync with ignore patterns
cargo run -- sync --source ./test_source --destination ./test_output --ignore "*.log" --ignore "*.dat"
Example 3: Dry Run with Deletion
bash# Remove a file from source
rm test_source/file1.txt

# Preview sync with deletion
cargo run -- sync --source ./test_source --destination ./test_output --delete --dry-run
Example 4: Actual Sync with Deletion
bash# Perform sync with deletion
cargo run -- sync --source ./test_source --destination ./test_output --delete
🔮 Future Enhancements
Potential future features:

Remote synchronization (SSH/FTP)
File checksum verification
Backup mode
Bandwidth limiting
Scheduled synchronization

📄 License
This project is open-source software licensed under the MIT license.

Happy coding everyone 