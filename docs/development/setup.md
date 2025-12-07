# Development Setup Guide

Complete guide for setting up a CareerBench development environment.

## Prerequisites

- **Node.js** (v18 or later)
- **Rust** (latest stable)
- **npm** or **yarn**
- **Git**

## Initial Setup

### 1. Clone the Repository

```bash
git clone <repository-url>
cd CareerBench
```

### 2. Install Dependencies

```bash
# Install Node.js dependencies
npm install

# Rust dependencies are managed by Cargo and install automatically
```

### 3. Initialize Database

The database is created automatically on first run. To manually initialize:

```bash
# Run the app once - database will be created
npm run tauri dev
```

Or use the test data script:

```bash
./scripts/populate_test_data.sh
```

## Development Workflow

### Running in Development Mode

```bash
npm run tauri dev
```

This will:
1. Start the Vite dev server on http://localhost:1420
2. Compile the Rust backend
3. Launch the Tauri window with hot-reload

### Building for Production

```bash
npm run tauri build
```

Creates a distributable app in `src-tauri/target/release/bundle/`

### Running Tests

```bash
# Rust tests
cd src-tauri && cargo test

# TypeScript tests
npm test
```

## Project Structure

```
CareerBench/
├── src/                    # React frontend
│   ├── components/         # Reusable React components
│   ├── pages/              # Page components
│   └── main.tsx            # Entry point
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # App entry & Tauri setup
│   │   ├── db.rs           # Database & migrations
│   │   ├── ai_cache.rs     # AI caching layer
│   │   ├── commands.rs     # Tauri commands
│   │   └── ai/             # AI provider system
│   └── Cargo.toml          # Rust dependencies
├── scripts/                # Utility scripts
│   ├── populate_test_data.rs  # Test data generator
│   └── populate_test_data.sh  # Wrapper script
└── docs/                   # Documentation
```

## Database

### Location

- **Development:** `src-tauri/.careerbench/careerbench.db`
- **Production:** Platform-specific app data directory

### Migrations

Migrations are run automatically on startup. See `src-tauri/src/db.rs` for migration definitions.

### Test Data

Populate test data for development:

```bash
./scripts/populate_test_data.sh
```

## Code Style

### Rust

- Follow standard Rust conventions
- Use `cargo fmt` to format code
- Use `cargo clippy` for linting

### TypeScript/React

- Use TypeScript strict mode
- Follow React best practices
- Format with Prettier (if configured)

## Debugging

### Frontend

- Use browser DevTools (when running in dev mode)
- React DevTools extension recommended
- Check console for errors

### Backend

- Check logs: `src-tauri/.careerbench/careerbench.log`
- Use `log::debug!()` for detailed logging
- Rust debugging with VS Code or your IDE

### Database

- Use SQLite browser tools to inspect database
- Location: `src-tauri/.careerbench/careerbench.db`

## Common Tasks

### Reset Database

```bash
rm -rf src-tauri/.careerbench
# Restart app - migrations will recreate database
```

### Clean Build

```bash
# Clean Rust build
cd src-tauri && cargo clean

# Clean frontend
rm -rf node_modules dist
npm install
```

### Update Dependencies

```bash
# Node.js
npm update

# Rust
cd src-tauri && cargo update
```

## IDE Setup

### VS Code

Recommended extensions:
- Rust Analyzer
- Tauri (official extension)
- ESLint
- Prettier

### Configuration

- Rust: Use `rust-analyzer` for best experience
- TypeScript: Configure `tsconfig.json` for strict mode

## Git Hooks

A pre-commit hook is installed to clear the database before commits. See `.git/hooks/pre-commit`.

To skip:
```bash
git commit --no-verify
```

## Troubleshooting

See the [Troubleshooting Guide](../guides/troubleshooting.md) for common issues.

## Related Documentation

- [Quick Start Guide](../../QUICKSTART.md) - Quick setup
- [Contributing Guide](../../CONTRIBUTING.md) - Contribution guidelines
- [Testing Guide](testing.md) - Testing standards
- [Architecture Overview](architecture.md) - System design

