# Application Pipeline Specification

This spec is intended to guide Cursor AI when generating code for the **Application Pipeline** feature in the CareerBench Tauri desktop application.

The Application Pipeline connects **jobs** to the user’s actual **applications**, tracks their progress through stages (Saved → Applied → Interviewing → Offer → Rejected/Ghosted), and powers the calendar, dashboard, and alerts.

---

## 1. Goals & Scope

### 1.1 Primary Goals

- Represent each user application as a record linked to a job.
- Track the **status**, **key dates**, and **events** (interviews, follow-ups, offers, rejections).
- Provide a simple **pipeline UI** (e.g., kanban-style or table with status filters).
- Enable other modules (calendar, alerts, dashboard, AI coaching) to read this data.

### 1.2 In-Scope (MVP)

- Data model for `applications` and `application_events`.
- Tauri commands to create, update, and list applications.
- UI to:
    - Create an application for a job.
    - View & edit application details.
    - Change status and key dates.
    - Log events (interviews, follow-ups, rejections, etc.).
- Basic pipeline view (by status) and per-application timeline.

### 1.3 Out-of-Scope (for this spec)
- Calendar UI and event sync (covered in Calendar spec later).
- AI coaching prompts based on application history.
- Email integration and auto-import of events.

---

## 2. Data Model

### 2.1 `applications` Table

Each row represents a **user’s application** for a specific job.

Fields:
- `id` (INTEGER PRIMARY KEY)
- `job_id` (INTEGER, FK → `jobs.id`, required)
- `status` (TEXT, required)
    - Enum-ish string: `"Saved" | "Draft" | "Applied" | "Interviewing" | "Offer" | "Rejected" | "Ghosted" | "Withdrawn"`
- `channel` (TEXT, optional)
    - e.g. `"LinkedIn"`, `"Company Site"`, `"Referral"`, `"Recruiter"`, `"Other"`
- `priority` (TEXT, optional)
    - e.g. `"Low" | "Medium" | "High" | "Dream"`
- `date_saved` (TEXT, ISO8601, required)
    - When the application record was created (not necessarily applied).
- `date_applied` (TEXT, ISO8601, optional)
- `last_activity_date` (TEXT, ISO8601, optional)
- `next_action_date` (TEXT, ISO8601, optional)
    - Drives reminders / alerts.
- `next_action_note` (TEXT, optional)
    - Short note like “Follow up with recruiter” or “Prepare for system design interview”.
- `notes_summary` (TEXT, optional)
    - AI or user-maintained summary of overall application.
- `contact_name` (TEXT, optional)
- `contact_email` (TEXT, optional)
- `contact_linkedin` (TEXT, optional)
- `location_override` (TEXT, optional)
    - If the application has a specific location differing from job’s default (e.g., one office vs general posting).
- `offer_compensation` (TEXT, optional)
    - Freeform for now (can be structured later).
- `archived` (INTEGER, 0/1, default 0)
- `created_at` (TEXT, ISO8601)
- `updated_at` (TEXT, ISO8601)

**Rules/Assumptions:**

- A `job` may have 0 or 1 _active_ application. (We can allow multiple in future if needed, but MVP assumes one.)  
- `status` must always be set; default for a freshly created application is usually `"Saved"` or `"Draft"`.

### 2.2 `application_events` Table

Each row represents a **timestamped event** related to an application.

Fields:
- `id` (INTEGER PRIMARY KEY)
- `application_id` (INTEGER, FK → `applications.id`, required)
- `event_type` (TEXT, required)
    - Suggested values:
        - `"ApplicationCreated"`
        - `"StatusChanged"`
        - `"Applied"`
        - `"InterviewScheduled"`
        - `"InterviewCompleted"`
        - `"FollowUpSent"`
        - `"FeedbackReceived"`
        - `"OfferReceived"`
        - `"OfferAccepted"`
        - `"OfferDeclined"`
        - `"Rejected"`
        - `"MarkedGhosted"`
    - Not enforced at DB level in MVP; treat as free text with conventions.
- `event_date` (TEXT, ISO8601, required)
- `from_status` (TEXT, optional)
- `to_status` (TEXT, optional)
- `title` (TEXT, optional)
    - Brief label, e.g. “Recruiter Phone Screen”, “System Design Interview”.
- `details` (TEXT, optional)
    - Free-form notes about what happened.
- `created_at` (TEXT, ISO8601)

**Usage:**
- Provides a **timeline** of the application.
- Supports analytics (time-in-stage, number of interviews, etc.).

---

## 3. Status Flow & Business Rules

### 3.1 Status Definitions

