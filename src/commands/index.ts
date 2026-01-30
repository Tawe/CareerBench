/**
 * Combined Tauri command types
 * 
 * This file re-exports all command types from domain-specific modules
 * and provides the main Commands interface.
 */

// Import all command modules
import type { DashboardCommands } from './dashboard';
import type { ProfileCommands } from './profile';
import type { JobCommands } from './jobs';
import type { ApplicationCommands } from './applications';
import type { AiCommands } from './ai';
import type { CalendarCommands } from './calendar';
import type { CacheCommands } from './cache';

// Export all types for external use
export type { DashboardCommands, DashboardData, DashboardKpis, StatusBucket, DailyActivityPoint, FunnelStep } from './dashboard';
export type { ProfileCommands, UserProfile, UserProfileData, Experience, Skill, Education, Certification, PortfolioItem } from './profile';
export type { JobCommands, Job, JobSummary, CreateJobInput, UpdateJobInput, ParsedJob } from './jobs';
export type { ApplicationCommands, Application, ApplicationSummary, ApplicationEvent, ApplicationDetail, ApplicationStatus, CreateApplicationInput, UpdateApplicationInput, AddEventInput } from './applications';
export type { AiCommands, ResumeSection, ResumeSectionItem, GeneratedResume, GeneratedLetter, GenerationOptions, ResumeGenerationResult, LetterGenerationResult, Artifact, SaveResumeInput, SaveCoverLetterInput } from './ai';
export type { CalendarCommands, CalendarEvent } from './calendar';
export type { CacheCommands, CacheStats } from './cache';

// Re-export the remaining types that weren't in command modules
export interface Reminder {
  id?: number;
  applicationId?: number;
  eventId?: number;
  reminderType: string;
  reminderDate: string;
  message?: string;
  isSent: boolean;
  sentAt?: string;
  createdAt: string;
}

