# CareerBench TODO List

This document tracks all remaining tasks based on the project specifications. Tasks are organized by feature area and priority.

## 🔴 High Priority - Core Features

### AI Provider System
- [x] **Local Model Integration** (Complete - Ready for Testing)
  - [x] Choose local model library (llama-cpp-sys-3)
  - [x] Implement model loading structure in `LocalProvider`
  - [x] Implement actual model loading with llama.cpp C API
  - [x] Implement inference pipeline for local models
  - [x] Add prompt formatting for local models
  - [ ] Test local model inference end-to-end (requires GGUF model download)
  - Reference: `docs/specs/ai-provider.md`
  - Status: See `docs/specs/ai-provider/local-model-complete.md` for details

### Testing Infrastructure
- [ ] **Write Rust Unit Tests**
  - [x] Data model helpers (serialization/deserialization)
  - [x] AI cache helpers (`compute_input_hash`, `ai_cache_get`, `ai_cache_put`)
  - [x] Query/aggregation helpers (dashboard queries)
  - [x] Pure helpers (`render_resume_to_text`, `render_letter_to_text`, `format_date`)
  - Reference: `docs/specs/testing/testing.md`

- [x] **Write Rust Integration Tests**
  - [x] `generate_resume_for_job` command
  - [x] `generate_cover_letter_for_job` command
  - [x] `parse_job_with_ai` command
  - [x] `get_dashboard_data` command
  - [x] `create_application`, `update_application_status` commands
  - [x] Use mock AI client for deterministic tests
  - Reference: `docs/specs/testing/testing.md`

- [x] **Write TypeScript/Frontend Tests**
  - [x] Hooks (`useDashboardData`, `useApplications`, `useJobs`)
  - [x] Pure utilities (filtering, sorting, status labels)
  - [x] UI rendering tests (Dashboard, Jobs, Applications, Profile pages)
  - [x] Core behavior tests (status filters, sheet open/close, unsaved changes)
  - Reference: `docs/specs/testing/testing.md`

- [x] **Golden File Tests**
  - [x] Update fixtures for `ParsedJob`, `GeneratedResume`, `GeneratedLetter`
  - [x] Add deserialization tests for all AI output types
  - [x] Validate schema changes break tests appropriately
  - Reference: `docs/specs/testing/testing.md`

## 🟡 Medium Priority - Feature Completion

### Job Intake & Parsing
- [x] **Job URL Scraping** (Implemented)
  - [x] Implement URL parsing and fetching - created job_scraper.rs module with HTTP client and HTML parsing using scraper crate, added scrape_job_url Tauri command
  - [x] Extract job description from web pages - implemented scrapers for LinkedIn, Indeed, Glassdoor, and generic sites with fallback to meta tags and JSON-LD structured data
  - [x] Handle different job board formats - added board detection (LinkedIn, Indeed, Glassdoor, Monster, ZipRecruiter, Dice) with board-specific CSS selectors, generic scraper for unknown sites
  - [x] Added UI integration - "Scrape" button in Add Job modal that extracts job details from URL and populates form fields
  - Reference: `docs/specs/features/job-intake.md`

### Resume & Cover Letter Generator
- [x] **Artifacts Table Implementation**
  - [x] Create/verify `artifacts` table migration
  - [x] Implement artifact CRUD operations
  - [x] Link artifacts to applications/jobs
  - Reference: `docs/specs/features/resume-coverletter.md`

- [x] **Resume/Letter Rendering**
  - [x] Implement `render_resume_to_text` helper
  - [x] Implement `render_letter_to_text` helper
  - [x] Support Markdown and plaintext formats
  - [ ] Add template system for different resume styles (future)
  - Reference: `docs/specs/features/resume-coverletter.md`

- [x] **Frontend Generation UI**
  - [x] Build generation modal/sheet with options (tone, length, focus)
  - [x] Add preview panel (structured view + raw text view)
  - [x] Implement save/edit/discard actions
  - [x] Link generated artifacts to applications
  - Reference: `docs/specs/features/resume-coverletter.md`

### Application Pipeline
- [x] **Pipeline View Enhancements**
  - [x] Implement drag-and-drop for status changes (or fallback to dropdown) - dropdown implemented
  - [x] Add kanban-style column view
  - [x] Add table view option
  - [x] Improve status transition UI guidance - inline status dropdowns with hover visibility
  - Reference: `docs/specs/features/application-pipeline.md`

- [x] **Event Timeline**
  - [x] Enhance timeline visualization
  - [x] Add event filtering/sorting
  - [x] Improve event creation flow
  - Reference: `docs/specs/features/application-pipeline.md`

### Dashboard
- [x] **Visualization Improvements**
  - [x] Add chart library (Recharts) integration
  - [x] Implement activity chart (line/bar chart for last 30 days)
  - [x] Enhance funnel visualization
  - [x] Add status breakdown chart
  - Reference: `docs/specs/features/simple-dashboard-visualization.md`

