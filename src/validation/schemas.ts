/**
 * Zod validation schemas for frontend data validation
 * 
 * These schemas match the TypeScript types in src/commands/types.ts
 * and provide runtime validation for user inputs and API responses.
 */

import { z } from 'zod';

// ============================================================================
// User Profile Schemas
// ============================================================================

export const userProfileSchema = z.object({
  id: z.number().optional(),
  fullName: z.string().min(1, "Full name is required"),
  headline: z.string().optional(),
  location: z.string().optional(),
  summary: z.string().optional(),
  currentRoleTitle: z.string().optional(),
  currentCompany: z.string().optional(),
  seniority: z.string().optional(),
  openToRoles: z.string().optional(),
  createdAt: z.string().optional(),
  updatedAt: z.string().optional(),
});

export const experienceSchema = z.object({
  id: z.number().optional(),
  company: z.string().min(1, "Company is required"),
  title: z.string().min(1, "Job title is required"),
  location: z.string().optional(),
  startDate: z.string().optional(),
  endDate: z.string().optional(),
  isCurrent: z.boolean().default(false),
  description: z.string().optional(),
  achievements: z.string().optional(),
  techStack: z.string().optional(),
}).refine(
  (data) => {
    // If not current job, end date must be after start date
    if (!data.isCurrent && data.startDate && data.endDate) {
      const start = new Date(data.startDate);
      const end = new Date(data.endDate);
      return start <= end;
    }
    return true;
  },
  {
    message: "End date must be after start date",
    path: ["endDate"],
  }
);

export const skillSchema = z.object({
  id: z.number().optional(),
  name: z.string().min(1, "Skill name is required"),
  category: z.string().optional(),
  selfRating: z.number().min(1).max(5).optional(),
  priority: z.enum(["Low", "Medium", "High"]).optional(),
  yearsExperience: z.number().min(0).optional(),
  notes: z.string().optional(),
});

export const educationSchema = z.object({
  id: z.number().optional(),
  institution: z.string().min(1, "Institution is required"),
  degree: z.string().optional(),
  fieldOfStudy: z.string().optional(),
  startDate: z.string().optional(),
  endDate: z.string().optional(),
  grade: z.string().optional(),
  description: z.string().optional(),
}).refine(
  (data) => {
    // End date must be after start date if both provided
    if (data.startDate && data.endDate) {
      const start = new Date(data.startDate);
      const end = new Date(data.endDate);
      return start <= end;
    }
    return true;
  },
  {
    message: "End date must be after start date",
    path: ["endDate"],
  }
);

export const certificationSchema = z.object({
  id: z.number().optional(),
  name: z.string().min(1, "Certification name is required"),
  issuingOrganization: z.string().optional(),
  issueDate: z.string().optional(),
  expirationDate: z.string().optional(),
  credentialId: z.string().optional(),
  credentialUrl: z.string().url("Please enter a valid URL").optional().or(z.literal("")),
}).refine(
  (data) => {
    // Expiration date must be after issue date if both provided
    if (data.issueDate && data.expirationDate) {
      const issue = new Date(data.issueDate);
      const expiration = new Date(data.expirationDate);
      return issue <= expiration;
    }
    return true;
  },
  {
    message: "Expiration date must be after issue date",
    path: ["expirationDate"],
  }
);

export const portfolioItemSchema = z.object({
  id: z.number().optional(),
  title: z.string().min(1, "Portfolio item title is required"),
  url: z.string().url("Please enter a valid URL").optional().or(z.literal("")),
  description: z.string().optional(),
  role: z.string().optional(),
  techStack: z.string().optional(),
  highlighted: z.boolean().default(false),
});

export const userProfileDataSchema = z.object({
  profile: userProfileSchema.optional().nullable(),
  experience: z.array(experienceSchema),
  skills: z.array(skillSchema),
  education: z.array(educationSchema),
  certifications: z.array(certificationSchema),
  portfolio: z.array(portfolioItemSchema),
});

// ============================================================================
// Job Schemas
// ============================================================================

