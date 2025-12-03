# Basic Job Parsing Specification

This spec is intended to guide Cursor AI when generating code for the **Basic Job Parsing** feature in the CareerBench Tauri desktop application.

The goal of Basic Job Parsing is:

- Take a raw job description (pasted text) stored in the `jobs` table.
- Send it to an AI provider with a well-defined prompt.
- Receive **strict JSON** with structured fields.
- Persist that JSON into `jobs.parsed_json` and expose parsed data to the UI.    

This is **read-only AI enrichment** of job data (no resume generation or learning plans here).

---

## 1. Scope & Goals

### 1.1 In-Scope (MVP)

- Implement a Rust-side function / Tauri command that:
    - Accepts a `job_id`.
    - Loads `raw_description` from DB.
    - Builds a parsing request and calls the configured AI provider.
    - Validates and normalizes the AI JSON output.
    - Stores the resulting JSON string in `jobs.parsed_json`.
    - Optionally updates convenience columns like `jobs.seniority` and `jobs.domain_tags`.
- Implement a TypeScript function / hook to call this command from the Job Detail UI.

### 1.2 Out-of-Scope (for this spec)
- Job list UI design (covered in Job Intake spec).
- Resume generation, learning plans, or matching logic.
- Advanced analytics based on parsed data.

---

## 2. Data Model Integration

### 2.1 `jobs` Table (Relevant Columns)

Assume the `jobs` table has at least:
- `id` (INTEGER PRIMARY KEY)
- `title` (TEXT, optional)
- `company` (TEXT, optional)
- `location` (TEXT, optional)
- `job_source` (TEXT, optional)
- `posting_url` (TEXT, optional)
- `raw_description` (TEXT, required for parsing)
- `parsed_json` (TEXT, optional)
- `seniority` (TEXT, optional)
- `domain_tags` (TEXT, optional)
- `date_added` (TEXT, ISO8601)
- `last_updated` (TEXT, ISO8601)    

### 2.2 Parsed JSON Structure

The AI must return a JSON object with the following expected shape (keys are lowerCamelCase for TS ergonomics):

```json
{
  "titleSuggestion": "Senior Backend Engineer",
  "companySuggestion": "Acme Corp",
  "seniority": "Senior",
  "location": "Remote, North America",
  "summary": "High-level description of the role.",
  "responsibilities": [
    "Build and maintain backend services.",
    "Collaborate with product managers.",
    "Improve system performance and scalability."
  ],
  "requiredSkills": [
    "Node.js",
    "TypeScript",
    "PostgreSQL",
    "AWS"
  ],
  "niceToHaveSkills": [
    "Kubernetes",
    "GraphQL"
  ],
  "domainTags": [
    "SaaS",
    "Fintech",
    "B2B"
  ],
  "seniorityScore": 0.8,
  "remoteFriendly": true
}
```

Notes:
- All fields are optional except `responsibilities`, `requiredSkills`, `niceToHaveSkills`, and `domainTags`, which should default to empty arrays if not inferable.
- We store this entire JSON as a string in `jobs.parsed_json`.    
- The Rust backend should define a corresponding struct and handle missing keys gracefully.

### 2.3 Rust Struct Definition

Example Rust struct used for deserialization:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParsedJob {
    #[serde(default)]
    pub title_suggestion: Option<String>,

    #[serde(default)]
    pub company_suggestion: Option<String>,

    #[serde(default)]
    pub seniority: Option<String>,

    #[serde(default)]
    pub location: Option<String>,

    #[serde(default)]
    pub summary: Option<String>,

    #[serde(default)]
    pub responsibilities: Vec<String>,

    #[serde(default)]
    pub required_skills: Vec<String>,

    #[serde(default)]
    pub nice_to_have_skills: Vec<String>,

    #[serde(default)]
    pub domain_tags: Vec<String>,

    #[serde(default)]
    pub seniority_score: Option<f32>,

    #[serde(default)]
    pub remote_friendly: Option<bool>,
}
```

> **Note:** Use `#[serde(rename = "camelCaseName")]` if the AI returns camelCase keys and the struct uses snake_case.

---

## 3. Tauri Command: `parse_job_with_ai`

### 3.1 Command Signature (Rust)

```rust
#[tauri::command]
pub async fn parse_job_with_ai(job_id: i64) -> Result<ParsedJob, String> {
    // Implementation described below
}
```

- Input: `job_id` for the job record to parse.
- Output: `ParsedJob` struct on success, serialized back to JSON for the frontend.
- Error: string message describing what went wrong.

### 3.2 Command Steps

1. **Load Job From DB**
    - Query `jobs` table by `id`.
    - If not found → return error: `"Job not found"`.
    - If `raw_description` is null or empty → return error: `"Job description is empty; cannot parse."`.
2. **Build AI Request Payload**
    - Construct a text prompt or JSON payload that includes:
        - Purpose: job parsing.
        - Short instruction: output must be **valid JSON** with a specific schema.
        - The raw job description.
3. **Check AI Cache** (if `ai_cache` table is implemented)
    - Compute a hash of the raw description (e.g. SHA-256).
    - Look up `ai_cache` where `purpose = "job_parse"` and `input_hash = ?`.
    - If cache **hit**:
        - Deserialize the stored `response_payload` into `ParsedJob`.
        - Skip the network call.
4. **Call AI Provider** (cache miss)
    - Use a generic AI provider trait/interface (e.g. `AiProvider`) implemented elsewhere.
    - Send the prompt + raw description.
    - Receive a text response that _should_ be JSON.
