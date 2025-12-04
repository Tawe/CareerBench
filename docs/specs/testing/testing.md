# Testing & AI Guardrails Specification

This spec defines **how we test CareerBench** and **how AI-generated code must behave** so we don’t silently break core flows.

Stack assumptions:
- Backend: **Rust + Tauri + SQLite**
- Frontend: **React + TypeScript**
- AI: external provider(s) behind an abstraction

The goals:
1. Make it easy to add features safely with AI assistance.
2. Ensure **AI-generated changes cannot quietly change contracts** or behavior.
3. Keep core user flows covered by tests (job intake → applications → artifacts).

---

## 1. Testing Goals

1. **Correctness** – Core flows (job intake, profile, resume/letter generation, pipeline, dashboard) must behave as expected.
2. **Stability of Contracts** – Tauri commands, data models, and AI schemas must not change shape without tests failing.
3. **Deterministic AI Integration** – AI boundary is tested with **mocks and fixtures**, not live calls.
4. **Fast Feedback** – Unit and integration tests must run quickly enough to be used frequently (e.g., pre-commit or CI).

---

## 2. Testing Layers Overview

We use four layers:
1. **Rust unit tests** – Functions, models, SQLite helpers, AI cache, AI boundary behavior.
2. **Rust integration tests** – Tauri commands, DB interactions with an on-disk or in-memory DB.
3. **TypeScript tests** – Hooks, pure utilities, and UI logic (React Testing Library where needed).
4. **Golden tests for AI** – Fixed input → expected structured JSON output; validate parsing and shaping code, not the LLM itself.

---

## 3. Rust Backend Testing

### 3.1 What must be unit-tested

**Required unit tests for any new or changed backend module:**
- **Data model helpers**
    - e.g., `GeneratedResume`, `GeneratedLetter` serialization/deserialization.
- **AI cache helpers**
    - `compute_input_hash`
    - `ai_cache_get`
    - `ai_cache_put`
- **Query/aggregation helpers**
    - Dashboard queries / data assembly
    - Application pipeline logic (status, funnel, dates)
- **Pure helpers**
    - `render_resume_to_text`
    - `render_letter_to_text`
    - Any “rules engine” style code (e.g., job aging/alerts).

> Rule: if a function has **logic branches** that aren’t pure DB/query wrappers, it must have unit tests.

---

### 3.2 Tauri Command Integration Tests

Each Tauri command that touches DB/AI must have at least one **integration-style test**:

Examples:
- `generate_resume_for_job`
- `generate_cover_letter_for_job`
- `parse_job_with_ai`
- `get_dashboard_data`
- `create_application`, `update_application_status`

**Patterns:**

- Use a **test-only SQLite DB file**; run migrations in test setup.
- Seed the DB with minimal fixtures (one profile, one job, one application).
- Use a **mock AI client** (see next section) so tests are deterministic.
- Assert:
    - Correct DB writes (new artifacts, events, etc.)
    - Correct returned types and fields
    - No panics on happy path

---

### 3.3 AI Client Mocking & Guardrails

We treat AI as an **external service** behind a small trait/abstraction.

`pub trait AiClient: Send + Sync {     fn generate_json(         &self,         prompt: &str,     ) -> Result<serde_json::Value, AiError>; }`

Two implementations:
1. **Real client** – Used in production.
2. **Mock client** – Used in tests.

The mock:
- Looks up fixed responses by **test key** or **purpose**.
- Returns structured JSON we control, or errors we want to simulate.

**Guardrail:**  
No test should call the real AI API. If a test hits the real client, that test is considered invalid.

---

## 4. AI Boundary & Schema Guardrails

The sharp edge of the system is where **free-form AI text** turns into **strongly typed structs**.  
This boundary must be heavily guarded.

### 4.1 Required validation steps

For every AI-backed feature:

1. **Define a Rust struct** for the expected result
    - e.g., `GeneratedResume`, `GeneratedLetter`, `ParsedJob`.
2. After receiving AI JSON:
    - **Validate with `serde_json::from_value`** into that struct.
    - If validation fails:
        - Return a **clean error** to the frontend.
        - Never insert malformed data into the DB.
3. Add **unit tests** that:
    - Deserialize **valid** example JSON successfully.
    - Fail gracefully on **invalid** or **partial** JSON.

---

### 4.2 Golden Files / Fixtures for AI Outputs

