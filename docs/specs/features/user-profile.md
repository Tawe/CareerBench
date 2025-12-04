# CareerBench – User Profile Entry Specification

This spec is intended to guide Cursor AI when generating code for the **User Profile Entry** feature in the CareerBench Tauri desktop application.

---

## 1. Goals & Constraints

### 1.1 Primary Goals

- Provide a **single, clear place** for the user to define and maintain their professional profile.
- Store the profile data in **SQLite** for offline use and to feed AI workflows (resume generation, learning plans, job fit scoring).
- Make profile entry feel **lightweight and progressive**: users can start with minimal info and refine over time.    

### 1.2 Non-Goals (for MVP)
- No LinkedIn or CV file import in v1 (we can add later).
- No multi-profile support in v1; assume a **single user profile**.
- No deep analytics here; profile is a data entry & editing surface.

### 1.3 Technical Constraints

- Frontend: **Tauri + React + TypeScript**.
- Backend: **Rust** commands for DB access and AI operations.
- Storage: **SQLite** database with migrations.

---

## 2. User Experience (UX) Overview

### 2.1 Navigation

- The app’s main navigation includes a **"Profile"** section.
- Selecting **Profile** opens a multi-section layout:
    - **Overview** (read-only summary + inline edit buttons)
    - **Experience**
    - **Skills**
    - **Education & Certifications**
    - **Portfolio**

### 2.2 Layout

- Left side: vertical tab/section list (Overview, Experience, Skills, Education, Portfolio).
- Main panel: content for the selected section.
- Top-right: **"Save"** / **"Save & Close"** (if needed) plus optional **"AI Assist"** actions later.

### 2.3 Interaction Model

- Changes are **not auto-saved**; user must click **"Save"**.
- If the user has unsaved changes and switches sections or navigates away, show a confirmation dialog:
    - “You have unsaved changes. Save before leaving?” → **Save / Discard / Cancel**. 

---

## 3. Data Model: User Profile (MVP)

These are the core tables/fields that the profile UI will read/write.

### 3.1 `user_profile` Table (Assume single row; `id = 1`)

Fields:
- `id` (INTEGER PRIMARY KEY)
- `full_name` (TEXT, required)
- `headline` (TEXT, optional) – e.g. "Senior Full-Stack Engineer | React, Node, AWS"
- `location` (TEXT, optional)
- `summary` (TEXT, optional) – 2–6 paragraph professional summary
- `current_role_title` (TEXT, optional)
- `current_company` (TEXT, optional)
- `seniority` (TEXT, optional, enum-ish string – e.g. `"Junior" | "Mid" | "Senior" | "Lead" | "Manager" | "Director" | "VP" | "C-Level"`)
- `open_to_roles` (TEXT, optional; comma-separated for MVP)
- `created_at` (TEXT, ISO8601 datetime)
- `updated_at` (TEXT, ISO8601 datetime)

### 3.2 `experience` Table

Fields:
- `id` (INTEGER PRIMARY KEY)
- `user_profile_id` (INTEGER, FK → `user_profile.id`)
- `company` (TEXT, required)
- `title` (TEXT, required)
- `location` (TEXT, optional)
- `start_date` (TEXT, optional, stored as `YYYY-MM-DD` or `YYYY-MM`)
- `end_date` (TEXT, optional, same format; null or empty for "Present")
- `is_current` (INTEGER, 0/1)
- `description` (TEXT, optional – freeform role description)
- `achievements` (TEXT, optional – newline-separated bullets)
- `tech_stack` (TEXT, optional – comma-separated tech tags)
- `created_at` (TEXT)
- `updated_at` (TEXT)

### 3.3 `skills` Table

Fields:
- `id` (INTEGER PRIMARY KEY)
- `user_profile_id` (INTEGER, FK)
- `name` (TEXT, required) – e.g. "React", "Leadership"
- `category` (TEXT, optional) – e.g. `"Technical" | "Soft" | "Domain" | "Tool"`
- `self_rating` (INTEGER, optional, 1–5)
- `priority` (TEXT, optional) – e.g. `"Core" | "Supporting" | "Learning"`
- `years_experience` (REAL, optional)
- `notes` (TEXT, optional)

### 3.4 `education` Table

