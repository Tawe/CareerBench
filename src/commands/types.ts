/**
 * TypeScript type definitions for all Tauri commands
 * 
 * These types match the Rust types in src-tauri/src/commands.rs
 * All types use camelCase to match Rust's serde rename_all = "camelCase"
 */

// ============================================================================
// Dashboard Types
// ============================================================================

export interface DashboardKpis {
  totalJobsTracked: number;
  totalApplications: number;
  activeApplications: number;
  applicationsLast30Days: number;
  offersReceived: number;
}

export interface StatusBucket {
  status: string;
  count: number;
}

export interface DailyActivityPoint {
  date: string;
  applicationsCreated: number;
  interviewsCompleted: number;
  offersReceived: number;
}

export interface FunnelStep {
  label: string;
  count: number;
}

export interface DashboardData {
  kpis: DashboardKpis;
  statusBreakdown: StatusBucket[];
  activityLast30Days: DailyActivityPoint[];
  funnel: FunnelStep[];
}

// ============================================================================
// User Profile Types
// ============================================================================

export interface UserProfile {
  id?: number;
  fullName: string;
  headline?: string;
  location?: string;
  summary?: string;
  currentRoleTitle?: string;
  currentCompany?: string;
  seniority?: string;
  openToRoles?: string;
  createdAt?: string;
  updatedAt?: string;
}

export interface Experience {
  id?: number;
  company: string;
  title: string;
  location?: string;
  startDate?: string;
  endDate?: string;
  isCurrent: boolean;
  description?: string;
  achievements?: string;
  techStack?: string;
}

export interface Skill {
  id?: number;
  name: string;
  category?: string;
  selfRating?: number;
  priority?: string;
  yearsExperience?: number;
  notes?: string;
}

export interface Education {
  id?: number;
  institution: string;
  degree?: string;
  fieldOfStudy?: string;
  startDate?: string;
  endDate?: string;
  grade?: string;
  description?: string;
}

export interface Certification {
  id?: number;
  name: string;
  issuingOrganization?: string;
  issueDate?: string;
  expirationDate?: string;
  credentialId?: string;
  credentialUrl?: string;
}

export interface PortfolioItem {
  id?: number;
  title: string;
  url?: string;
  description?: string;
  role?: string;
  techStack?: string;
  highlighted: boolean;
}

export interface UserProfileData {
  profile: UserProfile | null;
  experience: Experience[];
  skills: Skill[];
  education: Education[];
  certifications: Certification[];
  portfolio: PortfolioItem[];
}

// ============================================================================
// Job Types
// ============================================================================

export interface Job {
  id?: number;
  title?: string;
  company?: string;
  location?: string;
  jobSource?: string;
  postingUrl?: string;
  rawDescription?: string;
  parsedJson?: string;
  seniority?: string;
  domainTags?: string;
  isActive: boolean;
  dateAdded: string;
  lastUpdated: string;
}

export interface JobSummary {
  id: number;
  title?: string;
  company?: string;
  location?: string;
  seniority?: string;
  domainTags?: string;
  dateAdded: string;
}

export interface CreateJobInput {
  title?: string;
  company?: string;
  location?: string;
  jobSource?: string;
  postingUrl?: string;
  rawDescription?: string;
}

export interface UpdateJobInput {
  title?: string;
  company?: string;
  location?: string;
  jobSource?: string;
  postingUrl?: string;
  rawDescription?: string;
  isActive?: boolean;
}

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

// ============================================================================
// Application Types
// ============================================================================

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
  id?: number;
  jobId: number;
  status: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateSaved: string;
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

export interface ApplicationEvent {
  id?: number;
  applicationId: number;
  eventType: string;
  eventDate: string;
  fromStatus?: string;
  toStatus?: string;
  title?: string;
  details?: string;
  createdAt: string;
}

export interface ApplicationDetail {
  application: Application;
  events: ApplicationEvent[];
}

export interface CreateApplicationInput {
  jobId: number;
  status?: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
}

export interface UpdateApplicationInput {
  status?: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  dateApplied?: string;
  nextActionDate?: string;
  nextActionNote?: string;
  notesSummary?: string;
  contactName?: string;
  contactEmail?: string;
  contactLinkedin?: string;
  locationOverride?: string;
  offerCompensation?: string;
}

export interface AddEventInput {
  applicationId: number;
  eventType: string;
  eventDate: string;
  fromStatus?: string;
  toStatus?: string;
  title?: string;
  details?: string;
}

// ============================================================================
// Resume & Cover Letter Types
// ============================================================================

export interface ResumeSection {
  title: string;
  items: ResumeSectionItem[];
}

export interface ResumeSectionItem {
  heading: string;
  subheading?: string;
  bullets: string[];
}

export interface GeneratedResume {
  summary?: string;
  headline?: string;
  sections: ResumeSection[];
  highlights: string[];
}

