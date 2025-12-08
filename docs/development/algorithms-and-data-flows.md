# Complex Algorithms and Data Flows

This document describes the complex algorithms and data flows in CareerBench that require detailed understanding for maintenance and enhancement.

## Table of Contents

1. [Resume Generation Pipeline](#resume-generation-pipeline)
2. [AI Provider Resolution and Hybrid Mode](#ai-provider-resolution-and-hybrid-mode)
3. [AI Caching System](#ai-caching-system)
4. [Local AI Inference Pipeline](#local-ai-inference-pipeline)

---

## Resume Generation Pipeline

### Overview

The resume generation pipeline is a multi-step process that breaks down resume generation into smaller, focused AI calls with code-based preprocessing. This approach provides:

- **Determinism**: Code-based selection ensures consistent results
- **Efficiency**: Smaller AI calls are faster and cheaper
- **Caching**: Multiple cache points reduce redundant AI calls
- **Control**: Fine-grained control over each step

### Architecture

```
User Request
    ↓
[Step 0] Check Final Resume Cache
    ↓ (cache miss)
[Step 1] Summarize Job Description (cached)
    ↓
[Step 2] Map Experience to Job (code-based)
    ↓
[Step 3] Rewrite Bullets (cached per bullet)
    ↓
[Step 4] Generate Professional Summary (cached)
    ↓
[Step 5] Assemble Final Resume (code-based)
    ↓
[Step 6] Cache Final Resume
    ↓
Return to User
```

### Step-by-Step Flow

#### Step 0: Final Resume Cache Check

**Location**: `src-tauri/src/commands.rs::generate_resume_for_job`

**Purpose**: Check if we've already generated a resume for this exact combination of profile + job + options.

**Cache Key**: SHA256 hash of canonical JSON payload containing:
- User profile (full profile data)
- Experience (all experience entries)
- Skills (all skills)
- Education (all education entries)
- Job (title, company, raw description)
- Generation options (tone, length, focus)

**Cache TTL**: 30 days (`CACHE_TTL_RESUME_DAYS`)

**Implementation**:
```rust
let request_payload = serde_json::json!({
    "userProfile": profile_data.profile,
    "experience": profile_data.experience,
    "skills": profile_data.skills,
    "education": profile_data.education,
    "job": { "title": job.title, "company": job.company, "rawDescription": job.raw_description },
    "options": options
});

let input_hash = compute_input_hash(&request_payload)?;

if let Some(cached_entry) = ai_cache_get(&conn, "resume_generation", &input_hash, &now)? {
    // Return cached resume
}
```

#### Step 1: Summarize Job Description

**Location**: `src-tauri/src/resume_generator.rs::summarize_job_description`

**Purpose**: Extract key information from the job description to guide experience selection and bullet rewriting.

**Output**: `JobDescriptionSummary` containing:
- Role title
- Seniority level
- Must-have skills
- Nice-to-have skills
- Top responsibilities
- Tools/technologies
- Tone/style

**Cache Key**: SHA256 hash of:
- Job description text
- Parsed job JSON (if available)

**Cache TTL**: 90 days (`CACHE_TTL_JOB_PARSE_DAYS`)

**AI Call**: Uses `provider.parse_job()` and converts `ParsedJobOutput` to `JobDescriptionSummary`

**Why Cache**: Job descriptions don't change, so we can reuse summaries across multiple resume generations.

#### Step 2: Map Experience to Job

**Location**: `src-tauri/src/resume_generator.rs::map_experience_to_job`

**Purpose**: Select and rank relevant experience entries and bullet points for the job.

**Process**:
1. **Code-based filtering**: Filter experience by date, relevance keywords
2. **Relevance scoring**: Calculate relevance scores for each experience entry
3. **Bullet selection**: Select top N bullets per experience entry based on relevance

**Relevance Scoring Algorithm**:
- Count keyword matches (skills, tools, responsibilities)
- Weight matches by importance (must-have > nice-to-have)
- Calculate normalized score (0.0 - 1.0)

**No AI Call**: This step is purely code-based for speed and determinism.

**Output**: `Vec<MappedExperience>` with:
- Experience entry
- Relevance score
- Selected bullets with individual scores

#### Step 3: Rewrite Bullets

**Location**: `src-tauri/src/resume_generator.rs::rewrite_bullets_for_job`

**Purpose**: Rewrite selected bullet points to better match the job description.

**Process**:
1. Group bullets by experience entry
2. For each group, call AI to rewrite bullets
3. Cache rewritten bullets per group

**Cache Key**: SHA256 hash of:
- Original bullet texts
- Job description summary
- Experience context

**Cache TTL**: 30 days (`CACHE_TTL_RESUME_DAYS`)

**AI Call**: Small, focused call (~200-400 tokens input, ~150-300 tokens output per group)

**Why Group**: Reduces number of AI calls while maintaining context.

#### Step 4: Generate Professional Summary

**Location**: `src-tauri/src/resume_generator.rs::generate_professional_summary`

**Purpose**: Create a tailored professional summary for the resume.

**Cache Key**: SHA256 hash of:
- User profile
- Job description summary

**Cache TTL**: 30 days (`CACHE_TTL_RESUME_DAYS`)

**Fallback**: If profile already has a summary, use it. Otherwise, generate a simple summary from profile data.

#### Step 5: Assemble Final Resume

**Location**: `src-tauri/src/commands.rs::generate_resume_for_job`

**Purpose**: Combine all pieces into the final `GeneratedResume` structure.

**Process**:
1. Use professional summary from Step 4
2. Create experience sections from mapped/rewritten bullets
3. Add skills section (filtered by job requirements)
4. Add education section
5. Format as structured `GeneratedResume`

**No AI Call**: Pure code-based assembly.

#### Step 6: Cache Final Resume

**Location**: `src-tauri/src/commands.rs::generate_resume_for_job`

**Purpose**: Store the final resume for future identical requests.

**Cache Key**: Same as Step 0 (final resume cache)

**Cache TTL**: 30 days (`CACHE_TTL_RESUME_DAYS`)

### Cache Hit Scenarios

1. **Full Cache Hit (Step 0)**: User generates resume for same job/profile/options → Instant return
2. **JD Summary Cache Hit (Step 1)**: Same job description → Skip AI call, reuse summary
3. **Bullet Rewrite Cache Hit (Step 3)**: Same bullets + same job → Reuse rewritten bullets
4. **Summary Cache Hit (Step 4)**: Same profile + same job → Reuse professional summary

### Performance Characteristics

- **Best Case** (all cache hits): ~10-50ms (database lookups only)
- **Worst Case** (all cache misses): ~5-15 seconds (multiple AI calls)
- **Typical Case** (partial cache hits): ~2-5 seconds (1-2 AI calls)

### Benefits of This Approach

1. **Cost Efficiency**: Smaller AI calls cost less than one large call
2. **Speed**: Code-based steps are instant; only AI steps take time
3. **Caching**: Multiple cache points maximize reuse
4. **Determinism**: Code-based selection ensures consistent results
5. **Maintainability**: Each step is isolated and testable

---

## AI Provider Resolution and Hybrid Mode

### Overview

The AI provider system supports three modes: Local, Cloud, and Hybrid. The resolver determines which provider to use based on user settings and availability.

### Provider Resolution Flow

**Location**: `src-tauri/src/ai/resolver.rs::ResolvedProvider::resolve()`

```
Load AI Settings
    ↓
Check Mode
    ├─→ Local Mode
    │   ├─→ Check local model path configured
    │   ├─→ Verify model file exists
    │   └─→ Return LocalProvider
    │
    ├─→ Cloud Mode
    │   ├─→ Check API key configured
    │   ├─→ Get cloud provider (OpenAI/Anthropic)
    │   ├─→ Get model name
    │   └─→ Return CloudAiProvider
    │
    └─→ Hybrid Mode
        ├─→ Initialize HybridProvider
        ├─→ Configure cloud provider (if API key available)
        ├─→ Configure local provider (if model path available)
        └─→ Return HybridProvider
```

### Hybrid Mode Routing Logic

**Location**: `src-tauri/src/ai/hybrid_provider.rs`

**Purpose**: Intelligently route AI requests between cloud and local providers with automatic fallback.

#### Provider Preference

- **If both configured**: Prefers cloud (faster, higher quality)
- **If only one configured**: Uses that provider
- **If neither configured**: Returns error

#### Fallback Logic

```
Primary Provider Attempt
    ↓
Success? → Return Result
    ↓
Failure? → Check Error Type
    ├─→ Recoverable Error?
    │   ├─→ Network Error → Try Fallback
    │   ├─→ Rate Limit → Try Fallback
    │   └─→ Invalid Response → Try Fallback
    │
    └─→ Non-Recoverable Error?
        ├─→ Invalid API Key → Return Error (no fallback)
        ├─→ Validation Error → Return Error (no fallback)
        └─→ Model Not Found → Return Error (no fallback)
```

#### Recoverable vs Non-Recoverable Errors

**Recoverable** (triggers fallback):
- `NetworkError`: Connection issues, timeouts
- `RateLimitExceeded`: API rate limits
- `InvalidResponse`: Malformed API response
- `Unknown` errors containing "network", "connection", "timeout", "unavailable"

**Non-Recoverable** (no fallback):
- `InvalidApiKey`: API key is wrong (fallback won't help)
- `ValidationError`: Input validation failed (same input will fail on fallback)
- `ModelNotFound`: Local model file missing (fallback won't help)

#### Implementation Details

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

### Benefits of Hybrid Mode

1. **Resilience**: Automatic fallback on transient errors
2. **Cost Control**: Use local for development, cloud for production
3. **Privacy**: Sensitive data can use local provider
4. **Performance**: Cloud for speed, local for offline

---

## AI Caching System

### Overview

The AI caching system prevents redundant AI API calls by storing request/response pairs keyed by a hash of the input.

### Cache Key Generation

**Location**: `src-tauri/src/ai_cache.rs::compute_input_hash`

**Algorithm**:
1. Serialize input JSON payload to string
2. Compute SHA256 hash of serialized string
3. Return hex-encoded hash (64 characters)

**Properties**:
- **Deterministic**: Same input always produces same hash
- **Collision-resistant**: SHA256 provides strong collision resistance
- **Fast**: Hash computation is O(n) where n is input size

**Example**:
```rust
let payload = json!({
    "jobDescription": "Software Engineer...",
    "parsedJob": {...}
});

let hash = compute_input_hash(&payload)?;
// Returns: "a1b2c3d4e5f6..." (64 hex chars)
```

### Cache Lookup

**Location**: `src-tauri/src/ai_cache.rs::ai_cache_get`

**Process**:
1. Query database for entry matching `purpose` + `input_hash`
2. Check expiration (if `expires_at` < current time, return `None`)
3. Return cached `response_payload` if found

**SQL Query**:
```sql
SELECT id, purpose, input_hash, model_name, request_payload, response_payload, created_at, expires_at
FROM ai_cache
WHERE purpose = ? AND input_hash = ?
```

**Expiration Check**:
- If `expires_at` is `None`: Entry never expires
- If `expires_at` < current time: Entry expired, return `None`
- Otherwise: Entry valid, return cached response

### Cache Storage

**Location**: `src-tauri/src/ai_cache.rs::ai_cache_put`

**Process**:
1. Serialize request and response payloads to JSON strings
2. Calculate expiration date (if TTL provided)
3. Insert into `ai_cache` table

**TTL Calculation**:
```rust
let expires_at = if let Some(days) = ttl_days {
    let now = DateTime::parse_from_rfc3339(now_iso)?;
    let expires = now + chrono::Duration::days(days);
    Some(expires.to_rfc3339())
} else {
    None // Never expires
};
```

### Cache Purposes

Different cache purposes have different TTLs:

- `jd_summary`: 90 days (job descriptions rarely change)
- `resume_generation`: 30 days (profile may change)
- `cover_letter_generation`: 30 days (profile may change)
- `professional_summary`: 30 days (profile may change)
- `bullet_rewrite`: 30 days (profile may change)

### Cache Invalidation

**Automatic**: Entries expire based on TTL

**Manual**: 
- `ai_cache_clear_purpose(conn, purpose)`: Clear all entries for a purpose
- `ai_cache_clear_all(conn)`: Clear all cache entries

### Cache Benefits

1. **Cost Savings**: Avoid redundant API calls
2. **Speed**: Database lookup is much faster than AI call
3. **Consistency**: Same input always returns same output
4. **Offline Support**: Cached responses work offline

### Cache Limitations

1. **Storage**: Cache can grow large over time (consider cleanup)
2. **Staleness**: Cached data may become outdated (TTL mitigates)
3. **Model Changes**: Changing AI model doesn't invalidate cache (by design)

---

## Local AI Inference Pipeline

### Overview

The local AI inference pipeline uses `llama.cpp` via Rust bindings to run GGUF models locally without internet connectivity.

### Model Loading

**Location**: `src-tauri/src/ai/llama_wrapper.rs::LlamaModel::new`

**Process**:
1. Load GGUF model file from disk
2. Initialize model context with parameters:
   - Context size (number of tokens)
   - GPU layers (if GPU available)
   - Thread count
3. Allocate memory for model weights and context

**Memory Requirements**:
- Model weights: ~2-4GB for typical models (Phi-3-mini, Llama 3.2 3B)
- Context buffer: ~100-500MB depending on context size
- Total: ~2.5-5GB RAM

### Tokenization

**Location**: `src-tauri/src/ai/llama_wrapper.rs::LlamaModel::tokenize`

**Process**:
1. Convert input text string to token IDs using model's tokenizer
2. Return vector of token IDs

**Token Limits**:
- Input tokens: Limited by context size (typically 2048-4096)
- Output tokens: Limited by generation parameters

### Inference

**Location**: `src-tauri/src/ai/llama_wrapper.rs::LlamaModel::generate`

**Process**:

#### 1. Prepare Initial Batch

```rust
// Create batch for prompt tokens
let batch = llama_batch_get_one(prompt_tokens.as_slice(), n_past, 0, 0);
// Set logits flag for last token (needed for next token prediction)
*batch.logits = 1;
```

**Key Points**:
- `llama_batch_get_one()` creates a stack-allocated batch (don't free it)
- Must set `logits` flag for tokens that need logits computation
- `n_past` tracks number of tokens already processed

#### 2. Decode Prompt

```rust
llama_decode(ctx, batch);
```

This processes the prompt tokens and prepares the model for generation.

#### 3. Generate Tokens

```rust
loop {
    // Get logits for next token
    let logits = llama_get_logits(ctx);
    
    // Sample next token (using temperature, top-p, etc.)
    let next_token = sample_token(logits, temperature, top_p);
    
    // Stop if EOS token or max tokens reached
    if next_token == eos_token || generated_tokens >= max_tokens {
        break;
    }
    
    // Prepare batch for next token
    let batch = llama_batch_get_one(&[next_token], n_past, 0, 0);
    *batch.logits = 1; // Need logits for next iteration
    
    // Decode next token
    llama_decode(ctx, batch);
    
    n_past += 1;
    generated_tokens += 1;
}
```

**Sampling Parameters**:
- **Temperature**: Controls randomness (0.0 = deterministic, 1.0+ = creative)
- **Top-p (nucleus)**: Limits sampling to top-p probability mass
- **Top-k**: Limits sampling to top-k tokens

#### 4. Detokenize Output

```rust
let output_text = model.detokenize(&generated_token_ids);
```

Convert token IDs back to text string.

### Batch Management

**Critical**: `llama_batch_get_one()` creates stack-allocated batches that should **NOT** be freed with `llama_batch_free()`. Only batches created with `llama_batch_init()` should be freed.

**Logits Flag**: Must set `batch.logits = 1` (or `*batch.logits.add(idx) = 1`) for tokens that need logits computation. This is required for:
- Last token of prompt (to get first generated token)
- Each generated token (to get next token)

### Error Handling

**Common Errors**:
- `GGML_ASSERT`: Usually indicates incorrect batch handling or logits flag
- `ModelNotFound`: Model file doesn't exist or is invalid
- `OutOfMemory`: Context size too large for available RAM

### Performance Characteristics

- **Model Loading**: ~2-5 seconds (one-time cost)
- **Tokenization**: ~1-10ms (negligible)
- **Inference Speed**: ~10-50 tokens/second (CPU), ~50-200 tokens/second (GPU)
- **Total Generation Time**: ~5-30 seconds for 200-500 tokens

### Memory Management

- Model is loaded once and kept in memory
- Context is reused across multiple generations
- Tokens are streamed (not all loaded at once)
- Memory is freed when model is dropped

---

## Summary

These complex algorithms and data flows form the core of CareerBench's AI-powered features. Understanding them is essential for:

- **Debugging**: Knowing where to look when issues arise
- **Optimization**: Identifying bottlenecks and improvement opportunities
- **Enhancement**: Adding new features that integrate with existing systems
- **Maintenance**: Keeping the codebase healthy and performant

For questions or clarifications, refer to the source code or create an issue in the repository.