Fields:
- `id` (INTEGER PRIMARY KEY)
- `user_profile_id` (INTEGER, FK)
- `institution` (TEXT, required)
- `degree` (TEXT, optional)
- `field_of_study` (TEXT, optional)
- `start_date` (TEXT, optional)
- `end_date` (TEXT, optional)
- `grade` (TEXT, optional)
- `description` (TEXT, optional)

### 3.5 `certifications` Table (Optional for MVP, but nice)

Fields:
- `id` (INTEGER PRIMARY KEY)
- `user_profile_id` (INTEGER, FK)
- `name` (TEXT, required)
- `issuing_organization` (TEXT, optional)
- `issue_date` (TEXT, optional)
- `expiration_date` (TEXT, optional)
- `credential_id` (TEXT, optional)
- `credential_url` (TEXT, optional)

### 3.6 `portfolio_items` Table

Fields:
- `id` (INTEGER PRIMARY KEY)
- `user_profile_id` (INTEGER, FK)
- `title` (TEXT, required)
- `url` (TEXT, optional)
- `description` (TEXT, optional)
- `role` (TEXT, optional – what the user did)
- `tech_stack` (TEXT, optional)
- `highlighted` (INTEGER, 0/1, optional – for AI emphasis)

---

## 4. Detailed UX Per Section

### 4.1 Overview Section

**Purpose:** Quick, high-level profile editing.

Fields to show:
- Full Name (text input)
- Headline (text input)
- Location (text input)
- Current Role Title (text input)
- Current Company (text input)
- Seniority (dropdown)
- Summary (multi-line text area)
- Open to Roles (tag input or comma-separated text input)

**Behavior:**
- Provide an initial empty-state if no profile exists:
    - "Let’s set up your profile so CareerBench can tailor resumes and advice to you."
- Prefill with data if row already exists.
- Bottom of panel: **[Save]** button, disabled if no changes.

**Validation (MVP):**
- `full_name` is required.
- All other fields optional.
- On error: show inline message under field (e.g., "Name is required").

---

### 4.2 Experience Section

**Layout:**

- List of existing roles as cards, each showing:
    - `title` at `company` (dates)
    - Short snippet of description or first achievement bullet.
- Each card has **Edit** and **Delete** options.
- A primary button: **"Add Experience"** opens a modal or in-panel form.

**Experience Form Fields:**
- Company _(required)_
- Job Title _(required)_
- Location _(optional)_
- Start Date _(optional)_
- End Date _(optional)_
- Checkbox: **"This is my current role"**
- Description (multi-line)
- Achievements (multi-line; user can enter bullet-style text)
- Tech Stack (comma-separated tags)    

**Validation:**

- Company and Job Title are required.
- If `is_current` is true, you can allow end_date to be null.
- Optional: ensure end_date is not earlier than start_date if both are provided.

---

### 4.3 Skills Section

**Layout:**

- Two-column layout:
    - Left: existing skills in a table or pill list.
    - Right: "Add Skill" form or modal.

**Displayed Columns:**
- Skill name
- Category
- Self-rating
- Priority

**Skill Form Fields:**
- Name _(required)_
- Category _(dropdown: Technical, Soft, Domain, Tool, Other)_
- Self-rating _(1–5 slider or select)_
- Priority _(dropdown: Core, Supporting, Learning)_
- Years of Experience _(optional numeric)_
- Notes _(optional)_

**Behavior:**
- Quick-add: just entering a name creates a skill with default category = Technical, rating = 3.
- Editing existing skills should reuse the same form.    

---

### 4.4 Education & Certifications Section

Combine into a tabbed interface inside the main section:
- Tabs: **Education | Certifications**

**Education UI:**
- List of entries with institution, degree, dates.
- "Add Education" button with the fields listed in 3.4.

**Certifications UI:**
- List of certs with name, org, issue date.    
- "Add Certification" button with fields from 3.5.

Validation is minimal (only institution/cert name required).

---

### 4.5 Portfolio Section

**Layout:**
- Card grid or vertical list of portfolio items.
- Each card: title, URL (if present), 1–2 line description, role.

**Form Fields:**
- Title _(required)_
- URL _(optional)_
- Description _(optional)_
- Role _(optional)_
- Tech Stack _(optional)_    
- Checkbox: **"Highlight this project"** (sets `highlighted = 1`)

