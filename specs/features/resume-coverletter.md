# Resume & Cover Letter Generator Specification

This spec is intended to guide Cursor AI when generating code for the **Resume & Cover Letter Generator** feature in the CareerBench Tauri desktop application.

The feature takes:

- The **user’s profile** (experience, skills, education, portfolio).
- A specific **job** (and optionally its parsed AI structure).
- Optional **user preferences** (tone, length, style).

…and produces:
- A **tailored resume** for that job.
- An optional **cover letter** or **outreach email**.
- Stores these as **artifacts** linked to the application.

---

## 1. Goals & Scope

### 1.1 Primary Goals

- Generate **job-specific resumes** using AI, based on the user’s profile and the target job.
- Optionally generate **cover letters** or **cold outreach emails**.
- Store generated artifacts in SQLite so that users can view, edit, and re-use them.
- Support **caching** to avoid repeated AI calls for identical inputs.

### 1.2 In-Scope (MVP)

- Data model changes (if needed) to store generated resumes and letters.
- Tauri commands to:
    - Prepare an AI prompt using profile + job.
    - Call AI provider and get back structured resume/letter content.
    - Store the result as artifacts.
- A UI flow for:
    - Selecting a job and generation mode.
    - Previewing the generated resume/letter.
    - Saving and optionally editing in-place.

### 1.3 Out-of-Scope (for this spec)
- PDF/Word export (can be separate feature).
- Rich text editing toolbar details.
- Automatic sending of emails.

---

## 2. Data Model

The generator will lean on existing tables and introduce or clarify the **artifacts** model.

### 2.1 `artifacts` Table

Each row represents a generated or manually created artifact (resume, cover letter, note, etc.) attached to an application.

Fields:
- `id` (INTEGER PRIMARY KEY)
- `application_id` (INTEGER, FK → `applications.id`, optional but recommended)
    - For job-specific artifacts. For generic/base resumes, this may be null.
- `job_id` (INTEGER, FK → `jobs.id`, optional)
    - Useful when an artifact is tied to a job but created before application exists.
- `type` (TEXT, required)
    - e.g. `"Resume"`, `"CoverLetter"`, `"OutreachEmail"`, `"Notes"`, etc.
- `title` (TEXT, required)
    - Human-friendly label, e.g. "Resume – Senior Backend Engineer at Acme".
- `content` (TEXT, optional)
    - Main text content (Markdown/plain).
- `format` (TEXT, optional)
    - e.g. `"markdown"`, `"plaintext"`, `"html"`.
- `ai_payload` (TEXT, optional)
    - JSON string of the parsed AI response (structured sections, bullets, etc.).
- `ai_model` (TEXT, optional)
    - e.g. `"gpt-4.1"`, `"gemini-2.5"`.
- `source` (TEXT, optional)
    - e.g. `"ai_generated"`, `"manual"`, `"imported"`.
- `version` (INTEGER, optional)
    - For future versioning.
- `created_at` (TEXT, ISO8601)
- `updated_at` (TEXT, ISO8601)

### 2.2 `ai_cache` Table (Reused)

The generator will reuse `ai_cache` for:
- `purpose = "resume_generation"`
- `purpose = "cover_letter_generation"`

Fields (assumed):
- `id` (INTEGER PRIMARY KEY)
- `purpose` (TEXT)
- `input_hash` (TEXT)
- `model_name` (TEXT)
- `request_payload` (TEXT)
- `response_payload` (TEXT)
- `created_at` (TEXT)

We compute `input_hash` based on a canonical JSON of:
- User profile snapshot.
- Job description / parsed job object.
- Generation options (e.g. tone, style, length, role focus).    

---

## 3. AI Output Structure

We want AI to return a structured object that can be:
- Rendered as a text resume.
- Potentially rendered as different templates later.    

### 3.1 `GeneratedResume` JSON Structure

Expected JSON from AI:

```rust
{
  "summary": "Short professional summary tailored to this job.",
  "headline": "Senior Backend Engineer – Node.js, TypeScript, Distributed Systems",
  "sections": [
    {
      "title": "Experience",
      "items": [
        {
          "heading": "Senior Backend Engineer – Acme Corp",
          "subheading": "Jan 2020 – Present | Remote",
          "bullets": [
            "Designed and implemented scalable microservices handling 10M+ daily requests.",
            "Led a team of 4 engineers and improved deployment frequency by 3x.",
            "Optimized database queries, reducing latency by 40%."
          ]
        }
      ]
    },
    {
      "title": "Skills",
      "items": [
        {
          "heading": "Core Skills",
          "bullets": [
            "Node.js, TypeScript, PostgreSQL, AWS",
            "System Design, Distributed Systems",
            "Technical Leadership, Mentoring"
          ]
        }
      ]
    }
  ],
  "highlights": [
    "Tailored emphasis on backend and distributed systems",
    "Relevant leadership experience",
    "Tech stack alignment with job description"
  ]
}
```

