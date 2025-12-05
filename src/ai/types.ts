// AI Provider Types
// These types match the Rust types in src-tauri/src/ai/types.rs

export type AiMode = "local" | "cloud" | "hybrid";
export type CloudProvider = "openai" | "anthropic";

export interface AiSettings {
  mode: AiMode;
  cloudProvider?: CloudProvider;
  apiKey?: string;
  modelName?: string;
  localModelPath?: string;
}

export interface ResumeInput {
  profileData: any; // User profile JSON
  jobDescription: string;
  options?: ResumeOptions;
}

export interface ResumeOptions {
  tone?: string;
  length?: string;
  focus?: string;
}

export interface ResumeSuggestions {
  summary?: string;
  headline?: string;
  sections: ResumeSection[];
  highlights: string[];
}

export interface ResumeSection {
  title: string;
  items: ResumeSectionItem[];
}

export interface ResumeSectionItem {
  heading: string;
  subheading?: string;
  bullets: string[];
}

export interface CoverLetterInput {
  profileData: any;
  jobDescription: string;
  companyName?: string;
  options?: CoverLetterOptions;
}

export interface CoverLetterOptions {
  tone?: string;
  length?: string;
  audience?: string;
}

export interface CoverLetter {
  subject?: string;
  greeting?: string;
  bodyParagraphs: string[];
  closing?: string;
  signature?: string;
}

export interface SkillSuggestionsInput {
  currentSkills: string[];
  jobDescription: string;
  experience?: any;
}

export interface SkillSuggestions {
  missingSkills: string[];
  skillGaps: SkillGap[];
  recommendations: string[];
}

export interface SkillGap {
  skill: string;
  importance: "high" | "medium" | "low";
  reason: string;
}