**Purpose for AI:**
- Highlighted items will be emphasized when generating resumes and cover letters.

---

## 5. State Management & Data Flow

### 5.1 Frontend State

Use a global or page-level state structure such as:

```js
interface UserProfileState {
  profile: UserProfile | null;
  experience: Experience[];
  skills: Skill[];
  education: Education[];
  certifications: Certification[];
  portfolio: PortfolioItem[];
  isDirty: boolean;
  isLoading: boolean;
  error?: string;
}
```
- `isDirty` toggles when any field changes.
- `isLoading` shows while fetching/saving from Tauri/Rust.

### 5.2 Loading Flow
1. On Profile route load, call Tauri command e.g. `get_user_profile_data`.
2. Backend retrieves:
    - `user_profile` row (id = 1 or first row)
    - all related rows from experience/skills/education/certifications/portfolio.
3. Return as a composite JSON payload.
4. Initialize state from this payload.

### 5.3 Saving Flow

- When user clicks **Save** in any section:
    1. Collect `UserProfileState` (or minimal delta for that section).
    2. Call Tauri command e.g. `save_user_profile_data`.
    3. Backend performs upserts:
        - If `user_profile.id` exists → update, else insert new row.
        - For lists (experience, skills, etc.), handle create/update/delete based on `id` being present.
    4. Return updated composite payload.
    5. Frontend updates state and sets `isDirty = false`.

**Note:** For simplicity in MVP, save the whole section or whole profile at once; optimize later if needed.

---

## 6. Validation & Error Handling

### 6.1 Frontend Validation
- Perform basic required-field validation before sending to backend.
- Show inline messages near fields.
- Do not prevent saving if optional fields are empty.

### 6.2 Backend Validation
- Ensure required columns (e.g. `full_name` in user_profile, `company` & `title` in experience, etc.) are not null before insert/update.
- If validation fails, return an error with a human-readable message.
    

### 6.3 Error Display
- If a Tauri command returns an error, show a non-blocking toast or error banner at the top of the section.
- Optionally highlight the problematic field if known.

---

## 7. AI Integration Points (Future Enhancements, Not Required for MVP)

These are hooks to keep in mind when structuring the code.

### 7.1 AI-Assisted Summary
- Button: **"AI: Improve my summary"**
- Prompt uses:
    - Current summary
    - Experience list
    - Skills
- AI returns an improved professional summary.
- User can preview and accept/reject.

### 7.2 Skill Extraction
- Button: **"AI: Suggest skills"**
- Uses:
    - Experience descriptions
    - Portfolio descriptions
- Returns a list of suggested skills; user can accept individually.

### 7.3 Highlighted Portfolio Coaching
- Button: **"AI: Rewrite this project description for resume"**    
- AI returns a bullet-style description emphasizing outcomes.

No AI calls should be automatic in MVP; always user-triggered with explicit buttons.

---

## 8. Testing Scenarios (High-Level)

1. **First-Time Setup**
    - No profile data exists.
    - User enters minimum info (name) and saves.        
    - Profile row is created, and data persists after app restart.

2. **Editing Existing Profile**
    - Load existing profile, modify summary & skills, save.        
    - Ensure updates persisted correctly.

3. **Add / Edit / Delete Experience**
    - Add a new job, edit it, then delete it.        
    - Confirm DB state reflects changes.

4. **Skills Quick-Add**
    - Add skill with only name.        
    - Confirm defaults are applied (category, rating, priority).

5. **Unsaved Changes Warning**
    - Edit a field, try to navigate away.        
    - Confirm confirmation dialog appears.

6. **Error Case: DB Failure**
    - Simulate backend error.        
    - Ensure error message is shown and state is not incorrectly cleared.

---

## 9. Implementation Order (Recommended)

1. Create SQLite migrations for:
    - `user_profile`
    - `experience`
    - `skills`
    - `education`
    - `certifications`        
    - `portfolio_items`

2. Implement Rust Tauri commands:    
    - `get_user_profile_data` → returns composite JSON
    - `save_user_profile_data` → upserts all sections

3. Build React UI for **Overview** section.
4. Implement **Experience** section (list + add/edit form).
5. Implement **Skills** section.
6. Implement **Education & Certifications**.
7. Implement **Portfolio**.