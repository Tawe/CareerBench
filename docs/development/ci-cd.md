# CI/CD Documentation

This document describes the continuous integration and deployment setup for CareerBench.

## GitHub Actions Workflows

### Main CI Workflow (`.github/workflows/ci.yml`)

Runs on pushes and pull requests to `main` and `develop` branches.

**Jobs:**

1. **Rust Tests & Lint** (runs on Linux, macOS, Windows)
   - Checks Rust code formatting (`cargo fmt --check`)
   - Runs Clippy linter (`cargo clippy -- -D warnings`)
   - Runs Rust unit tests (`cargo test --lib`)
   - Runs Rust integration tests (`cargo test --test integration_tests`)
   - Verifies Rust compilation (`cargo check --lib`)

2. **Frontend Tests & Lint** (runs on Linux)
   - Checks TypeScript compilation (`tsc --noEmit`)
   - Runs frontend tests (`npm test -- --run`)

3. **Build Verification** (runs on Linux, macOS, Windows)
   - Builds frontend (`npm run build`)
   - Verifies Rust compilation
   - Ensures the project can be built across all platforms

### PR Checks Workflow (`.github/workflows/pr-checks.yml`)

Runs on pull request events (opened, synchronize, reopened) for quick feedback.

**Jobs:**

1. **Quick PR Checks** (runs on Linux)
   - Checks Rust formatting
   - Runs Clippy with warnings as errors
   - Checks TypeScript compilation
   - Runs Rust unit tests
   - Runs frontend tests

This workflow is optimized for speed to provide quick feedback on PRs.

## Caching

Both workflows use caching to speed up builds:

- **Cargo dependencies**: Cached based on `Cargo.lock` hash
- **npm dependencies**: Cached via `actions/setup-node@v4` with `cache: 'npm'`

## Required Status Checks

To block merges on test failures, configure branch protection rules in GitHub:

1. Go to repository Settings → Branches
2. Add a branch protection rule for `main` (and optionally `develop`)
3. Enable "Require status checks to pass before merging"
4. Select the following required checks:
   - `Rust Tests & Lint` (all platforms or just ubuntu-latest for speed)
   - `Frontend Tests & Lint`
   - `Quick PR Checks`

## Local Testing

Before pushing, you can run the same checks locally:

```bash
# Rust formatting
cd src-tauri && cargo fmt --check

# Rust linting
cd src-tauri && cargo clippy -- -D warnings

# Rust tests
cd src-tauri && cargo test

# TypeScript compilation
npx tsc --noEmit

# Frontend tests
npm test -- --run
```

## Adding New Checks

### Adding ESLint

1. Install ESLint:
   ```bash
   npm install --save-dev eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin
   ```

2. Create `.eslintrc.json` configuration

3. Add to workflows:
   ```yaml
   - name: Run ESLint
     run: npx eslint . --ext .ts,.tsx
   ```

### Adding More Test Types

Add new test jobs following the pattern in `ci.yml`. Ensure they:
- Use appropriate caching
- Run on appropriate platforms
- Fail fast on errors

## Troubleshooting

### Tests Pass Locally But Fail in CI

- Check for platform-specific issues (file paths, line endings)
- Verify all dependencies are in `package.json` or `Cargo.toml`
- Check for environment-specific behavior

### Slow CI Runs

- Verify caching is working (check Actions logs)
- Consider splitting jobs that can run in parallel
- Use `--test-threads=1` for Rust tests if there are race conditions

### Clippy Warnings

Fix Clippy warnings locally:
```bash
cd src-tauri
cargo clippy --fix
```

Then review and commit the changes.