export interface EmailAccount {
  id?: number;
  emailAddress: string;
  provider: string;
  imapServer?: string;
  imapPort?: number;
  smtpServer?: string;
  smtpPort?: number;
  useSsl: boolean;
  isActive: boolean;
  lastSyncAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface EmailThread {
  id?: number;
  applicationId?: number;
  threadId: string;
  subject?: string;
  participants?: string;
  lastMessageDate?: string;
  messageCount: number;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface EmailMessage {
  id?: number;
  threadId: number;
  emailAccountId: number;
  messageId: string;
  fromAddress?: string;
  toAddress?: string;
  subject?: string;
  bodyText?: string;
  bodyHtml?: string;
  receivedDate: string;
  isRead: boolean;
  isApplicationEvent: boolean;
  eventType?: string;
  extractedData?: string;
  createdAt: string;
}

export interface SkillGap {
  skill: string;
  frequency: number;
  priority: "high" | "medium" | "low";
  userHasSkill: boolean;
  userRating?: number;
}

export interface LearningPlan {
  id?: number;
  title: string;
  description?: string;
  targetJobId?: number;
  skillGaps?: string; // JSON string
  estimatedDurationDays?: number;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface LearningTrack {
  id?: number;
  learningPlanId: number;
  title: string;
  description?: string;
  skillFocus?: string;
  orderIndex: number;
  createdAt: string;
}

export interface LearningTask {
  id?: number;
  learningTrackId: number;
  title: string;
  description?: string;
  taskType: string;
  resourceUrl?: string;
  estimatedHours?: number;
  completed: boolean;
  completedAt?: string;
  dueDate?: string;
  orderIndex: number;
  createdAt: string;
  updatedAt: string;
}

export interface LearningResource {
  id?: number;
  learningTaskId?: number;
  title: string;
  url?: string;
  resourceType: string;
  description?: string;
  createdAt: string;
}

export interface RecruiterContact {
  id?: number;
  name: string;
  email?: string;
  phone?: string;
  linkedinUrl?: string;
  company?: string;
  title?: string;
  notes?: string;
  relationshipStrength: string;
  lastContactDate?: string;
  tags?: string;
  createdAt: string;
  updatedAt: string;
}

export interface RecruiterInteraction {
  id?: number;
  contactId: number;
  interactionType: string;
  interactionDate: string;
  subject?: string;
  notes?: string;
  linkedApplicationId?: number;
  linkedJobId?: number;
  outcome?: string;
  followUpDate?: string;
  createdAt: string;
}

export interface ContactApplicationLink {
  id?: number;
  contactId: number;
  applicationId: number;
  role?: string;
  notes?: string;
  createdAt: string;
}

export interface Company {
  id?: number;
  name: string;
  website?: string;
  industry?: string;
  companySize?: string;
  location?: string;
  description?: string;
  mission?: string;
  vision?: string;
  values?: string;
  notes?: string;
  createdAt: string;
  updatedAt: string;
}

export interface CompanyWithStats extends Company {
  jobCount: number;
  applicationCount: number;
}

// Additional command interfaces for the remaining commands
export interface ReminderCommands {
  create_reminder: {
    args: [
      applicationId: number | null,
      eventId: number | null,
      reminderType: string,
      reminderDate: string,
      message: string | null
    ];
    return: number; // reminder ID
  };
  get_reminders: {
    args: [startDate: string, endDate: string, includeSent: boolean];
    return: Reminder[];
  };
  get_due_reminders: {
    args: [];
    return: Reminder[];
  };
  get_reminders_for_application: {
    args: [applicationId: number];
    return: Reminder[];
  };
  mark_reminder_sent: {
    args: [reminderId: number];
    return: void;
  };
  delete_reminder: {
    args: [reminderId: number];
    return: void;
  };
}

export interface EmailCommands {
  save_email_account: {
    args: [account: EmailAccount];
    return: number;
  };
  get_email_accounts: {
    args: [];
    return: EmailAccount[];
  };
  delete_email_account: {
    args: [accountId: number];
    return: void;
  };
  get_email_threads_for_application: {
    args: [applicationId: number];
    return: EmailThread[];
  };
  link_email_thread_to_application: {
    args: [threadId: number, applicationId: number];
    return: void;
  };
  get_email_messages_for_thread: {
    args: [threadId: number];
    return: EmailMessage[];
  };
  test_email_connection: {
    args: [email: string, password: string, provider: string];
    return: string;
  };
  sync_email_account: {
    args: [accountId: number];
    return: string;
  };
}

export interface LearningCommands {
  analyze_skill_gaps: {
    args: [jobId: number | null, includeAllJobs: boolean];
    return: SkillGap[];
  };
  create_learning_plan: {
    args: [
      title: string,
      description: string | null,
      targetJobId: number | null,
      skillGaps: SkillGap[],
      estimatedDurationDays: number | null
    ];
    return: number; // plan ID
  };
  get_learning_plans: {
    args: [status: string | null];
    return: LearningPlan[];
  };
  get_learning_tracks: {
    args: [learningPlanId: number];
    return: LearningTrack[];
  };
  get_learning_tasks: {
    args: [learningTrackId: number];
    return: LearningTask[];
  };
  create_learning_track: {
    args: [
      learningPlanId: number,
      title: string,
      description: string | null,
      skillFocus: string | null,
      orderIndex: number
    ];
    return: number; // track ID
  };
  create_learning_task: {
    args: [
      learningTrackId: number,
      title: string,
      description: string | null,
      taskType: string,
      resourceUrl: string | null,
      estimatedHours: number | null,
      dueDate: string | null,
      orderIndex: number
    ];
    return: number; // task ID
  };
  complete_learning_task: {
    args: [taskId: number, completed: boolean];
    return: void;
  };
  add_learning_resource: {
    args: [
      learningTaskId: number | null,
      title: string,
      url: string | null,
      resourceType: string,
      description: string | null
    ];
    return: number; // resource ID
  };
  get_learning_resources: {
    args: [learningTaskId: number];
    return: LearningResource[];
  };
  delete_learning_plan: {
    args: [planId: number];
    return: void;
  };
  update_learning_plan_status: {
    args: [planId: number, status: string];
    return: void;
  };
  generate_learning_content: {
    args: [learningPlanId: number, skillGaps: SkillGap[]];
    return: void;
  };
}

export interface RecruiterCommands {
  create_recruiter_contact: {
    args: [
      name: string,
      email: string | null,
      phone: string | null,
      linkedinUrl: string | null,
      company: string | null,
      title: string | null,
      notes: string | null,
      relationshipStrength: string | null,
      tags: string | null
    ];
    return: number; // contact ID
  };
  get_recruiter_contacts: {
    args: [companyFilter: string | null, searchQuery: string | null];
    return: RecruiterContact[];
  };
  get_recruiter_contact: {
    args: [contactId: number];
    return: RecruiterContact;
  };
  update_recruiter_contact: {
    args: [
      contactId: number,
      name: string | null,
      email: string | null,
      phone: string | null,
      linkedinUrl: string | null,
      company: string | null,
      title: string | null,
      notes: string | null,
      relationshipStrength: string | null,
      tags: string | null
    ];
    return: void;
  };
  delete_recruiter_contact: {
    args: [contactId: number];
    return: void;
  };
  create_interaction: {
    args: [
      contactId: number,
      interactionType: string,
      interactionDate: string,
      subject: string | null,
      notes: string | null,
      linkedApplicationId: number | null,
      linkedJobId: number | null,
      outcome: string | null,
      followUpDate: string | null
    ];
    return: number; // interaction ID
  };
  get_interactions_for_contact: {
    args: [contactId: number];
    return: RecruiterInteraction[];
  };
  get_interactions_for_application: {
    args: [applicationId: number];
    return: RecruiterInteraction[];
  };
  link_contact_to_application: {
    args: [contactId: number, applicationId: number, role: string | null, notes: string | null];
    return: number; // link ID
  };
  get_contacts_for_application: {
    args: [applicationId: number];
    return: RecruiterContact[];
  };
  get_applications_for_contact: {
    args: [contactId: number];
    return: number[];
  };
  unlink_contact_from_application: {
    args: [contactId: number, applicationId: number];
    return: void;
  };
  delete_interaction: {
    args: [interactionId: number];
    return: void;
  };
}

// Combined Commands interface
export interface Commands extends 
  DashboardCommands,
  ProfileCommands,
  JobCommands,
  ApplicationCommands,
  AiCommands,
  CalendarCommands,
  CacheCommands,
  ReminderCommands,
  EmailCommands,
  LearningCommands,
  RecruiterCommands {}

/**
 * Helper type to extract return type of a command
 */
export type CommandReturn<T extends keyof Commands> = Commands[T]['return'];

/**
 * Helper type to extract args type of a command
 */
export type CommandArgs<T extends keyof Commands> = Commands[T]['args'];