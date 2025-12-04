# Job Intake Specification

This spec is intended to guide Cursor AI when generating code for the **Job Intake** feature in the CareerBench Tauri desktop application.

---

## 1. Goals & Constraints

### 1.1 Primary Goals

- Allow users to **quickly capture job postings** from different sources (paste text, manual entry, later URL import).
- Store job data in **SQLite** in a structured way that supports:
    - AI parsing and comparison to the user profile.
    - Application tracking and funnel metrics.
    - Learning-plan generation based on skill patterns.
- Integrate with AI to **parse job descriptions** into structured fields (skills, responsibilities, seniority, tags).

### 1.2 Non-Goals (for MVP)
- No automatic web scraping from URLs in v1 (only stored for reference).
- No direct job board integration (LinkedIn API, etc.).
- No multi-user / multi-profile complexity; assume a single local user.

### 1.3 Technical Constraints
- Frontend: **Tauri + React + TypeScript**.
- Backend: **Rust** commands for DB, AI calls, and parsing.
- Storage: **SQLite** database.
- AI: pluggable provider (OpenAI, Gemini, local LLM) with optional caching in `ai_cache` table.

---

## 2. User Experience (UX) Overview

### 2.1 Navigation

- The app’s main navigation includes a **"Jobs"** section.
- Inside **Jobs**, we support:
    - **Job List View** – table or cards for all captured jobs.
    - **Job Detail View** – full job data, parsed insights, and actions.
    - **Add Job** flow – via button ("Add Job") that opens a modal or dedicated page.        

### 2.2 User Flows

#### Flow A: Add Job by Paste (Primary MVP Flow)

1. User clicks **"Add Job"**.
2. Modal or page appears with these fields:
    - Job Title (optional for paste, can infer via AI)
    - Company (optional)
    - Job Source (dropdown: LinkedIn, Company Site, Referral, Other)
    - Job URL (optional)
    - Location (optional)
    - **Job Description (large textarea for paste)**
3. User pastes the job description.
4. User clicks **"Save & Parse"**.
5. Backend:
    - Inserts a new row in `jobs` (status = draft; raw_description = pasted text).
    - Triggers AI parsing (if enabled in settings).
6. Frontend navigates to **Job Detail View**, which shows parsed fields once ready.

#### Flow B: Add Job Manually (No Description)
1. User clicks **"Add Job"**.
2. User fills in minimal fields (title, company) without a description.
3. User clicks **"Save"**.
4. Job is saved without AI parsing (can parse later if description is added).    

#### Flow C: View / Edit Job

- From Job List, user selects a job.
- Detail view shows:
    - Core info: title, company, source, URL, location.
    - Raw description (editable textarea).
    - Parsed AI summary:
        - Seniority
        - Responsibilities
        - Required skills
        - Nice-to-have skills
        - Tags / domain
    - Actions:
        - **"Re-parse with AI"** (if user edits description).
        - **"Create Application"** (if no application exists yet).     

---

## 3. Data Model: Job Intake

### 3.1 `jobs` Table

Fields:
- `id` (INTEGER PRIMARY KEY)
- `title` (TEXT, optional)
- `company` (TEXT, optional)
- `location` (TEXT, optional)
- `job_source` (TEXT, optional) – e.g. `"LinkedIn"`, `"Company Site"`, `"Referral"`, `"Other"`
- `posting_url` (TEXT, optional)
- `raw_description` (TEXT, optional) – user-pasted or manually entered job description
- `parsed_json` (TEXT, optional) – JSON string returned by AI parse (see 3.2)
- `seniority` (TEXT, optional) – AI-derived or user-edited (e.g. `"Junior"`, `"Senior"`, `"Lead"`)
- `domain_tags` (TEXT, optional) – comma-separated tags (e.g. `"Fintech, B2B SaaS"`)
- `is_active` (INTEGER, 0/1, default 1) – for archiving
- `date_added` (TEXT, ISO8601)
- `last_updated` (TEXT, ISO8601)

