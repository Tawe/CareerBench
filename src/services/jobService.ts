import { invoke } from '@tauri-apps/api/core';
import type { Job, JobSummary, CreateJobInput, UpdateJobInput, ParsedJob } from '../commands/jobs';
import { ServiceError } from './dashboardService';

export class JobService {
  static async createJob(input: CreateJobInput): Promise<Job> {
    try {
      return await invoke<Job>('create_job', { input });
    } catch (error) {
      throw new ServiceError('Failed to create job', error);
    }
  }

  static async updateJob(id: number, input: UpdateJobInput): Promise<Job> {
    try {
      return await invoke<Job>('update_job', { id, input });
    } catch (error) {
      throw new ServiceError('Failed to update job', error);
    }
  }

  static async getJobList(options?: { search?: string | null; activeOnly?: boolean; source?: string | null }): Promise<JobSummary[]> {
    try {
      return await invoke<JobSummary[]>('get_job_list', { options });
    } catch (error) {
      throw new ServiceError('Failed to fetch job list', error);
    }
  }

  static async getJobDetail(id: number): Promise<Job> {
    try {
      return await invoke<Job>('get_job_detail', { id });
    } catch (error) {
      throw new ServiceError('Failed to fetch job details', error);
    }
  }

  static async parseJobWithAI(jobId: number): Promise<ParsedJob> {
    try {
      return await invoke<ParsedJob>('parse_job_with_ai', { jobId });
    } catch (error) {
      throw new ServiceError('Failed to parse job with AI', error);
    }
  }
}