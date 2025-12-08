# ADR-0004: Hybrid Mode Routing with Automatic Fallback

## Status

Accepted

## Context

Users want flexibility in AI provider selection:
- **Privacy-conscious users**: Prefer local models (data never leaves device)
- **Quality-focused users**: Prefer cloud APIs (better models, faster)
- **Cost-conscious users**: Want to use local when possible, cloud when needed
- **Reliability-focused users**: Want automatic fallback if one provider fails

We need a way to support multiple providers simultaneously with intelligent routing and fallback logic.

## Decision

We implemented **Hybrid Mode** with automatic fallback between cloud and local providers.

### Architecture

```rust
struct HybridProvider {
    cloud_provider: Option<Arc<CloudAiProvider>>,
    local_provider: Option<Arc<LocalProvider>>,
    prefer_cloud: bool,  // If both available, prefer cloud
}

impl AiProvider for HybridProvider {
    async fn generate_resume_suggestions(&self, input: ResumeInput) -> Result<ResumeSuggestions> {
        self.try_with_fallback(|provider| {
            provider.generate_resume_suggestions(input)
        }).await
    }
}
```

### Routing Logic

1. **Provider Preference**:
   - If both cloud and local configured: Prefer cloud (faster, higher quality)
   - If only one configured: Use that provider
   - If neither configured: Return error

2. **Fallback Strategy**:
   ```
   Try Primary Provider
       ↓
   Success? → Return Result
       ↓
   Failure? → Check Error Type
       ├─→ Recoverable? → Try Fallback Provider
       └─→ Non-Recoverable? → Return Error (no fallback)
   ```

3. **Error Classification**:

   **Recoverable** (triggers fallback):
   - `NetworkError`: Connection issues, timeouts
   - `RateLimitExceeded`: API rate limits
   - `InvalidResponse`: Malformed API response
   - `Unknown` errors containing "network", "connection", "timeout", "unavailable"

   **Non-Recoverable** (no fallback):
   - `InvalidApiKey`: API key is wrong (fallback won't help)
   - `ValidationError`: Input validation failed (same input will fail on fallback)
   - `ModelNotFound`: Local model file missing (fallback won't help)

### Implementation

```rust
async fn try_with_fallback<F, Fut, T>(&self, operation: F) -> Result<T, AiProviderError>
where
    F: Fn(Arc<dyn AiProvider>) -> Fut,
    Fut: Future<Output = Result<T, AiProviderError>>,
{
    // Try primary provider
    if let Some(provider) = primary {
        match operation(provider).await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if Self::is_recoverable_error(&error) && fallback.is_some() {
                    // Try fallback
                } else {
                    return Err(error); // Non-recoverable
                }
            }
        }
    }
    
    // Try fallback provider
    if let Some(provider) = fallback {
        operation(provider).await
    } else {
        Err(AiProviderError::Unknown("No provider available".to_string()))
    }
}
```

## Consequences

### Positive

- **Resilience**: Automatic fallback on transient errors improves reliability
- **User Choice**: Users can configure both providers and get best of both worlds
- **Cost Control**: Can use local for development/testing, cloud for production
- **Privacy**: Sensitive data can use local provider, less sensitive can use cloud
- **Performance**: Cloud for speed when available, local for offline capability

### Negative

- **Complexity**: More code to maintain, more edge cases to handle
- **Error Handling**: Need to classify errors correctly (recoverable vs non-recoverable)
- **Latency**: Fallback adds latency on primary provider failure (but still faster than manual retry)
- **Testing**: Need to test all fallback scenarios

### Mitigations

- **Clear Error Types**: Comprehensive error enum makes classification straightforward
- **Logging**: Detailed logging helps debug fallback decisions
- **Testing**: Integration tests cover fallback scenarios
- **Documentation**: Clear documentation on error classification

## Use Cases

### Use Case 1: Cloud Primary, Local Fallback

**Scenario**: User has both cloud API key and local model configured
**Behavior**: 
- Tries cloud first (faster, higher quality)
- Falls back to local on network errors or rate limits
- **Benefit**: Best performance with resilience

### Use Case 2: Local Primary, Cloud Fallback

**Scenario**: User prefers local but has cloud as backup
**Behavior**:
- Tries local first (privacy, no cost)
- Falls back to cloud on model errors or slow performance
- **Benefit**: Privacy-first with quality backup

### Use Case 3: Single Provider

**Scenario**: User only has one provider configured
**Behavior**:
- Uses that provider (no fallback available)
- Returns error if provider fails (expected behavior)
- **Benefit**: Simpler configuration, still works

## Alternatives Considered

### Manual Provider Selection

**Approach**: User manually chooses provider per operation
**Why not chosen**: Too much friction, users don't want to think about it

### Always Use Cloud with Local Fallback

**Approach**: Always prefer cloud, only use local if cloud unavailable
**Why not chosen**: Doesn't respect user privacy preferences

### No Automatic Fallback

**Approach**: User must manually retry with different provider
**Why not chosen**: Poor user experience, users expect automatic recovery

### Task-Based Routing

**Approach**: Route different tasks to different providers (e.g., parsing → local, generation → cloud)
**Why not chosen**: Too complex, user preferences may vary, current approach is more flexible

## Future Enhancements

1. **User Preferences**: Allow users to set per-task provider preferences
2. **Performance-Based Routing**: Route to faster provider based on historical performance
3. **Cost-Based Routing**: Route to cheaper provider when quality difference is negligible
4. **Health Checks**: Proactively check provider health and route accordingly
5. **Fallback Metrics**: Track fallback frequency and reasons for optimization

## References

- Algorithms Documentation: `docs/development/algorithms-and-data-flows.md#ai-provider-resolution-and-hybrid-mode`
- Implementation: `src-tauri/src/ai/hybrid_provider.rs`
- Provider Resolution: `src-tauri/src/ai/resolver.rs`