### 3.2 `parsed_json` Structure (Expected AI Output)

`parsed_json` should be a JSON object with at least:

```js
{
  "title_suggestion": "Senior Backend Engineer",
  "company_suggestion": "Acme Corp",
  "seniority": "Senior",
  "location": "Remote, North America",
  "summary": "High-level description of role.",
  "responsibilities": [
    "Build and maintain backend services in Node.js.",
    "Collaborate with product managers and designers.",
    "Improve system performance and scalability."
  ],
  "required_skills": [
    "Node.js",
    "TypeScript",
    "PostgreSQL",
    "AWS"
  ],
  "nice_to_have_skills": [
    "Kubernetes",
    "GraphQL"
  ],
  "domain_tags": [
    "SaaS",
    "B2B",
    "Fintech"
  ],
  "seniority_score": 0.8,
  "remote_friendly": true
}
```
- The Rust backend should parse this JSON into a Rust struct for internal use but store it as a TEXT column in SQLite.
- Frontend will treat it as optional; if missing, show an empty/placeholder parsed section.

### 3.3 `job_skills` Table (Optional, but Recommended)

We may denormalize some skill data from `parsed_json` into a `job_skills` table for easier querying.

Fields:
- `id` (INTEGER PRIMARY KEY)
- `job_id` (INTEGER, FK → `jobs.id`)
- `name` (TEXT, required) – e.g. "React", "System Design"
- `type` (TEXT, optional) – `"required"` or `"nice_to_have"`
- `source` (TEXT, optional) – `"ai_parsed"` or `"manual"`

For MVP, this can be implemented later; initially, we can rely on `parsed_json` only.

---

## 4. API / Tauri Command Design (Backend)

### 4.1 Commands

1. `create_job`
    - Input: basic job fields + raw description.
    - Behavior:
        - Insert into `jobs`.            
        - Return created job with `id`.

2. `update_job`
    - Input: `id` + updated fields (title, company, raw_description, etc.).
    - Behavior:
        - Update existing row.            
        - Optionally clear `parsed_json` to encourage re-parse when raw_description changes.

3. `get_job_list`
    - Input: optional filters (search term, active only, source, etc.).        
    - Output: list of jobs (id, title, company, location, date_added, seniority, domain_tags).

4. `get_job_detail`
    - Input: `job_id`.    
    - Output: full job object including `raw_description` and `parsed_json`.

5. `parse_job_with_ai`
    - Input:
        - `job_id`
        - (optional) explicit `raw_description` override
    - Behavior:
        - Fetch job from DB.
        - Build AI prompt using `raw_description`.
        - Check `ai_cache` for an existing parse with the same input hash.
        - If cache miss: call AI provider, store result in `ai_cache`, and update `jobs.parsed_json` and `jobs.seniority`, `jobs.domain_tags` if provided.            
    - Output: structured parsed JSON.

---

## 5. Frontend State & UI Behavior

### 5.1 Job List View

**State Shape Example:**

```js
interface JobListState {
  jobs: JobSummary[];
  isLoading: boolean;
  error?: string;
  filters: {
    search: string;
    source?: string;
    activeOnly: boolean;
  };
}

interface JobSummary {
  id: number;
  title: string;
  company: string;
  location?: string;
  seniority?: string;
  domainTags?: string[];
  dateAdded: string; // ISO string
}
```

**UI Elements:**
- Search input: filters by job title/company/location.
- Filters: active only toggle, source dropdown.
- Table columns:
    - Title
    - Company
    - Location
    - Seniority
    - Date Added
- "Add Job" button (primary CTA).    

### 5.2 Job Detail View

**State Shape Example:**

