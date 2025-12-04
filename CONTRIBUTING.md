# Contributing to CareerBench

Thank you for your interest in contributing to CareerBench!

## Development Setup

See [QUICKSTART.md](./QUICKSTART.md) for setup instructions.

## Testing Requirements

All code changes must include appropriate tests. See [docs/specs/testing/TESTING_GUIDE.md](./docs/specs/testing/TESTING_GUIDE.md) for details.

## AI-Assisted Development Guardrails

When using AI tools (like Cursor) to generate or modify code, follow these rules:

### 1. Never Remove Tests Without Replacement
- If you delete a test, you must add a new one that covers the same behavior
- Or explain in comments why the test is obsolete

### 2. When Changing Tauri Command Signatures
- Update integration tests in `src-tauri/tests/integration_tests.rs`
- Update TypeScript types that use `invoke<...>` generics
- Update any AI-facing specs that reference the command

### 3. When Changing AI Structs
When modifying `GeneratedResume`, `GeneratedLetter`, `ParsedJob`, or similar:
- Update the Rust struct definition
- Update fixture files in `tests/fixtures/`
- Update deserialization tests
- Update any TypeScript interfaces that mirror them

### 4. For New AI-Backed Features
Add at least:
- 1 mock-based integration test that exercises the full flow
- 1 golden-file deserialization test

### 5. AI Tools Must Not Modify Tests to Match Broken Behavior
- If a test is failing, fix the **code** or update the **intent** explicitly
- Don't just relax the test to make it pass

### 6. Prompt Discipline
- Specs live in `/docs/specs`
- When asking AI to change logic, reference the relevant spec:
  > "Update this function according to `CareerBench Local AI Caching Spec` and add/adjust unit tests to cover cache hit/miss behavior."

## Code Style

- Rust: Follow standard Rust conventions
- TypeScript: Use TypeScript strict mode, follow React best practices
- Format code before committing

## Running Tests

```bash
# Rust tests
cd src-tauri && cargo test

# TypeScript tests
npm test
```

## Submitting Changes

1. Ensure all tests pass
2. Update documentation if needed
3. Submit a pull request with a clear description

For full testing specifications, see [docs/specs/testing/testing.md](./docs/specs/testing/testing.md).