5. **Validate & Deserialize JSON**
    - Attempt to parse the AI response into `ParsedJob` using `serde_json`.
    - If parsing fails:
        - Optionally try to extract JSON via simple heuristics (e.g., find the first `{` and last `}` and re-parse).
        - If still failing, return an error such as:
            - `"AI response was not valid JSON"`.
6. **Persist Results**
    - Serialize `ParsedJob` back to a JSON string.
    - Update `jobs` record:
        - `parsed_json = json_string`.
        - `seniority = parsed.seniority.unwrap_or(existing_seniority)`.
        - `domain_tags = parsed.domain_tags.join(", ")`.
        - `last_updated = now()` (ISO8601 timestamp).
7. **Update AI Cache** (if applicable)
    - Insert a row into `ai_cache` with:
        - `purpose = "job_parse"`.
        - `input_hash`.
        - `model_name`.
        - `request_payload`.
        - `response_payload` (JSON string).
8. **Return ParsedJob to Frontend**
    - Convert `ParsedJob` into JSON and return from the Tauri command.

---

## 4. AI Prompt Design (High-Level)

### 4.1 Prompt Requirements

The prompt should:
- Clearly state that the model must output **only JSON**, no extra commentary.
- Describe the keys and value types expected.
- Provide the raw job description as the main input.

### 4.2 Example Prompt (Pseudo-Template)

Backend can use something like:

```rust
You are a job description parser.

Given the job description below, extract structured information about the role.

Return ONLY valid JSON with the following structure (no additional text):
{
  "titleSuggestion": string | null,
  "companySuggestion": string | null,
  "seniority": string | null, // e.g. "Junior", "Mid", "Senior", "Lead", "Director"
  "location": string | null,
  "summary": string | null,
  "responsibilities": string[],
  "requiredSkills": string[],
  "niceToHaveSkills": string[],
  "domainTags": string[],
  "seniorityScore": number | null, // from 0.0 to 1.0, representing how senior the role is
  "remoteFriendly": boolean | null
}

Job description:
"""
{{RAW_JOB_DESCRIPTION}}
"""
```

> Cursor AI should transform this template into a concrete string builder in Rust, interpolating the raw job description.

---

## 5. Frontend Integration

### 5.1 TypeScript Types

Define corresponding types:

```js
export interface ParsedJob {
  titleSuggestion?: string | null;
  companySuggestion?: string | null;
  seniority?: string | null;
  location?: string | null;
  summary?: string | null;
  responsibilities: string[];
  requiredSkills: string[];
  niceToHaveSkills: string[];
  domainTags: string[];
  seniorityScore?: number | null;
  remoteFriendly?: boolean | null;
}
```

### 5.2 Calling the Tauri Command

Example React/TS usage:

```
import { invoke } from "@tauri-apps/api/tauri";

export async function parseJob(jobId: number): Promise<ParsedJob> {
  const result = await invoke<ParsedJob>("parse_job_with_ai", { jobId });
  return result;
}
```

- The Job Detail view can:
    - Show a **"Parse with AI"** or **"Re-parse"** button.
    - Set a local `isParsing` flag while waiting.
    - Update its state with the returned `ParsedJob`.

### 5.3 UI Behavior

- When parsing starts:
    - Disable the parse button.
    - Show a spinner or loading state.
- On success:
    - Show parsed responsibilities, skills, etc., in a dedicated panel.
- On error:
    - Show an error toast/banner with the returned message.
    - Do not clear existing parsed data unless explicitly requested.

---

## 6. Error Handling Strategy

### 6.1 Expected Error Types
- Job not found.
- Empty or missing `raw_description`.
- AI provider error (network failure, rate limit, invalid API key).
- AI returned non-JSON or malformed JSON.
- Database update failure.

### 6.2 Rust Error Mapping

- Implement a custom error enum (e.g. `JobParseError`) with variants:
    - `JobNotFound`
    - `EmptyDescription`
    - `AiError(String)`
    - `InvalidJson(String)`
    - `DbError(String)`
- Convert these into user-friendly error strings in the Tauri command return.

### 6.3 Frontend Messaging

Map error strings to user messages, e.g.:
- "Job not found" → "We couldn't find that job in your database."
- "Job description is empty; cannot parse." → "Please add a job description before parsing."
- "AI parsing failed. Please try again later." for all AI-related issues

---

## 7. Testing Scenarios

1. **Successful Parse**
    - Job has a non-empty description.
    - AI returns valid JSON.
    - `parsed_json` updated, frontend shows parsed data.
2. **Empty Description**
    - `raw_description` is empty.
    - Command returns error and does not update `parsed_json`.
3. **Malformed AI Response**
    - Simulate invalid JSON from AI.
    - Command returns `InvalidJson` error and does not update DB.
4. **Cache Hit**
    - Parse a job once (cache filled).
    - Call parse again with identical description.
    - Ensure no second network call (via logs in dev mode) and same result is returned.
5. **DB Failure**
    - Simulate DB error on update.    
    - Ensure error is returned and frontend displays message without crashing.

---

## 8. Implementation Order (Recommended)

1. Define `ParsedJob` struct in Rust and TypeScript.
2. Implement the `parse_job_with_ai` Tauri command with:
    - DB fetch.
    - AI cache lookup.
    - AI call.
    - JSON validation & DB update.
3. Add AI provider abstraction if not already present.
4. Wire up frontend function `parseJob(jobId: number)`.
5. Add a **"Parse with AI"** button in Job Detail UI and handle loading/error states.