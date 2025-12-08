# ADR-0003: Multi-Level AI Caching Strategy

## Status

Accepted

## Context

AI API calls are:
- **Expensive**: Cloud APIs charge per token (input + output)
- **Slow**: Network latency adds 1-5 seconds per call
- **Redundant**: Same inputs often produce same outputs

Resume generation involves multiple AI calls:
1. Job description summary
2. Experience mapping
3. Bullet rewriting (multiple groups)
4. Professional summary
5. Final resume assembly

Without caching, generating a resume for the same job twice would:
- Cost 2x in API fees
- Take 2x the time
- Produce identical results

## Decision

We implemented a **multi-level caching strategy** with cache points at different stages of the pipeline.

### Cache Architecture

```
┌─────────────────────────────────────────┐
│  Final Resume Cache (Step 0)            │
│  Key: profile + job + options hash      │
│  TTL: 30 days                           │
└──────────────┬──────────────────────────┘
               │ Cache miss
┌──────────────▼──────────────────────────┐
│  JD Summary Cache (Step 1)             │
│  Key: job description hash             │
│  TTL: 90 days                           │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  Bullet Rewrite Cache (Step 3)         │
│  Key: bullets + job summary hash       │
│  TTL: 30 days                           │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│  Professional Summary Cache (Step 4)   │
│  Key: profile + job summary hash       │
│  TTL: 30 days                           │
└─────────────────────────────────────────┘
```

### Cache Key Generation

**Algorithm**: SHA256 hash of canonical JSON payload

```rust
fn compute_input_hash(json_payload: &Value) -> Result<String, String> {
    let serialized = serde_json::to_string(json_payload)?;
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}
```

**Properties**:
- **Deterministic**: Same input → same hash
- **Collision-resistant**: SHA256 provides strong guarantees
- **Fast**: O(n) where n is input size

### Cache Storage

**Location**: SQLite `ai_cache` table

**Schema**:
```sql
CREATE TABLE ai_cache (
    id INTEGER PRIMARY KEY,
    purpose TEXT NOT NULL,           -- "jd_summary", "resume_generation", etc.
    input_hash TEXT NOT NULL,        -- SHA256 hash of input
    model_name TEXT NOT NULL,        -- Which model generated this
    request_payload TEXT NOT NULL,   -- Original request (for debugging)
    response_payload TEXT NOT NULL,  -- Cached response
    created_at TEXT NOT NULL,        -- ISO 8601 timestamp
    expires_at TEXT                  -- ISO 8601 timestamp (nullable)
);
```

**Indexes**: `(purpose, input_hash)` for fast lookups

### Cache TTL Strategy

Different cache purposes have different TTLs based on how often data changes:

- **JD Summary (90 days)**: Job descriptions rarely change
- **Resume Generation (30 days)**: Profile may change, but job stays same
- **Bullet Rewrite (30 days)**: Experience may be updated
- **Professional Summary (30 days)**: Profile summary may change

### Cache Hit Scenarios

1. **Full Cache Hit**: User generates resume for same job/profile/options → Instant return (~10-50ms)
2. **JD Summary Hit**: Same job description → Skip AI call, reuse summary
3. **Bullet Rewrite Hit**: Same bullets + same job → Reuse rewritten bullets
4. **Summary Hit**: Same profile + same job → Reuse professional summary

## Consequences

### Positive

- **Cost Savings**: Avoid redundant API calls (can save 80-90% on repeated operations)
- **Speed**: Database lookup is 100-1000x faster than AI call
- **Consistency**: Same input always returns same output (deterministic)
- **Offline Support**: Cached responses work without internet
- **User Experience**: Faster response times for common operations

### Negative

- **Storage Growth**: Cache can grow large over time (needs cleanup)
- **Staleness**: Cached data may become outdated (TTL mitigates)
- **Memory**: Large cache entries consume database space
- **Complexity**: More code to maintain cache logic

### Mitigations

- **TTL Expiration**: Automatic expiration prevents infinite growth
- **Cache Cleanup**: Manual cleanup functions available (`ai_cache_clear_purpose`, `ai_cache_clear_all`)
- **Monitoring**: Can query cache size and hit rates (future enhancement)
- **Selective Caching**: Only cache expensive operations (AI calls, not code-based steps)

## Performance Impact

### Without Caching

- **Resume Generation**: ~5-15 seconds (multiple AI calls)
- **Cost**: ~$0.01-0.05 per generation (cloud APIs)
- **User Experience**: Slow, especially on repeated operations

### With Caching

- **Cache Hit**: ~10-50ms (database lookup)
- **Partial Cache Hit**: ~2-5 seconds (1-2 AI calls instead of 4-5)
- **Cost**: $0.00 for cache hits, reduced cost for partial hits
- **User Experience**: Much faster for common operations

### Typical Cache Hit Rates

- **First Generation**: 0% (all cache misses)
- **Regeneration (same job)**: 100% (full cache hit)
- **New Job (same profile)**: ~30-50% (JD summary, bullets may hit)
- **New Profile (same job)**: ~10-20% (JD summary may hit)

## Alternatives Considered

### Single-Level Caching

**Approach**: Only cache final resume output
**Why not chosen**: Less granular, misses opportunities to cache intermediate steps

### No Caching

**Approach**: Always call AI APIs
**Why not chosen**: Too expensive and slow, poor user experience

### In-Memory Caching

**Approach**: Cache in application memory instead of database
**Why not chosen**: Lost on app restart, can't share across sessions, memory limits

### Time-Based Invalidation

**Approach**: Invalidate cache after fixed time (e.g., 24 hours)
**Why not chosen**: TTL-based expiration is more flexible and efficient

## Future Enhancements

1. **Cache Statistics**: Track hit rates, cache size, most-used entries
2. **Cache Warming**: Pre-populate cache for common operations
3. **Selective Invalidation**: Invalidate related caches when profile/job changes
4. **Cache Compression**: Compress large cache entries to save space
5. **Distributed Caching**: Share cache across multiple CareerBench instances (future)

## References

- Algorithms Documentation: `docs/development/algorithms-and-data-flows.md#ai-caching-system`
- Implementation: `src-tauri/src/ai_cache.rs`
- Resume Generation: `src-tauri/src/resume_generator.rs`