export interface GeneratedLetter {
  subject?: string;
  greeting?: string;
  bodyParagraphs: string[];
  closing?: string;
  signature?: string;
}

export interface GenerationOptions {
  tone?: string;
  length?: string;
  focus?: string;
  audience?: string; // For cover letters
}

export interface ResumeGenerationResult {
  resume: GeneratedResume;
  content: string;
}

export interface LetterGenerationResult {
  letter: GeneratedLetter;
  content: string;
}

// ============================================================================
// Artifact Types
// ============================================================================

export interface Artifact {
  id: number;
  jobId?: number;
  applicationId?: number;
  type: "resume" | "cover_letter" | "notes" | "other";
  title: string;
  content?: string;
  filePath?: string;
  createdAt: string;
  updatedAt: string;
}

export interface SaveResumeInput {
  jobId: number;
  applicationId?: number;
  title: string;
  resume: GeneratedResume;
  content: string;
}

export interface SaveCoverLetterInput {
  jobId: number;
  applicationId?: number;
  title: string;
  letter: GeneratedLetter;
  content: string;
}

// ============================================================================
// Tauri Command Type Definitions
// ============================================================================

/**
 * Type-safe wrapper for Tauri invoke calls
 * 
 * Usage:
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * import type { Commands } from './commands/types';
 * 
 * const data = await invoke<Commands['get_dashboard_data']['return']>('get_dashboard_data');
 * ```
 */

export interface Commands {
  // Dashboard
  get_dashboard_data: {
    args: [];
    return: DashboardData;
  };

  // User Profile
  get_user_profile_data: {
    args: [];
    return: UserProfileData;
  };
  save_user_profile_data: {
    args: [data: UserProfileData];
    return: UserProfileData;
  };

  // Jobs
  create_job: {
    args: [input: CreateJobInput];
    return: Job;
  };
  update_job: {
    args: [id: number, input: UpdateJobInput];
    return: Job;
  };
  get_job_list: {
    args: [options?: { search?: string | null; activeOnly?: boolean; source?: string | null }];
    return: JobSummary[];
  };
  get_job_detail: {
    args: [id: number];
    return: Job;
  };
  parse_job_with_ai: {
    args: [jobId: number];
    return: ParsedJob;
  };

  // Applications
  create_application: {
    args: [input: CreateApplicationInput];
    return: Application;
  };
  update_application: {
    args: [id: number, input: UpdateApplicationInput];
    return: Application;
  };
  get_applications: {
    args: [options?: { status?: ApplicationStatus | null; jobId?: number | null; activeOnly?: boolean }];
    return: ApplicationSummary[];
  };
  get_application_detail: {
    args: [id: number];
    return: ApplicationDetail;
  };
  add_application_event: {
    args: [input: AddEventInput];
    return: ApplicationEvent;
  };
  archive_application: {
    args: [id: number];
    return: Application;
  };

  // Resume & Cover Letter Generation
  generate_resume_for_job: {
    args: [jobId: number, applicationId?: number | null, options?: GenerationOptions];
    return: ResumeGenerationResult;
  };
  generate_cover_letter_for_job: {
    args: [jobId: number, applicationId?: number | null, options?: GenerationOptions];
    return: LetterGenerationResult;
  };
  save_resume: {
    args: [input: SaveResumeInput];
    return: Artifact;
  };
  save_cover_letter: {
    args: [input: SaveCoverLetterInput];
    return: Artifact;
  };
  update_artifact_title: {
    args: [id: number, title: string];
    return: Artifact;
  };

  // AI Settings
  get_ai_settings: {
    args: [];
    return: import('../ai/types').AiSettings;
  };
  save_ai_settings: {
    args: [settings: import('../ai/types').AiSettings];
    return: void;
  };
  test_ai_connection: {
    args: [];
    return: string;
  };
  check_local_provider_availability: {
    args: [];
    return: boolean;
  };

  // AI Operations (lower-level)
  ai_resume_suggestions: {
    args: [input: import('../ai/types').ResumeInput];
    return: import('../ai/types').ResumeSuggestions;
  };
  ai_cover_letter: {
    args: [input: import('../ai/types').CoverLetterInput];
    return: import('../ai/types').CoverLetter;
  };
  ai_skill_suggestions: {
    args: [input: import('../ai/types').SkillSuggestionsInput];
    return: import('../ai/types').SkillSuggestions;
  };

  // Profile AI Enhancements
  generate_profile_summary: {
    args: [];
    return: string;
  };
  extract_skills_from_experience: {
    args: [];
    return: string[];
  };
  rewrite_portfolio_description: {
    args: [portfolioId: number, description: string];
    return: string;
  };
}

/**
 * Helper type to extract return type of a command
 */
export type CommandReturn<T extends keyof Commands> = Commands[T]['return'];

/**
 * Helper type to extract args type of a command
 */
export type CommandArgs<T extends keyof Commands> = Commands[T]['args'];

