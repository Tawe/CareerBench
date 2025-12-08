# ADR-0002: Pluggable AI Provider Architecture

## Status

Accepted

## Context

CareerBench needs AI capabilities for:
- Job description parsing
- Resume generation
- Cover letter generation
- Skill suggestions

We need to support multiple AI backends:
- **Local models**: Privacy-friendly, offline-capable, no API costs
- **Cloud providers**: Higher quality, faster, but requires API keys and internet
- **Hybrid mode**: Best of both worlds with automatic fallback

The application should be agnostic to which AI provider is used - the same code should work regardless of whether we're using a local model or cloud API.

## Decision

We implemented a **pluggable AI provider architecture** using a trait-based design in Rust.

### Architecture

```rust
// Trait defining AI operations
trait AiProvider {
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions>;
    async fn generate_cover_letter(&self, input: CoverLetterInput) -> Result<CoverLetter>;
    async fn generate_skill_suggestions(&self, input: SkillSuggestionsInput) -> Result<SkillSuggestions>;
    async fn parse_job(&self, input: JobParsingInput) -> Result<ParsedJobOutput>;
}

// Implementations
struct LocalProvider { ... }      // llama.cpp integration
struct CloudAiProvider { ... }    // OpenAI, Anthropic APIs
struct HybridProvider { ... }     // Automatic fallback
```

### Provider Resolution

```rust
enum ResolvedProvider {
    Local(Arc<LocalProvider>),
    Cloud(Arc<CloudAiProvider>),
    Hybrid(Arc<HybridProvider>),
}

impl ResolvedProvider {
    fn resolve() -> Result<Self, String> {
        // Resolves based on user settings
        // Returns appropriate provider implementation
    }
}
```

### Key Design Principles

1. **Unified Interface**: All providers implement the same trait
2. **Type Safety**: Strongly typed inputs/outputs prevent errors
3. **Async Support**: All operations are async to support both local and cloud
4. **Error Handling**: Comprehensive error types cover all failure modes
5. **Settings-Driven**: User chooses provider via settings UI

## Consequences

### Positive

- **Flexibility**: Easy to add new providers (just implement the trait)
- **Testability**: Can use mock providers for testing
- **User Choice**: Users can choose based on privacy, cost, quality needs
- **Maintainability**: Provider-specific code is isolated
- **Type Safety**: Compile-time guarantees prevent misuse

### Negative

- **Abstraction Overhead**: Trait objects have slight performance cost (negligible)
- **Complexity**: More code to maintain (but better organized)
- **Error Handling**: Need to handle provider-specific errors uniformly

### Mitigations

- **Mock Provider**: Created `MockProvider` for deterministic testing
- **Error Types**: Unified error types with provider-agnostic messages
- **Documentation**: Clear documentation on adding new providers
- **Integration Tests**: Tests ensure all providers work correctly

## Implementation Details

### Provider Types

1. **LocalProvider**: Uses `llama.cpp` C API for GGUF model inference
2. **CloudAiProvider**: HTTP client for OpenAI/Anthropic APIs
3. **HybridProvider**: Tries primary provider, falls back on recoverable errors

### Settings Storage

AI settings stored in SQLite `ai_settings` table:
- `mode`: "local" | "cloud" | "hybrid"
- `cloud_provider`: "openai" | "anthropic"
- `api_key`: Encrypted API key (future: use Tauri secure storage)
- `model_name`: Model identifier
- `local_model_path`: Path to GGUF file

### Caching

All AI operations are cached using input hash:
- Same input → same output (deterministic)
- Reduces API costs and improves performance
- Cache TTL varies by operation type

## Alternatives Considered

### Direct Integration

**Approach**: Hard-code OpenAI API calls directly in commands
**Why not chosen**: No flexibility, can't support local models, harder to test

### Strategy Pattern (Runtime Selection)

**Approach**: Use enum with match statements instead of trait
**Why not chosen**: Less flexible, harder to add new providers, more code duplication

### Factory Pattern

**Approach**: Factory creates provider instances
**Why not chosen**: Trait-based approach is more idiomatic Rust, better type safety

## References

- AI Provider Specification: `docs/specs/ai-provider.md`
- Implementation: `src-tauri/src/ai/`
- Algorithms Documentation: `docs/development/algorithms-and-data-flows.md`