export const createJobInputSchema = z.object({
  title: z.string().optional(),
  company: z.string().optional(),
  location: z.string().optional(),
  jobSource: z.string().optional(),
  postingUrl: z.string().url("Please enter a valid URL").optional().or(z.literal("")),
  rawDescription: z.string().optional(),
}).refine(
  (data) => {
    // At least one of title, company, or rawDescription must be provided
    return !!(data.title || data.company || data.rawDescription);
  },
  {
    message: "At least one of title, company, or description must be provided",
  }
);

export const updateJobInputSchema = z.object({
  title: z.string().optional(),
  company: z.string().optional(),
  location: z.string().optional(),
  jobSource: z.string().optional(),
  postingUrl: z.string().url("Please enter a valid URL").optional().or(z.literal("")),
  rawDescription: z.string().optional(),
  isActive: z.boolean().optional(),
});

// ============================================================================
// Application Schemas
// ============================================================================

export const applicationStatusSchema = z.enum([
  "Saved",
  "Draft",
  "Applied",
  "Interviewing",
  "Offer",
  "Rejected",
  "Ghosted",
  "Withdrawn",
]);

export const applicationPrioritySchema = z.enum(["Low", "Medium", "High", "Dream"]);

export const createApplicationInputSchema = z.object({
  jobId: z.number().int().positive("Job ID is required"),
  status: applicationStatusSchema.optional(),
  channel: z.string().optional(),
  priority: applicationPrioritySchema.optional(),
});

export const updateApplicationInputSchema = z.object({
  status: applicationStatusSchema.optional(),
  channel: z.string().optional(),
  priority: applicationPrioritySchema.optional(),
  dateApplied: z.string().optional(),
  nextActionDate: z.string().optional(),
  nextActionNote: z.string().optional(),
  notesSummary: z.string().optional(),
  contactName: z.string().optional(),
  contactEmail: z.string().email("Please enter a valid email address").optional().or(z.literal("")),
  contactLinkedin: z.string().url("Please enter a valid URL").optional().or(z.literal("")),
  locationOverride: z.string().optional(),
  offerCompensation: z.string().optional(),
});

export const addEventInputSchema = z.object({
  applicationId: z.number().int().positive("Application ID is required"),
  eventType: z.string().min(1, "Event type is required"),
  eventDate: z.string().min(1, "Event date is required"),
  fromStatus: z.string().optional(),
  toStatus: z.string().optional(),
  title: z.string().optional(),
  details: z.string().optional(),
});

// ============================================================================
// AI Settings Schema
// ============================================================================

export const aiSettingsSchema = z.object({
  mode: z.enum(["local", "cloud", "hybrid"]),
  cloudProvider: z.enum(["openai", "anthropic"]).optional(),
  apiKey: z.string().optional(),
  modelName: z.string().optional(),
  localModelPath: z.string().optional(),
}).refine(
  (data) => {
    // If cloud mode, API key is required
    if (data.mode === "cloud" && !data.apiKey) {
      return false;
    }
    // If local mode, local model path is required
    if (data.mode === "local" && !data.localModelPath) {
      return false;
    }
    return true;
  },
  {
    message: "Cloud mode requires an API key, and local mode requires a model path",
  }
);

// ============================================================================
// Type Exports (inferred from schemas)
// ============================================================================

export type UserProfileInput = z.infer<typeof userProfileSchema>;
export type ExperienceInput = z.infer<typeof experienceSchema>;
export type SkillInput = z.infer<typeof skillSchema>;
export type EducationInput = z.infer<typeof educationSchema>;
export type CertificationInput = z.infer<typeof certificationSchema>;
export type PortfolioItemInput = z.infer<typeof portfolioItemSchema>;
export type UserProfileDataInput = z.infer<typeof userProfileDataSchema>;
export type CreateJobInput = z.infer<typeof createJobInputSchema>;
export type UpdateJobInput = z.infer<typeof updateJobInputSchema>;
export type CreateApplicationInput = z.infer<typeof createApplicationInputSchema>;
export type UpdateApplicationInput = z.infer<typeof updateApplicationInputSchema>;
export type AddEventInput = z.infer<typeof addEventInputSchema>;
export type AiSettingsInput = z.infer<typeof aiSettingsSchema>;

