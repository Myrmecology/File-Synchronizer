Rust File Synchronizer
A fast and efficient file synchronization tool written in Rust. This utility helps you keep directories in sync by copying files from a source to a destination with various options like dry-run, ignore patterns, and parallel processing.
Features

✅ Fast Synchronization - Quickly copy files from source to destination
✅ Parallel Processing - Utilize multiple CPU cores for faster transfers
✅ Smart Detection - Only copy files that are different or new
✅ Deletion Support - Optionally remove files from destination that no longer exist in source
✅ Ignore Patterns - Skip files matching specified patterns (like .gitignore)
✅ Dry Run Mode - Preview changes without modifying files
✅ Detailed Progress - See transfer speeds, file sizes, and estimated completion times

Installation
Prerequisites

Rust and Cargo (latest stable version)

Building from Source

Clone this repository or download the source code
Open a terminal or command prompt and navigate to the project directory
Build the project with:
cargo build --release

The executable will be available at target/release/file_sync

Usage
Basic Synchronization
To copy files from a source directory to a destination:
cargo run -- sync --source SOURCE_DIR --destination DEST_DIR
Replace SOURCE_DIR and DEST_DIR with your actual directory paths.
Preview Changes (Dry Run)
To see what files would be copied without actually making changes:
cargo run -- sync --source SOURCE_DIR --destination DEST_DIR --dry-run
Delete Files Not in Source
To remove files from the destination that don't exist in the source:
cargo run -- sync --source SOURCE_DIR --destination DEST_DIR --delete
Ignore Specific Files
To skip certain files or patterns:
cargo run -- sync --source SOURCE_DIR --destination DEST_DIR --ignore "*.log" --ignore "temp/*"
You can specify multiple --ignore patterns.
Control Parallel Processing
To set the number of parallel jobs:
cargo run -- sync --source SOURCE_DIR --destination DEST_DIR --jobs 4
Combine Multiple Options
You can combine options as needed:
cargo run -- sync --source SOURCE_DIR --destination DEST_DIR --delete --dry-run --ignore "*.tmp" --jobs 8
Ignore Pattern Examples
The File Synchronizer supports glob patterns for ignoring files:

*.log - Ignore all files with .log extension
temp/* - Ignore all files in the temp directory
**/*.bak - Ignore .bak files in any directory
node_modules/** - Ignore the entire node_modules directory and contents
data/cache_*.dat - Ignore specific filename patterns

Testing It Out
Let's walk through a simple test to see how the synchronizer works:

Create test directories and files:
mkdir -p test_source/subfolder
echo "Test content" > test_source/file1.txt
echo "More content" > test_source/subfolder/file2.txt
echo "Log entry" > test_source/debug.log

Run a dry run first to preview:
cargo run -- sync --source ./test_source --destination ./test_output --dry-run

Perform the actual synchronization:
cargo run -- sync --source ./test_source --destination ./test_output

Verify the files were copied by checking the destination:
dir test_output

Try with ignore patterns:
cargo run -- sync --source ./test_source --destination ./test_output --ignore "*.log"

Test the delete option by removing a file from source:
del test_source/file1.txt
cargo run -- sync --source ./test_source --destination ./test_output --delete


Command Options Explained
OptionShortDescription--source-sSource directory to copy files from--destination-dDestination directory to copy files to--delete-DDelete files from destination that don't exist in source--dry-run-nSimulate the operation without making changes--ignore-iPattern of files to ignore (can be used multiple times)--jobs-jNumber of parallel jobs (default: number of CPU cores)
Future Enhancements
Potential future features:

Remote synchronization (SSH/FTP)
File checksum verification
Backup mode
Bandwidth limiting
Scheduled synchronization

License
This project uses the MIT license 

Happy coding everyone 