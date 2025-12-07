#!/bin/bash
# Wrapper script to populate test data
# Can be run from project root or src-tauri directory

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TAURI_DIR="$PROJECT_ROOT/src-tauri"

# Change to tauri directory
cd "$TAURI_DIR" || exit 1

# Run the binary
if [ "$1" == "--build" ]; then
    echo "Building populate_test_data..."
    cargo build --bin populate_test_data
    if [ $? -eq 0 ]; then
        echo "Build successful. Run with: ./target/debug/populate_test_data"
    fi
else
    echo "Running populate_test_data..."
    cargo run --bin populate_test_data
fi