- [x] **Dashboard Features**
  - [x] Add refresh button functionality - already implemented
  - [ ] Add date range selector (future)
  - [ ] Add export functionality (future)
  - Reference: `docs/specs/features/simple-dashboard-visualization.md`

### User Profile
- [x] **Profile Enhancements**
  - [x] Add AI-assisted summary generation - UI added, command stubbed (needs full AI implementation)
  - [x] Add AI skill extraction from experience - UI added, command stubbed (needs full AI implementation)
  - [x] Add AI portfolio description rewriting - UI added, command stubbed (needs full AI implementation)
  - [x] Improve validation and error handling - added validation function, inline error messages, replaced alerts with toast notifications
  - Reference: `docs/specs/features/user-profile.md`

- [x] **Profile Import** (Completed)
  - [x] PDF and TXT resume parsing - implemented extract_text_from_pdf and extract_text_from_txt
  - [x] DOCX resume parsing - implemented extract_text_from_docx using docx-rs library
  - [x] AI-powered profile extraction - added generic call_llm method to AiProvider trait, implemented in all providers (Cloud, Local, Hybrid, Mock)
  - [x] Frontend integration - Profile page has "Import Resume" button with file dialog, extracts text, processes with AI, and merges data
  - [x] Caching - AI extraction results are cached to avoid redundant calls
  - [ ] LinkedIn import
  - [x] CV/Resume file parsing - implemented PDF and TXT parsing, added file picker UI, created extract_resume_text command, DOCX parsing TODO (needs proper docx-rs API integration), AI extraction partially implemented (needs generic LLM call method in AiProvider trait)
  - [ ] Multi-profile support
  - Reference: `docs/specs/features/user-profile.md`

## 🟢 Low Priority - Polish & Enhancements

### UI/UX Improvements
- [ ] **Notion-like Interactions**
  - [x] Implement hover-first UI for all cards/lists - actions now show on hover for all Profile cards, skill items, job cards, and application cards
  - [x] Add inline editing for more fields - added InlineEditable component for job titles/companies, application notes, profile name/location, and skill names
  - [x] Improve sheet animations and transitions - enhanced with backdrop blur, smoother easing (cubic-bezier), improved shadows, and exit animations
  - [x] Add keyboard shortcuts (`Cmd+K` quick switcher, etc.) - implemented Cmd+K/Ctrl+K quick switcher for navigation
  - Reference: `docs/specs/design/ui-design.md`

- [x] **Component Polish**
  - [x] Ensure all inputs use soft styling - standardized all inputs to use soft styling with consistent focus states
  - [x] Add loading skeletons for all async operations - implemented LoadingSkeleton component with variants (text, card, list, table) and integrated into all pages
  - [x] Improve error states and messaging - enhanced error banners with better styling and messaging
  - [x] Add toast notifications for actions - implemented Toast system with success/error/info/warning types, replaced all alert() calls with toasts
  - Reference: `docs/specs/design/ui-design.md`

- [ ] **Accessibility**
  - [x] Add ARIA labels to interactive elements
  - [x] Ensure keyboard navigation works everywhere
  - [ ] Test with screen readers
  - [x] Add focus indicators

### AI Features
- [ ] **Additional AI Providers**
  - [x] Add Anthropic API support - implemented call_anthropic method with proper API format (messages endpoint, x-api-key header, anthropic-version header), integrated into all AI operations (resume, cover letter, skills, job parsing), updated Settings UI with Anthropic option and model recommendations
  - [ ] Add other cloud providers as needed
  - [x] Implement hybrid mode routing logic - created HybridProvider with intelligent routing and automatic fallback, tries preferred provider first (cloud if both configured), falls back to secondary provider on recoverable errors (network, rate limits), supports both cloud and local providers simultaneously
  - Reference: `docs/specs/ai-provider.md`

- [ ] **AI Enhancements**
  - [x] Add progress indicators for long-running AI operations - implemented ProgressIndicator component with spinner, steps, and bar variants, integrated into resume/cover letter generation and job parsing
  - [x] Implement retry logic for transient failures - created retry utility with exponential backoff, integrated into cloud provider API calls, retries on NetworkError and RateLimitExceeded, configurable max retries and delays
  - [x] Add rate limiting for cloud providers - implemented token bucket rate limiter, integrated into cloud provider API calls, provider-specific limits (OpenAI: 50/min, Anthropic: 50/min), automatic token refill over time
  - [x] Improve error messages and recovery - created error message utilities (Rust and TypeScript), user-friendly error messages with recovery suggestions, improved error display in UI with actionable guidance, error categorization (recoverable vs requires action)
  - Reference: `docs/specs/ai-provider.md`

