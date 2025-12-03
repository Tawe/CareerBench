# CareerBench Overview Specification

## 1. Product Summary

CareerBench is a self-hosted, AI-powered desktop application designed to help users manage their job search, generate tailored application materials, track interviews, and develop personalized learning plans. Built as a Tauri application with a local SQLite database, CareerBench uses AI to automate resume customization, job parsing, follow-up recommendations, interview preparation, and skill-gap analysis.

---

## 2. Core Modules

### **A. User Profile Module**

Stores structured information about the user for AI-driven personalization.

- Personal details (name, location, role)
- Work experience entries
- Skills (technical, soft, domain)
- Education & certifications
- Portfolio items
- Parsed resume import (future enhancement)

### **B. Job Intake Module**

Allows users to add jobs through:
- Text paste of job descriptions
- URL (later enhancement)
- Manual entry

AI processing extracts:
- Required & preferred skills
- Responsibilities
- Seniority estimation
- Domain tags

### **C. Application Tracker & Calendar**

Tracks:
- Application status (Saved, Applied, Interviewing, Offer, Rejected, Ghosted)
- Dates (applied, last activity, follow-up reminders)
- Interview events
- Recruiter contacts

Calendar view visualizes:
- Interviews
- Follow-up deadlines
- Application timelines

### **D. Artifact & Notes Module**

Each application may contain:
- Tailored resume
- Cover letter / outreach email
- Interview notes
- Email threads (manually pasted)
- Take-home submissions
- AI summaries

### **E. AI Resume & Cover Letter Generator**

Uses:
- User profile snapshot  
- Job description
- Skill alignment
- Tone/style preferences

Outputs:
- Structured resume sections
- Markdown/Plain Text resume
- Optional cover letter or outreach message

Caching prevents repeat charges for identical generations.

### **F. Learning Plan Generator**

AI compares user skills vs. aggregated job requirements. Produces:
- Priority skill gaps
- Curated learning tracks
- Suggested timeline
- Resource links
- Practice project ideas

Learning Tasks are tracked via a dedicated progress system.

### **G. Dashboard & Alerts**

Displays:
- Funnel metrics (Saved → Applied → Interview → Offer)    
- Time-in-stage analytics
- Weekly application activity
    
Alert system recommends:
- Follow-ups for stale applications
- Resume adjustments based on repeated rejection patterns
- Reminders to apply to saved roles

---

## 3. Architecture & Technology Stack
- **Frontend:** Tauri (React/TypeScript)
- **Backend:** Rust commands for DB, file management, AI calls
- **Database:** SQLite with migrations
- **AI Providers:** OpenAI, Gemini, Local LLM (pluggable)
- **AI Caching:** Stored per request hash (purpose + payload)

Backend responsibilities:
- Fetching/saving jobs
- AI call orchestration
- Generating alerts
- Maintaining full audit logs of artifacts

---

## 4. Data Model Overview

### **user_profile**
- name, headline, location
- summary
- seniority

### **experience**
- company, title, dates
- achievements
- tech stack

### **skills**
- name, category
- rating (self-assessed)
- priority

### **jobs**
- title, company, location
- posting_url
- raw_description
- parsed_json
- date_added

### **applications**
- job_id
- status
- dates (applied, last_updated, next_action)
- channel
- notes_summary

### **application_events**
- event_type
- details
- timestamp

### **artifacts**
- type (resume, letter, notes, etc.)
- content or file_path

### **learning_goals / learning_tasks**
- AI-generated objectives
- task lists with progress tracking

### **ai_cache**
- request hash
- purpose
- model
- request payload
- response payload

---

## 5. AI Workflows

### **Job Parsing Workflow**
1. User adds job
2. AI extracts structured fields
3. Save to DB
4. Use structured data throughout UI

### **Resume Generator Workflow**
1. Load user profile snapshot
2. Load job data
3. AI generates tailored resume
4. Output stored as artifact

### **Learning Plan Workflow**
1. Analyze all job skill frequencies
2. Compare with user skills
3. AI generates roadmap
4. Persist goals & tasks

### **Alert Workflow**
1. Periodic scan of application data
2. AI interprets patterns
3. Suggestions displayed on dashboard

---

## 6. MVP Scope
- User profile entry
- Job intake (paste text)
- Basic job parsing
- Application pipeline
- Resume & cover letter generator
- Simple dashboard visualization
- Local AI caching

---

## 7. Future Enhancements
- Job scraping via URL
- Multi-profile support
- Email integration
- Offline local LLM mode
- Portfolio builder
- Recruiter CRM view

---

## 8. Next Steps
1. Create initial Tauri app scaffold
2. Implement SQLite migrations
3. Build Job Intake → AI Parse → Detail Page vertical slice
4. Add Resume Generator vertical slice
5. Integrate caching layer
