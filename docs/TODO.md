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
  - Status: See `docs/specs/ai-provider/LOCAL_MODEL_COMPLETE.md` for details

### Testing Infrastructure
- [ ] **Write Rust Unit Tests**
  - [ ] Data model helpers (serialization/deserialization)
  - [ ] AI cache helpers (`compute_input_hash`, `ai_cache_get`, `ai_cache_put`)
  - [ ] Query/aggregation helpers (dashboard queries)
  - [ ] Pure helpers (`render_resume_to_text`, `render_letter_to_text`)
  - Reference: `docs/specs/testing/testing.md`

- [ ] **Write Rust Integration Tests**
  - [ ] `generate_resume_for_job` command
  - [ ] `generate_cover_letter_for_job` command
  - [ ] `parse_job_with_ai` command
  - [ ] `get_dashboard_data` command
  - [ ] `create_application`, `update_application_status` commands
  - [ ] Use mock AI client for deterministic tests
  - Reference: `docs/specs/testing/testing.md`

- [ ] **Write TypeScript/Frontend Tests**
  - [ ] Hooks (`useDashboardData`, `useApplications`, `useJobs`)
  - [ ] Pure utilities (filtering, sorting, status labels)
  - [ ] UI rendering tests (Dashboard, Jobs, Applications, Profile pages)
  - [ ] Core behavior tests (status filters, sheet open/close, unsaved changes)
  - Reference: `docs/specs/testing/testing.md`

- [ ] **Golden File Tests**
  - [ ] Update fixtures for `ParsedJob`, `GeneratedResume`, `GeneratedLetter`
  - [ ] Add deserialization tests for all AI output types
  - [ ] Validate schema changes break tests appropriately
  - Reference: `docs/specs/testing/testing.md`

## 🟡 Medium Priority - Feature Completion

### Job Intake & Parsing
- [ ] **Job URL Scraping** (Future Enhancement)
  - [ ] Implement URL parsing and fetching
  - [ ] Extract job description from web pages
  - [ ] Handle different job board formats
  - Reference: `docs/specs/features/job-intake.md`

### Resume & Cover Letter Generator
- [ ] **Artifacts Table Implementation**
  - [ ] Create/verify `artifacts` table migration
  - [ ] Implement artifact CRUD operations
  - [ ] Link artifacts to applications/jobs
  - Reference: `docs/specs/features/resume-coverletter.md`

- [ ] **Resume/Letter Rendering**
  - [ ] Implement `render_resume_to_text` helper
  - [ ] Implement `render_letter_to_text` helper
  - [ ] Support Markdown and plaintext formats
  - [ ] Add template system for different resume styles (future)
  - Reference: `docs/specs/features/resume-coverletter.md`

- [ ] **Frontend Generation UI**
  - [ ] Build generation modal/sheet with options (tone, length, focus)
  - [ ] Add preview panel (structured view + raw text view)
  - [ ] Implement save/edit/discard actions
  - [ ] Link generated artifacts to applications
  - Reference: `docs/specs/features/resume-coverletter.md`

### Application Pipeline
- [ ] **Pipeline View Enhancements**
  - [ ] Implement drag-and-drop for status changes (or fallback to dropdown)
  - [ ] Add kanban-style column view
  - [ ] Add table view option
  - [ ] Improve status transition UI guidance
  - Reference: `docs/specs/features/application-pipeline.md`

- [ ] **Event Timeline**
  - [ ] Enhance timeline visualization
  - [ ] Add event filtering/sorting
  - [ ] Improve event creation flow
  - Reference: `docs/specs/features/application-pipeline.md`

### Dashboard
- [ ] **Visualization Improvements**
  - [ ] Add chart library (Recharts) integration
  - [ ] Implement activity chart (line/bar chart for last 30 days)
  - [ ] Enhance funnel visualization
  - [ ] Add status breakdown chart
  - Reference: `docs/specs/features/simple-dashboard-visualization.md`

- [ ] **Dashboard Features**
  - [ ] Add refresh button functionality
  - [ ] Add date range selector (future)
  - [ ] Add export functionality (future)
  - Reference: `docs/specs/features/simple-dashboard-visualization.md`

### User Profile
- [ ] **Profile Enhancements**
  - [ ] Add AI-assisted summary generation
  - [ ] Add AI skill extraction from experience
  - [ ] Add AI portfolio description rewriting
  - [ ] Improve validation and error handling
  - Reference: `docs/specs/features/user-profile.md`

- [ ] **Profile Import** (Future Enhancement)
  - [ ] LinkedIn import
  - [ ] CV/Resume file parsing
  - [ ] Multi-profile support
  - Reference: `docs/specs/features/user-profile.md`

## 🟢 Low Priority - Polish & Enhancements

### UI/UX Improvements
- [ ] **Notion-like Interactions**
  - [ ] Implement hover-first UI for all cards/lists
  - [ ] Add inline editing for more fields
  - [ ] Improve sheet animations and transitions
  - [ ] Add keyboard shortcuts (`Cmd+K` quick switcher, etc.)
  - Reference: `docs/specs/design/ui-design.md`

- [ ] **Component Polish**
  - [ ] Ensure all inputs use soft styling
  - [ ] Add loading skeletons for all async operations
  - [ ] Improve error states and messaging
  - [ ] Add toast notifications for actions
  - Reference: `docs/specs/design/ui-design.md`

- [ ] **Accessibility**
  - [ ] Add ARIA labels to interactive elements
  - [ ] Ensure keyboard navigation works everywhere
  - [ ] Test with screen readers
  - [ ] Add focus indicators

### AI Features
- [ ] **Additional AI Providers**
  - [ ] Add Anthropic API support
  - [ ] Add other cloud providers as needed
  - [ ] Implement hybrid mode routing logic
  - Reference: `docs/specs/ai-provider.md`

- [ ] **AI Enhancements**
  - [ ] Add progress indicators for long-running AI operations
  - [ ] Implement retry logic for transient failures
  - [ ] Add rate limiting for cloud providers
  - [ ] Improve error messages and recovery
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
  - [ ] Add JSDoc comments to all public functions
  - [ ] Document complex algorithms and data flows
  - [ ] Create architecture decision records (ADRs)
  - [ ] Update README with setup and development instructions

- [ ] **Type Safety**
  - [ ] Ensure all Tauri commands have proper TypeScript types
  - [ ] Add runtime validation for AI responses
  - [ ] Use Zod or similar for frontend validation

- [ ] **Error Handling**
  - [ ] Standardize error types across Rust modules
  - [ ] Improve error messages for users
  - [ ] Add error logging/monitoring

### Security
- [ ] **API Key Security**
  - [ ] Encrypt API keys in database
  - [ ] Use Tauri secure storage for sensitive data
  - [ ] Add key rotation support

- [ ] **Data Privacy**
  - [ ] Add data export functionality
  - [ ] Add data deletion/privacy controls
  - [ ] Ensure local-first data storage

### Build & Deployment
- [ ] **CI/CD**
  - [ ] Set up GitHub Actions or similar
  - [ ] Run tests on every PR
  - [ ] Block merge on test failures
  - [ ] Add linting checks

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