```js
interface JobDetailState {
  job: Job | null;
  isLoading: boolean;
  isSaving: boolean;
  isParsing: boolean;
  error?: string;
}

interface Job {
  id: number;
  title?: string;
  company?: string;
  location?: string;
  jobSource?: string;
  postingUrl?: string;
  rawDescription?: string;
  parsed?: ParsedJob | null;
  seniority?: string;
  domainTags?: string[];
  dateAdded: string;
  lastUpdated?: string;
}

interface ParsedJob {
  titleSuggestion?: string;
  companySuggestion?: string;
  seniority?: string;
  location?: string;
  summary?: string;
  responsibilities: string[];
  requiredSkills: string[];
  niceToHaveSkills: string[];
  domainTags: string[];
  seniorityScore?: number;
  remoteFriendly?: boolean;
}
```

**Layout:**

- Left: basic metadata form (title, company, source, URL, location).
- Middle / main:
    - **Raw Description** textarea (editable).
- Right / below:
    - **Parsed Insights** panel:
        - Title suggestion
        - Seniority
        - Responsibilities (bulleted)
        - Required / nice-to-have skills
        - Domain tags

**Actions:**
- **Save** – persists changes to fields, but doesn’t automatically re-parse.
- **Save & Re-Parse** – saves changes then calls `parse_job_with_ai`.
- **Re-Parse Only** – re-parses without changing other fields.

---

## 6. AI Prompting Strategy (High-Level)

### 6.1 Parsing Prompt Requirements

The AI prompt for job parsing should:
- Take the raw job description as input.
- Ask the model to output **strict JSON** in the `parsed_json` structure defined above.
- Infer:
    - Job title & company (if missing)
    - Seniority
    - Responsibilities
    - Required vs nice-to-have skills
    - Domain tags
    - Remote friendliness

The backend should:
- Validate returned JSON.
- If invalid, log error and optionally store raw AI response in `ai_cache` for debugging.

### 6.2 Caching

Before calling AI:
- Compute a hash of the raw job description.
- Look up in `ai_cache` for `purpose = "job_parse"` and `input_hash = hash`.
- If found, reuse cached `response_payload` as `parsed_json`.
- If not, call model and store result.

---

## 7. Validation & Error Handling

### 7.1 Frontend

- Required for adding a job via paste:
    - Raw description should not be empty.
- For manual entry:
    - Require at least `title` or `company`.
- Show inline validation messages.

### 7.2 Backend

- Reject create/update if required minimal fields are completely empty.
- On AI errors:
    - Return a clear error message to UI:
        - "AI parsing failed. You can still save the job and try again later."
    - Do **not** block basic saving of the job record.

### 7.3 UI Error Display

- Non-blocking banner or toast at the top of the Job Detail view for system-level errors.
- Inline error states for form validation issues.

---

## 8. Testing Scenarios (High-Level)

1. **Add Job via Paste**
    - Paste description, save & parse.        
    - Confirm job created and parsed fields populated.

2. **Add Job Manually**
    - Enter only title and company, save.        
    - Confirm job appears in list and detail.

3. **Edit Job and Re-Parse**
    - Update raw description.
    - Use "Save & Re-Parse".        
    - Confirm parsed fields updated accordingly.

4. **AI Failure Handling**
    - Simulate AI service error.        
    - Confirm job still saved and clear error message is shown.

5. **Job List Filtering**
    - Add multiple jobs with different companies and sources.        
    - Confirm search and filters work as expected.

6. **Cache Hit**
    - Parse job description once.
    - Trigger parse again with identical text.
    - Confirm that cache is used (e.g., via logs or reduced call count in dev mode).

---

## 9. Implementation Order (Recommended)

1. Create SQLite migration for `jobs` (and optionally `job_skills`).
2. Implement Rust Tauri commands:
    - `create_job`
    - `update_job`
    - `get_job_list`
    - `get_job_detail`
3. Build **Job List** UI in React.
4. Build **Add Job** modal/page with paste & manual flow.
5. Implement `parse_job_with_ai` command and wire it to detail view.
6. Add AI caching logic in `ai_cache` for `job_parse` purpose.    
7. Improve Job Detail UX with parsed insights panel and re-parse actions.
    