Notes:
- `sections` is flexible; templates can choose how to render.
- `highlights` can be presented in the UI as "What changed for this job".    

### 3.2 `GeneratedLetter` JSON Structure

Expected JSON for a cover letter or outreach email:

```rust
{
  "subject": "Application for Senior Backend Engineer – John Doe",
  "greeting": "Dear Hiring Manager,",
  "bodyParagraphs": [
    "I am excited to apply for the Senior Backend Engineer role at Acme Corp.",
    "For the past 5 years, I have led backend teams building scalable systems in Node.js and TypeScript.",
    "I am particularly drawn to Acme's focus on reliability and developer experience..."
  ],
  "closing": "Thank you for your time and consideration.",
  "signature": "Sincerely,\nJohn Doe"
}
```

The frontend will join `bodyParagraphs` with blank lines to form the full letter.

---

## 4. Tauri Command Design (Backend)

### 4.1 Commands

1. `generate_resume_for_job`
    - Input:
        - `job_id` (required)
        - `application_id` (optional)
        - `options` (optional JSON):
            - `tone` (e.g. `"neutral"`, `"confident"`, `"friendly"`)
            - `length` (e.g. `"concise"`, `"standard"`, `"detailed"`)
            - `focus` (e.g. `"IC"`, `"Leadership"`, `"Hybrid"`) 
    - Behavior:
        1. Load:
            - User profile and related entities (experience, skills, education, portfolio).
            - Job by `job_id` (including `raw_description` and `parsed_json` if available).
        2. Construct a canonical JSON `request_payload` containing:
            - `user_profile_snapshot`
            - `job_data`
            - `options`
        3. Compute `input_hash` from this JSON.
        4. Check `ai_cache` for `purpose = "resume_generation"` and `input_hash`.
        5. If cache hit → deserialize `response_payload` into `GeneratedResume`.
        6. If cache miss → call AI provider with an appropriate prompt and parse JSON.
        7. Store the response in `ai_cache`.
        8. Create an `artifacts` row with:
            - `type = "Resume"`
            - `job_id` and `application_id` (if provided)
            - `title` derived from job + role (e.g. `"Resume – {job_title} @ {company}"`)
            - `content` as rendered Markdown/plaintext version of resume.
            - `ai_payload` as JSON string.
            - `ai_model` = model used.
        9. Return the `GeneratedResume` + artifact metadata to the frontend.
2. `generate_cover_letter_for_job`
    - Input:
        - `job_id` (required)
        - `application_id` (optional)
        - `options` (optional JSON):
            - `tone`
            - `length`
            - `audience` (e.g. `"hiring_manager"`, `"recruiter"`) 
    - Behavior:
        - Same cache pattern as resume generation with `purpose = "cover_letter_generation"`.
        - Use AI to produce `GeneratedLetter` JSON.
        - Save as `artifacts` row with `type = "CoverLetter"` and rendered `content`.
        - Return `GeneratedLetter` + artifact metadata.
3. `render_resume_to_text`
    - Input:
        - `GeneratedResume` JSON.
    - Output:
        - String containing Markdown/plaintext resume.  
    - This can be implemented in Rust or TS; for now we assume a Rust helper used by Tauri command.
4. `render_letter_to_text`
    - Input: `GeneratedLetter` JSON.
    - Output: String letter.

---

## 5. AI Prompt Design (High-Level)

### 5.1 Resume Prompt Requirements

The prompt should:
- Provide the **user profile** in structured form (summary, experience, skills, portfolio).
- Provide the **job description** and parsed job data.
- Ask the model to:
    - Select the **most relevant experience and achievements**.
    - Prioritize skills and accomplishments that map to job requirements.
    - Adjust tone and focus per options.
    - Output **only valid JSON** in the `GeneratedResume` schema.        

Pseudo-template:

```rust
You are an expert resume writer.

You will receive:
1) A candidate profile (experience, skills, education, portfolio).
2) A job description and parsed job requirements.
3) Options for tone, length, and focus.

Your task is to create a resume tailored to this job.

Rules:
- Emphasize experience and skills most relevant to the job.
- Use clear, metric-driven bullet points when possible.
- Keep the tone professional and aligned to the tone option.
- If length = "concise", limit to the most recent and relevant roles.

Return ONLY valid JSON with this structure:
{
  "summary": string,
  "headline": string,
  "sections": [
    {
      "title": string,
      "items": [
        {
          "heading": string,
          "subheading": string,
          "bullets": string[]
        }
      ]
    }
  ],
  "highlights": string[]
}

Candidate profile:
{{USER_PROFILE_JSON}}

Job data:
{{JOB_DATA_JSON}}

Options:
{{OPTIONS_JSON}}
```