- **Saved** – You’ve saved the job and created an application record but have not applied yet.
- **Draft** – You’re working on the resume/cover letter but haven’t submitted.
- **Applied** – Your application has been submitted.
- **Interviewing** – You’ve been invited to one or more interviews.
- **Offer** – You’ve received an offer.
- **Rejected** – Company has declined.
- **Ghosted** – No response after a configured period (manual mark in MVP).
- **Withdrawn** – You chose to withdraw your application.

### 3.2 Status Transitions

MVP: allow any transition, but the UI should **guide** the typical flow:

- Saved/Draft → Applied → Interviewing → Offer → (Accepted/Declined later) → Archived
- Applied → Rejected
- Applied → Ghosted
- Interviewing → Rejected
- Interviewing → Offer

Each status change should:

- Update `applications.status`.
- Update `last_activity_date` to `now()`.
- Optionally update `date_applied` if status becomes `Applied`.
- Create an `application_events` entry of type `"StatusChanged"`.
---

## 4. Tauri Command Design (Backend)

### 4.1 Commands

1. `create_application`
    - Input:
        - `job_id` (number, required)
        - Optional: `status`, `channel`, `priority`.
    - Behavior:
        - Insert a new row into `applications` with:
            - `status` default = `"Saved"` if not provided.
            - `date_saved` and `created_at` = `now()`.
            - `updated_at` = `now()`.
        - Insert an `application_events` row of type `"ApplicationCreated"`.
    - Output:
        - Full `Application` object (see types below).
2. `update_application`
    - Input:
        - `id` (application id, required).
        - Partial application fields (status, channel, dates, contact info, etc.).
    - Behavior:
        - Load existing application.
        - If `status` is changing:
            - Add `application_events` row with `event_type = "StatusChanged"`, `from_status`, `to_status`, and `event_date = now()`.
            - Update `last_activity_date = now()`.
            - If new status is `"Applied"` and `date_applied` is empty, set `date_applied = now()`.
        - Save updated fields to `applications`.
    - Output:
        - Updated `Application` object.
3. `get_applications`
    - Input (optional filters):
        - `status?` (string or array of strings)
        - `job_id?`
        - `active_only?` (boolean; filters `archived = 0`)
    - Output:
        - List of `ApplicationSummary` objects for pipeline view.
4. `get_application_detail`
    - Input: `application_id`.
    - Output:
        - Full `Application` object.
        - Associated `application_events` list ordered by `event_date` ascending.
5. `add_application_event`
    - Input:
        - `application_id`
        - `event_type`
        - `event_date` (optional; default `now()`)
        - `title` (optional)
        - `details` (optional)
    - Behavior:
        - Insert into `application_events`.
        - Update `last_activity_date` if appropriate.
    - Output:
        - Newly created `ApplicationEvent` object.
6. `archive_application`
    - Input: `application_id`.
    - Behavior:
        - Set `archived = 1`.
    - Output:
        - Updated `Application` object.

---

## 5. Frontend Types & State

### 5.1 TypeScript Interfaces

```rust
export type ApplicationStatus =
  | "Saved"
  | "Draft"
  | "Applied"
  | "Interviewing"
  | "Offer"
  | "Rejected"
  | "Ghosted"
  | "Withdrawn";

export interface Application {
  id: number;
  jobId: number;
  status: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateSaved: string; // ISO
  dateApplied?: string;
  lastActivityDate?: string;
  nextActionDate?: string;
  nextActionNote?: string;
  notesSummary?: string;
  contactName?: string;
  contactEmail?: string;
  contactLinkedin?: string;
  locationOverride?: string;
  offerCompensation?: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface ApplicationEvent {
  id: number;
  applicationId: number;
  eventType: string;
  eventDate: string;
  fromStatus?: ApplicationStatus;
  toStatus?: ApplicationStatus;
  title?: string;
  details?: string;
  createdAt: string;
}

export interface ApplicationSummary {
  id: number;
  jobId: number;
  jobTitle?: string;
  company?: string;
  status: ApplicationStatus;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateSaved: string;
  dateApplied?: string;
  lastActivityDate?: string;
}
```

### 5.2 Pipeline View State

```rust
interface ApplicationPipelineState {
  applicationsByStatus: Record<ApplicationStatus, ApplicationSummary[]>;
  isLoading: boolean;
  error?: string;
}
```

---

## 6. UI / UX Design

### 6.1 Where Application Pipeline Lives

- Main nav includes **"Applications"** or **"Pipeline"**.
- View types:
    - **Kanban-style columns** for each status (Saved, Applied, Interviewing, Offer, Rejected, Ghosted, Withdrawn).
    - Optional table view for compact listing.

