# Testing Guide

This guide explains how to run tests and what they cover, following the [Testing & AI Guardrails Specification](../specs/testing/testing.md).

## Running Tests

### Rust Tests

Run all Rust tests:
```bash
cd src-tauri
cargo test
```

Run specific test modules:
```bash
# Unit tests for ai_cache
cargo test ai_cache

# Integration tests
cargo test --test integration_tests
```

### TypeScript Tests

Run all TypeScript tests:
```bash
npm test
```

Run tests in watch mode:
```bash
npm test -- --watch
```

Run tests with UI:
```bash
npm run test:ui
```

## Test Structure

### Rust Backend

- **Unit Tests** (`src/*/tests`): Test individual functions and modules
  - `ai_cache.rs`: Tests for cache hash computation, get/put operations, expiration
  - `ai_client.rs`: Tests for mock AI client behavior

- **Integration Tests** (`tests/integration_tests.rs`): Test Tauri commands end-to-end
  - Uses in-memory SQLite database
  - Tests deserialization of AI structs from fixtures
  - Tests rendering functions

### TypeScript Frontend

- **Unit Tests** (`src/test/utils.test.ts`): Test pure utility functions
- **UI Tests** (`src/test/pages.test.tsx`): Ensure pages render without crashing

## Test Fixtures

Golden test fixtures are stored in `tests/fixtures/`:
- `job_parsing/basic_job.json`: Example ParsedJob structure
- `resume_generation/basic_resume.json`: Example GeneratedResume structure
- `cover_letter/basic_letter.json`: Example GeneratedLetter structure

These fixtures are used to:
1. Test deserialization of AI responses
2. Ensure contract stability (changing structs breaks tests)
3. Validate rendering functions

## Adding New Tests

### For a New Rust Function

1. Add a `#[cfg(test)]` module at the bottom of the file
2. Write tests for all logic branches
3. Use `setup_test_db()` helper for DB-dependent tests

### For a New Tauri Command

1. Add a test in `tests/integration_tests.rs`
2. Use `setup_test_db()` to create a test database
3. Test both success and error cases
4. Use mock AI client (not real API calls)

### For a New TypeScript Utility

1. Add tests to `src/test/utils.test.ts` or create a new test file
2. Test edge cases and error handling
3. Keep tests focused on logic, not UI

## AI Guardrails

When working with AI-generated code:

1. **Never remove a test without replacing it**
2. **When changing Tauri command signatures**: Update integration tests and TypeScript types
3. **When changing AI structs**: Update fixtures and deserialization tests
4. **For new AI features**: Add at least 1 integration test and 1 golden-file test
5. **AI tools must not modify tests to match broken behavior** - fix the code instead

See [Testing & AI Guardrails Specification](./testing.md) for full details.

