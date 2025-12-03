# CareerBench – Local AI Caching Specification

This spec is intended to guide Cursor AI when generating code for the **Local AI Caching** layer in the CareerBench Tauri desktop application.

The goal is to **avoid repeated, identical AI calls** (and extra cost/time) by caching responses in the local SQLite database, while keeping the design simple and feature-agnostic.

This cache will support multiple features, including:

- Job parsing (Basic Job Parsing)
- Resume generation
- Cover letter / outreach email generation
- Future features (learning plan generator, coaching, etc.)

---

## 1. Goals & Scope

### 1.1 Primary Goals

- Provide a **centralized local cache** for AI responses.
- Make caching **feature-agnostic** via a `purpose` + `input_hash` model.
- Ensure that:
    - Given the same _logical_ input, CareerBench reuses the previous AI result.
    - Cache access is simple from any Tauri command.
- Support **observability** for debugging (what was cached, when).

### 1.2 In-Scope (MVP)

- SQLite `ai_cache` table.
- Rust helper functions to:
    - Generate a canonical input hash.
    - Lookup cached responses by `(purpose, input_hash)`.
    - Insert new cache records.
- Integration points for:
    - Job parsing (`purpose = "job_parse"`).
    - Resume generation (`purpose = "resume_generation"`).
    - Cover letter generation (`purpose = "cover_letter_generation"`).  

### 1.3 Out-of-Scope (for this spec)

- UI to manually view/clear cache (can be a dev tool later).
- Multi-user or multi-device sync.
- Advanced invalidation policies (beyond basic TTL and manual clear hooks).

---

## 2. Data Model – `ai_cache` Table

### 2.1 Table Definition

Each row represents a single AI response cached for a given logical input.

**Fields:**
- `id` (INTEGER PRIMARY KEY)
- `purpose` (TEXT, required)
    - Short string describing the use case, e.g. `"job_parse"`, `"resume_generation"`.
- `input_hash` (TEXT, required, indexed)
    - Stable hash of the logical input.
- `model_name` (TEXT, required)
    - Name/identifier of the model used, e.g. `"gpt-4.1"`, `"gemini-2.5"`.
- `request_payload` (TEXT, required)
    - JSON string containing the _canonical_ input context (after any normalization).
- `response_payload` (TEXT, required)
    - Raw JSON string returned by the AI model.
- `created_at` (TEXT, ISO8601, required)
- `expires_at` (TEXT, ISO8601, optional)
    - When this cache entry should be considered expired. Nullable for "no expiry".

**Suggested indexes:**

- `INDEX idx_ai_cache_purpose_input_hash (purpose, input_hash)`

### 2.2 Example SQLite Migration (Pseudo)

```sql
CREATE TABLE IF NOT EXISTS ai_cache (
  id INTEGER PRIMARY KEY,
  purpose TEXT NOT NULL,
  input_hash TEXT NOT NULL,
  model_name TEXT NOT NULL,
  request_payload TEXT NOT NULL,
  response_payload TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_ai_cache_purpose_input_hash
  ON ai_cache (purpose, input_hash);
```

> Cursor should adapt this into the actual migration format used in the project.

---

## 3. Hashing Strategy – `input_hash`

### 3.1 Canonical Input JSON

For each feature using the cache, we must build a **canonical JSON payload** representing the logical input.

Examples:

- **Job Parsing (**`**job_parse**`**):**
    
    ```json
    {
      "jobDescription": "...full raw text...",
      "jobMeta": {
        "source": "LinkedIn",
        "url": "https://..." // optional
      }
    }
    ```

- **Resume Generation (**`**resume_generation**`**):**
    
    ```json
    {
      "userProfile": { /* snapshot of profile */ },
      "job": { /* job data (raw + parsed) */ },
      "options": {
        "tone": "neutral",
        "length": "standard",
        "focus": "IC"
      }
    }
    ```
    
- **Cover Letter (**`**cover_letter_generation**`**):** similar to resume, plus `audience`.

### 3.2 Hash Function

- Use a stable, widely available hash like **SHA-256**.
- Implementation in Rust can use a crate such as `sha2`.
- Steps:
    1. Serialize the canonical input JSON with sorted keys (if needed) to a string.
    2. Compute SHA-256 digest of the string.
    3. Encode as hex and store as `input_hash`. 

Pseudocode in Rust:

```rust
use sha2::{Digest, Sha256};

pub fn compute_input_hash(json_payload: &serde_json::Value) -> Result<String, String> {
    let serialized = serde_json::to_string(json_payload)
        .map_err(|e| format!("Failed to serialize cache payload: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}
```

---

## 4. Rust Cache Abstraction

### 4.1 Core Structs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AiCacheKey {
    pub purpose: String,
    pub input_hash: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiCacheEntry {
    pub id: i64,
    pub purpose: String,
    pub input_hash: String,
    pub model_name: String,
    pub request_payload: serde_json::Value,
    pub response_payload: serde_json::Value,
    pub created_at: String,
    pub expires_at: Option<String>,
}
```

### 4.2 Cache API (Rust)

The cache should expose simple, reusable functions:

```rust
/// Attempts to load a cached AI response.
/// Returns Some(entry) if found and not expired, otherwise None.
pub fn ai_cache_get(
    conn: &rusqlite::Connection,
    purpose: &str,
    input_hash: &str,
    now_iso: &str,
) -> Result<Option<AiCacheEntry>, String>;