### 6.2 Creating an Application

**From Job Detail View:**

- If the job does not yet have an application, show a button: **"Create Application"**.
- Clicking itz
    - Calls `create_application(job_id)`.
    - Opens Application Detail view (or sidebar) to edit details.
    - Default status: `Saved`.

**From Applications/Pipeline View:**
- Provide an **"Add Application"** button.
- User selects an existing job from dropdown/search.
- Then same as above.

### 6.3 Application Detail View

Sections:
1. **Header**
    - Job title @ company (read-only, from job data).
    - Status dropdown.
    - Priority dropdown.
    - Buttons: **Save**, **Archive**.
2. **Core Info**
    - Channel (select or text input).
    - Contact name/email/LinkedIn.
    - Location override.
3. **Dates & Next Actions**
    - Date saved (read-only).
    - Date applied (editable).
    - Last activity date (read-only).
    - Next action date (date picker).
    - Next action note (short text input).
4. **Notes**
    - Freeform text area for personal notes.
    - Later: AI summary / suggestions.
5. **Timeline (Events)**
    - List of `application_events`, sorted by `event_date`.
    - Each event shows:
        - event_date, event_type, title, details.
    - Button: **"Add Event"** (opens small form to log e.g. interview, follow-up, feedback).

**Behavior:**
- Changing status in the dropdown triggers `update_application` and auto-creates a `StatusChanged` event.
- Edits require explicit **Save**.
- If navigating away with unsaved changes, prompt: "You have unsaved changes. Save before leaving?".

### 6.4 Pipeline View (Kanban)
- Columns: statuses in a logical order.
- Within each column: cards representing `ApplicationSummary`.
- Each card shows:
    - Job title @ company.
    - Date applied (if any).
    - Last activity date.
    - Priority indicator.
- Drag & drop between columns:
    - Changes `status` via `update_application`.
    - Creates `StatusChanged` event.

If drag-and-drop is too heavy for MVP, fallback:
- Click a card → open detail view where status can be changed via dropdown.

---

## 7. Validation & Error Handling

### 7.1 Backend Rules

- `job_id` must refer to a valid job.
- `status` must not be empty.
- `date_saved` must be set on creation.
- For `application_events`:
    - `application_id` must exist.
    - `event_date` must not be empty.

### 7.2 Frontend Validation

- When creating an application:
    - Ensure a job is selected.
- When setting dates:
    - Optionally warn if `date_applied` is before `date_saved`.        

### 7.3 Error Messaging

- If backend returns error, show toast/banner with user-friendly message.
- Do not leave the UI in a half-updated state; revert local changes if save fails.

---

## 8. Hooks for Future AI Features

The Application Pipeline feeds several planned AI features:
- **Follow-up Suggestions:**
    - Use `status`, `last_activity_date`, and `next_action_date` to suggest follow-up emails and timing.
- **Interview Coaching:**
    - Analyze `application_events` (interview types, feedback) to tailor prep.
- **Conversion Analytics:**
    - Use status transitions and timestamps to compute conversion rates and time-in-stage.

Design the code so that:
- All significant changes generate `application_events`.
- The data model is rich enough to answer questions about "what happened when".

---

## 9. Testing Scenarios (High-Level)

1. **Create Application from Job**
    - From a Job Detail view, create an application.
    - Confirm `applications` row and `ApplicationCreated` event are created.
2. **Change Status**
    - Change status from `Saved` to `Applied`.
    - Confirm `status`, `last_activity_date`, and `date_applied` (if empty) updated.
    - Confirm `StatusChanged` event added.
3. **Add Interview Event**
    - Add an `InterviewScheduled` event.
    - Confirm event appears in timeline and `last_activity_date` updates.
4. **Pipeline View Grouping**
    - Add applications with multiple statuses.
    - Confirm pipeline view groups them correctly.
5. **Archive Application**
    - Archive an application and confirm it disappears from default pipeline view when `active_only` filter is true.
6. **Error Handling**
    - Simulate DB or Tauri error on update.
    - Ensure error is displayed and local state is not incorrectly marked saved.

---

## 10. Implementation Order (Recommended)

1. Create SQLite migrations for `applications` and `application_events`.
2. Implement Rust Tauri commands:
    - `create_application`
    - `update_application`
    - `get_applications`
    - `get_application_detail`
    - `add_application_event`
    - `archive_application`
3. Create TypeScript interfaces for `Application`, `ApplicationEvent`, and `ApplicationSummary`.
4. Build initial **Application Detail** view (form + timeline).
5. Build **Pipeline** view (grouped list by status; drag & drop later).
6. Integrate with Job Detail view’s **"Create Application"** button.