- [ ] **Learning Plan Generator** (Future Feature)
  - [ ] Implement skill gap analysis
  - [ ] Generate learning tracks
  - [ ] Create task lists with progress tracking
  - [ ] Add resource links and practice projects
  - Reference: `docs/specs/overview.md`

### Data & Performance
- [ ] **Caching Improvements**
  - [ ] Add cache invalidation strategies
  - [ ] Implement cache size limits
  - [ ] Add cache statistics/management UI
  - Reference: `docs/specs/features/local-ai-caching.md`

- [ ] **Database Optimizations**
  - [ ] Add indexes for common queries
  - [ ] Optimize dashboard aggregation queries
  - [ ] Add database migration versioning
  - [ ] Implement database backup/restore

- [ ] **Performance**
  - [ ] Optimize large list rendering
  - [ ] Add pagination for jobs/applications
  - [ ] Implement virtual scrolling if needed
  - [ ] Optimize bundle size

### Future Features (From Overview Spec)
- [ ] **Email Integration**
  - [ ] Connect to email providers
  - [ ] Auto-import application events from emails
  - [ ] Track email threads

- [ ] **Calendar Integration**
  - [ ] Sync interviews with system calendar
  - [ ] Add calendar view for applications
  - [ ] Reminder notifications

- [ ] **Analytics & Insights**
  - [ ] Conversion rate analytics
  - [ ] Time-in-stage metrics
  - [ ] Channel effectiveness analysis
  - [ ] AI coaching based on patterns

- [ ] **Portfolio Builder**
  - [ ] Rich portfolio item editor
  - [ ] Portfolio export/generation
  - [ ] Link portfolio to applications

- [ ] **Recruiter CRM**
  - [ ] Contact management
  - [ ] Interaction history
  - [ ] Relationship tracking

## 🔧 Technical Debt & Maintenance

### Code Quality
- [ ] **Documentation**
  - [x] Add JSDoc comments to all public functions - added comprehensive JSDoc to utility functions (applicationUtils, errorUtils) and custom React hooks (useDashboardData, useApplications, useJobs) with @param, @returns, and @example tags
  - [x] Document complex algorithms and data flows - created comprehensive documentation for resume generation pipeline (multi-level caching), AI provider resolution and hybrid mode routing, AI caching system (hash computation and lookup), and local AI inference pipeline (llama.cpp integration)
  - [x] Create architecture decision records (ADRs) - created ADR directory structure and documented 4 key decisions: Tauri framework choice, pluggable AI provider architecture, multi-level caching strategy, and hybrid mode routing with automatic fallback
  - [x] Update README with setup and development instructions - enhanced README with comprehensive setup instructions, prerequisites (including platform-specific), development workflow, project structure, key technologies, development resources, and updated project status

- [ ] **Type Safety**
  - [x] Ensure all Tauri commands have proper TypeScript types - created comprehensive centralized type definitions file (src/commands/types.ts) with all command types, input/output types, and helper types for type-safe invoke calls
  - [x] Add runtime validation for AI responses - created validation module (src-tauri/src/ai/validation.rs) with schema validation for all AI response types (ParsedJobOutput, ResumeSuggestions, CoverLetter, SkillSuggestions), integrated into both cloud and local providers, includes business rule validation (e.g., seniority_score range, required fields, importance values)
  - [x] Use Zod or similar for frontend validation - installed Zod, created comprehensive validation schemas (src/validation/schemas.ts) for all data types (UserProfile, Experience, Skills, Education, Certifications, Portfolio, Jobs, Applications, AI Settings), created validation utilities (src/validation/utils.ts) with validate, validateOrThrow, validateSafe functions, updated Profile form to use Zod validation

- [ ] **Error Handling**
  - [x] Standardize error types across Rust modules - created unified error type system (src-tauri/src/errors.rs) with CareerBenchError enum containing DatabaseError, AiProviderError, ValidationError, ConfigurationError, and FileSystemError variants, implemented std::error::Error for all types, added automatic conversions from rusqlite::Error and std::io::Error, created to_user_message() and get_short_error_message() helpers for user-friendly error formatting, integrated into commands module with example usage
  - [x] Improve error messages for users - error types now provide structured, user-friendly messages via to_user_message() function, integrates with existing ai::error_messages module for AI-specific errors
  - [x] Add error logging/monitoring - enhanced logging.rs with log_careerbench_error() function for structured error logging, added automatic error logging in CareerBenchError::to_string_for_tauri() and log_and_return() methods, created error_logging.rs module with ErrorMetrics tracking (error counts by type/context, recent error records, recoverability tracking), integrated error metrics initialization in main.rs, all errors are now automatically logged and tracked when converted to strings for Tauri commands

