/**
 * AI and Resume Generation command types
 */

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

export interface AiCommands {
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
}