For each AI-powered feature, maintain **fixture files**:
- e.g., `tests/fixtures/resume_generation/basic_resume.json`
- e.g., `tests/fixtures/job_parsing/basic_job.json`

Tests should:
- Load the fixture JSON.
- Run it through the same deserialization and transformation pipeline as a real response.
- Assert:
    - Important fields are present and correctly mapped.
    - No panics if extra/unknown fields are present (ignore them).

**Guardrail:**  
If you change the shape of `GeneratedResume` or other AI structs, you must update both:
- The struct definition
- The fixture(s) + tests

Tests failing is your warning that you’re breaking an existing contract.

---

## 5. TypeScript / Frontend Testing

### 5.1 What to unit-test

Focus on **logic**, not visuals:

- Hooks:
    - `useDashboardData` (loading, error, success states)
    - `useApplications` / `useJobs`
- Pure utilities:
    - Filtering / sorting applications
    - Deriving labels from statuses
    - Client-side guards (e.g., disabling “Generate” without profile)

Example (Jest / Vitest):

`it("computes active applications count correctly", () => {   const apps = [     { id: 1, archived: false },     { id: 2, archived: true },   ];   expect(getActiveApplicationsCount(apps)).toBe(1); });`

---

### 5.2 Minimal UI Tests

Use React Testing Library sparingly:
- Make sure key pages render without crashing:
    - Dashboard
    - Jobs
    - Applications
    - Profile
- Test core behavior:
    - Status filter pills change what’s shown
    - Sheets open/close when buttons are clicked
    - “Unsaved changes” indicator appears when editing Profile

**Guardrail:**  
Front-end tests should fail if we:
- Rename critical Tauri commands
- Change types for core data (e.g., `DashboardData`, application statuses)
- Break basic flows (cannot open a job, cannot create an application)

---

## 6. AI-Specific Guardrail Rules for Cursor / AI-assisted Coding

These are **process rules** you can literally paste into your CONTRIBUTING.md or “Cursor instructions”:

1. **Never remove a test without replacing it**
    - If you delete a test, you must add a new one that covers the same behavior or explain in comments why it’s obsolete.
2. **When changing a Tauri command’s signature or return type:**
    - Update its **integration tests**.
    - Update any **frontend types** (`invoke<...>` generics).
    - Update **AI-facing spec** (if this command is referenced in an AI spec file).
3. **When changing AI structs (`GeneratedResume`, `GeneratedLetter`, `ParsedJob`):**
    - Update:
        - Rust structs
        - Fixtures (golden JSON)
        - Deserialization tests
        - Any TypeScript interfaces mirroring them
4. **For new AI-backed features:**
    - Add at least:
        - 1 mock-based integration test that exercises the full flow
        - 1 golden-file deserialization test
5. **AI Tools must not modify tests to match broken behavior**
    - If a test is failing, fix the **code** or update the **intent** explicitly, don’t just relax the test.
6. **Prompt discipline:**
    - Specs (like the ones we’ve written) live in `/docs/specs` or similar.
    - When asking Cursor to change logic, always reference the relevant spec and require tests:
        > “Update this function according to `CareerBench Local AI Caching Spec` and add/adjust unit tests to cover cache hit/miss behavior.”

---

## 7. Example Test Scenarios (Per Feature)

### 7.1 Job Parsing

- Parse a simple job description into `ParsedJob`
- Uses cache when same job text is passed again
- Gracefully handles malformed AI response (returns error, no DB write)

### 7.2 Resume Generation

- Generates artifact row with correct `job_id` / `application_id`
- Stores `GeneratedResume` JSON in `ai_payload`
- `render_resume_to_text` produces non-empty Markdown
- Respects AI cache when same profile+job+options are used

### 7.3 Dashboard

- DashboardData counts are correct for sample DB
- Archived applications don’t count as active
- Funnel numbers follow logical constraints (Interviewing ≤ Applied, Offer ≤ Interviewing)

### 7.4 Profile

- Updating profile fields persists correctly
- Missing required fields fail validation

---

## 8. CI / Automation Expectations (Optional but Recommended)

If you set up CI later:
- Run **Rust tests** & **TS tests** on every branch.
- Block merge on test failures.
- Optionally, add a simple linter rule:
    - Warn if files changed in `src-tauri` or `src` but no test files changed in the same PR (useful heuristic, not strict).