### Security
- [ ] **API Key Security**
  - [x] Encrypt API keys in database - implemented AES-GCM encryption (src-tauri/src/encryption.rs) with key derivation from system identifier, integrated encryption/decryption into save_ai_settings() and load_ai_settings() functions, API keys are now automatically encrypted before storage and decrypted when loaded, backward compatible with existing plaintext keys
  - [x] Use Tauri secure storage for sensitive data - created secure_storage.rs module with OS keychain integration (macOS Keychain via security command, Windows Credential Manager placeholder, Linux Secret Service placeholder), API keys now stored in OS keychain when available with fallback to encrypted file storage, updated AI settings to use secure storage instead of database for API keys, maintains backward compatibility with existing encrypted database values
  - [x] Add key rotation support - implemented key rotation system (src-tauri/src/ai/key_rotation.rs) with validation before rotation, added KeyMetadata tracking (creation date, last rotated, rotation count) in secure_storage.rs, created rotate_secret() function with optional validator, added rotate_api_key() command that validates new key with AI provider before rotation, added get_api_key_metadata() and check_api_key_rotation_needed() commands for monitoring, keys are validated before rotation to prevent invalid keys from replacing valid ones

- [ ] **Data Privacy**
  - [x] Add data export functionality - created comprehensive data export module (src-tauri/src/data_export.rs) that exports all user data (profile, jobs, applications, artifacts, events) in structured JSON format, includes export metadata (timestamp, version, record counts), created export_all_data() Tauri command that returns JSON string, exports include all relationships (applications with events, artifacts linked to jobs/applications), data is exported in a format suitable for backup, migration, or privacy compliance
  - [x] Add data deletion/privacy controls - created data deletion module (src-tauri/src/data_deletion.rs) with functions to delete individual records (jobs, applications, artifacts, profile sections), implemented delete_all_user_data() for GDPR "Right to be Forgotten" compliance, added get_deletion_summary() to show counts before deletion, created Tauri commands: delete_job(), delete_application(), delete_artifact(), delete_profile_section(), delete_all_user_data(), get_deletion_summary(), all deletions respect foreign key constraints and include proper logging
  - [x] Ensure local-first data storage - verified and documented that all user data is stored locally (SQLite database, logs, secure storage files), created local_storage.rs module with verification functions (verify_local_storage(), get_storage_info(), get_storage_size()), updated db.rs documentation to clarify local-first storage approach, added Tauri commands: get_storage_info(), verify_local_storage(), get_storage_size(), all data storage is confirmed to be local with no cloud dependencies for user data (only AI API calls use cloud when configured)

### Build & Deployment
- [ ] **CI/CD**
  - [x] Set up GitHub Actions or similar - created comprehensive GitHub Actions workflows (.github/workflows/ci.yml and pr-checks.yml) with Rust tests (unit and integration), TypeScript tests, Rust formatting checks (cargo fmt), Clippy linting, TypeScript compilation checks, build verification across platforms (Linux, macOS, Windows), caching for dependencies, and PR-specific quick checks workflow
  - [x] Run tests on every PR - PR checks workflow runs on all PR events, includes Rust and TypeScript tests, formatting and linting checks
  - [x] Block merge on test failures - workflows are configured to fail on test/lint errors; branch protection rules need to be configured in GitHub Settings → Branches (see docs/development/ci-cd.md for instructions)
  - [x] Add linting checks - added Clippy checks for Rust (warnings as errors), TypeScript compilation checks, Rust formatting verification

- [ ] **Packaging**
  - [ ] Optimize Tauri bundle size
  - [ ] Add auto-update mechanism
  - [ ] Create installers for all platforms
  - [ ] Code signing for releases

## 📝 Notes

### Completed Features
- ✅ AI Provider Architecture (core structure)
- ✅ Tauri Commands for AI operations
- ✅ Settings UI for AI configuration
- ✅ Frontend integration for AI features
- ✅ Resume/Cover Letter integration with AI provider
- ✅ Job Parsing integration with AI provider
- ✅ Testing infrastructure setup
- ✅ Basic UI/UX improvements (Notion-like styling)
- ✅ Profile screen redesign
- ✅ Skills section refinement
- ✅ Date pickers implementation
- ✅ Experience section fixes
- ✅ Dashboard calendar view
- ✅ Collapsible sidebar

### Current Status
- The AI provider system is fully integrated into existing commands
- Local model inference is fully implemented (C API integration complete, ready for testing with GGUF model)
- Testing infrastructure is set up but tests need to be written
- Most core features are implemented but need polish and testing

### Priority Guidelines
- **High Priority**: Features that block core functionality or are required for MVP
- **Medium Priority**: Features that enhance existing functionality or complete feature sets
- **Low Priority**: Nice-to-have features, polish, and future enhancements

---

Last Updated: Based on review of all spec documents in `docs/specs/`

