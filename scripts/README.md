# CareerBench Scripts

This directory contains utility scripts for development and testing.

## populate_test_data.rs

Populates the CareerBench database with realistic test data for development and testing.

### Usage

**Option 1: Using the wrapper script (easiest)**

From the project root:

```bash
# Run directly
./scripts/populate_test_data.sh

# Or build first, then run manually
./scripts/populate_test_data.sh --build
./src-tauri/target/debug/populate_test_data
```

**Option 2: From the project root with cargo**

```bash
# Run directly with cargo
cargo run --manifest-path src-tauri/Cargo.toml --bin populate_test_data

# Or build and run
cargo build --manifest-path src-tauri/Cargo.toml --bin populate_test_data
./src-tauri/target/debug/populate_test_data
```

**Option 3: From the src-tauri directory**

```bash
cd src-tauri

# Run directly with cargo
cargo run --bin populate_test_data

# Or build and run
cargo build --bin populate_test_data
./target/debug/populate_test_data
```

### What it does

The script:
1. Clears all existing data from the database
2. Populates the database with test data including:
   - User profile (Alex Johnson - Senior Full-Stack Engineer)
   - 3 work experience entries
   - 10 skills (technical, soft, tools)
   - Education (UC Berkeley CS degree)
   - 2 certifications (AWS, Kubernetes)
   - 2 portfolio items
   - 3 job postings
   - 3 applications (saved, applied, interview)
   - Application events

### Requirements

- The database must already exist (run the app once to initialize it)
- Database location: `src-tauri/.careerbench/careerbench.db`

## Git Pre-commit Hook

A git pre-commit hook is installed at `.git/hooks/pre-commit` that automatically clears the database before each commit. This prevents accidentally committing test data or personal information.

The hook:
- Checks if the database exists
- Clears all data tables (keeps schema)
- Runs VACUUM to reclaim space
- Does not fail the commit if clearing fails (just warns)

### Requirements

- `sqlite3` command-line tool must be installed
- On macOS: `brew install sqlite3`
- On Linux: Usually pre-installed or `apt-get install sqlite3`
- On Windows: Download from sqlite.org or use WSL

### Disabling the hook

If you need to commit with data in the database:

```bash
# Skip hooks for one commit
git commit --no-verify -m "your message"

# Or temporarily disable
chmod -x .git/hooks/pre-commit
```