/// Inserts a new cache entry.
pub fn ai_cache_put(
    conn: &rusqlite::Connection,
    purpose: &str,
    input_hash: &str,
    model_name: &str,
    request_payload: &serde_json::Value,
    response_payload: &serde_json::Value,
    ttl_days: Option<i64>,
    now_iso: &str,
) -> Result<(), String>;

/// Optional: Clear cache by purpose or entirely.
pub fn ai_cache_clear_purpose(
    conn: &rusqlite::Connection,
    purpose: &str,
) -> Result<(), String>;

pub fn ai_cache_clear_all(conn: &rusqlite::Connection) -> Result<(), String>;
```

### 4.3 Expiration Logic

- `ai_cache_get` should return `None` if:
    - No row matches `(purpose, input_hash)`, OR
    - `expires_at` is non-null and `< now`.
- `ai_cache_put` should compute `expires_at` if `ttl_days` is set, e.g. `now + ttl_days`.

MVP TTL suggestions:
- Job parsing: 30–90 days (job descriptions rarely change).
- Resume & cover letter generation: 7–30 days (user profile may change).    

---

## 5. Integration with Features

### 5.1 Job Parsing (`job_parse`)

In `parse_job_with_ai` command:

1. Build canonical `request_payload` JSON:
    - Includes full `raw_description` and minimal metadata.
2. Compute `input_hash` via `compute_input_hash`.
3. Call `ai_cache_get(conn, "job_parse", &input_hash, now_iso)`.
    - If hit → parse `response_payload` into `ParsedJob` and skip API call.
4. If miss → call AI provider, get `ParsedJob` as JSON.
5. Call `ai_cache_put` with `purpose = "job_parse"`, TTL e.g. 90 days.

### 5.2 Resume Generation (`resume_generation`)

In `generate_resume_for_job`:
1. Build canonical JSON:
    - `userProfile`, `job`, `options` as specified in Resume spec.
2. Compute hash.
3. `ai_cache_get("resume_generation", hash)`.
4. On miss, call AI, then `ai_cache_put` with `purpose = "resume_generation"` and shorter TTL.

> **Important:** canonical JSON must be stable (e.g., avoid including volatile fields like `created_at`).

### 5.3 Cover Letter Generation (`cover_letter_generation`)

Same pattern as resume, with `purpose = "cover_letter_generation"`.

### 5.4 Future Features

- **Learning Plan Generator:**
    - `purpose = "learning_plan"`.
    - Input payload could include aggregated job requirements + current skill profile.
- **Interview Coaching:**
    - `purpose = "coaching"`.
    - Input payload might include application history and feedback.

The same abstraction supports these as long as they build canonical JSON + use `purpose` and `input_hash`.

---

## 6. Configuration & Environment

### 6.1 Model Name & Provider
- `model_name` should be provided by the AI provider abstraction.
- The cache should not care **which vendor** is used, only the name string.

### 6.2 TTL Defaults

Define a small config struct or constants in Rust:

```rust
pub const CACHE_TTL_JOB_PARSE_DAYS: i64 = 90;
pub const CACHE_TTL_RESUME_DAYS: i64 = 30;
pub const CACHE_TTL_COVER_LETTER_DAYS: i64 = 30;
```

Later this can be made user-configurable via settings.

---

## 7. Privacy & Security Considerations

- All cache entries are stored **locally** in SQLite.
- Cache entries may contain:
    - Personal data (resume content, job details, company names).
- Because this is a **self-hosted desktop app**, this is acceptable, but:
    - Do **not** log full `response_payload` to console in production builds.
    - Allow future option to clear cache (`ai_cache_clear_all`).

No additional encryption is required for MVP, but the DB is user-local.

---

## 8. Testing Scenarios

1. **Cache Miss → Put → Hit**
    - First call with given input should:
        - Miss the cache.
        - Call AI.
        - Insert into `ai_cache`.
    - Second call with same input should:
        - Hit the cache.
        - Skip AI call.
2. **Different Inputs → Different Hashes**
    - Slightly modify the job description or options.
    - Confirm `input_hash` is different and cache does not mistakenly reuse.
3. **Expiration**
    - Insert entry with `expires_at` in the past.
    - `ai_cache_get` should return `None`.
4. **Purpose Separation**
    - Insert entries with same `input_hash` but different `purpose`.
    - Confirm lookups are purpose-specific.
5. **Error Handling**
    - Simulate DB error (bad connection or locked DB).
    - Cache functions should return `Err` and callers should continue without cache (i.e., still call AI).
6. **Large Payloads**    
    - Ensure `response_payload` can handle large JSON documents (e.g., long resumes) without truncation.

---

## 9. Implementation Order (Recommended)

1. Add `ai_cache` table via SQLite migration.
2. Implement `compute_input_hash` helper.
3. Implement `ai_cache_get`, `ai_cache_put`, and optional clear functions.
4. Integrate cache into:
    - Job parsing (`parse_job_with_ai`).
    - Resume generation.
    - Cover letter generation.
5. Add basic logging for cache hits/misses (dev mode).
6. Add unit/integration tests for cache behavior.