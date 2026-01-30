/**
 * Jobs command types
 */

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

export interface JobCommands {
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
}