### 5.2 Cover Letter Prompt Requirements

Similar approach but output in `GeneratedLetter` format.

Pseudo-template:

```rust
You are an expert cover letter writer.

Given:
1) Candidate profile.
2) Job description and parsed job requirements.
3) Options for tone, length, and audience.

Write a tailored cover letter for this specific role.

Return ONLY valid JSON with this structure:
{
  "subject": string,
  "greeting": string,
  "bodyParagraphs": string[],
  "closing": string,
  "signature": string
}

Candidate profile:
{{USER_PROFILE_JSON}}

Job data:
{{JOB_DATA_JSON}}

Options:
{{OPTIONS_JSON}}
```

---

## 6. Frontend UX & State

### 6.1 Entry Points

- **From Job Detail View:**
    - Button: **"Generate Resume for this Job"**.
    - Button: **"Generate Cover Letter"**.
    - Optional: Combined flow with checkboxes (Resume, Cover Letter).
- **From Application Detail View:**
    - If application exists for job, show same buttons but associate artifact with `application_id`.

### 6.2 Generation Modal / Panel

Step 1: **Choose Options**
- Tone dropdown: Neutral / Confident / Friendly.
- Length dropdown: Concise / Standard / Detailed.
- Focus dropdown: Individual Contributor / Leadership / Hybrid.
- For letters: Audience dropdown (Hiring Manager / Recruiter).

Step 2: **Generate**
- Button: **Generate** triggers Tauri command.
- Show loading state with status text: "Generating tailored resume...".

Step 3: **Preview**
- Display generated resume/letter in a scrollable panel.
- Option to switch between:
    - Structured view (sections, bullets).
    - Raw text view (what will be stored in `content`).

Actions:
- **Save** – saves artifact if not already saved (though backend will usually save upon generation).
- **Edit** – allow inline text editing in the UI (edits update `content` but not `ai_payload`).
- **Discard** – close without attaching to application (optional; may still be in DB as artifact flagged as `archived` or left unsaved).

### 6.3 Linking to Application

If called from Application Detail:
- Generated artifact should automatically attach to that `application_id`.
- Application Detail view should show a list of attached artifacts (with quick open buttons).
If called from Job Detail (no application yet):
- Attach artifact to `job_id` only.
- When an application is later created for that job, optionally prompt: "Attach existing resume/letter to this application?".

---

## 7. Error Handling & Edge Cases

### 7.1 Backend Errors
- AI provider error (network, rate limit, invalid key).
- Invalid JSON returned by AI.
- DB insert/update failure.

Strategy:
- If AI fails:
    - Return error and **do not** create an artifact.
    - Frontend shows message: "Generation failed. Please try again or adjust your options.".
- If JSON parsing fails:
    - Log raw AI response for debugging in dev.        
    - Return user-friendly error.


### 7.2 Frontend Behavior

- Disable Generate button while request in progress.
- Allow user to retry.
- Preserve option selections across retries.

---

## 8. Testing Scenarios (High-Level)

1. **Successful Resume Generation**
    - User selects job and options.
    - Resume generated and artifact created.
    - Artifact appears in application detail.
2. **Successful Cover Letter Generation**
    - Same as above but for letters.
3. **Cache Hit**
    - Generate resume for same job with same options twice.
    - Confirm second call uses cached result (via dev logs / no second API call).
4. **AI Error**
    - Simulate provider error.
    - UI shows error, no artifact created.
5. **Manual Edit After Generation**
    - User edits generated resume content.
    - Confirm edits persist after closing/reopening artifact.
6. **Job Without Parsed JSON**    
    - Generator should still work using raw description; prompt must handle missing parsed job data gracefully.

---

## 9. Implementation Order (Recommended)

1. Ensure `user_profile`, `experience`, `skills`, `education`, `portfolio_items`, `jobs`, and `applications` tables exist.
2. Implement/extend `artifacts` table as described.
3. Implement AI provider abstraction (if not already built) and `ai_cache` integration.
4. Implement Tauri commands:
    - `generate_resume_for_job`
    - `generate_cover_letter_for_job`
    - `render_resume_to_text`
    - `render_letter_to_text`
5. Implement frontend types and hooks for generation.
6. Build generation modal/panel and integrate into Job Detail and Application Detail views.
    
