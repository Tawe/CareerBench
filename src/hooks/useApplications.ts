import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { ApplicationStatus } from '../utils/applicationUtils';
import type { ApplicationSummary } from '../utils/applicationUtils';

export interface ApplicationDetail {
  application: {
    id: number;
    job_id: number;
    status: ApplicationStatus;
    // ... other fields
  };
  job: {
    id: number;
    title?: string;
    company?: string;
    // ... other fields
  };
}

export interface Job {
  id?: number;
  title?: string;
  company?: string;
}

interface UseApplicationsOptions {
  status?: ApplicationStatus | 'all';
  autoLoad?: boolean;
}

/**
 * Custom React hook for fetching and managing application data.
 * 
 * Supports filtering by status and automatically loads both applications
 * and available jobs for creating new applications.
 * 
 * @param options - Configuration options for the hook
 * @param options.status - Filter applications by status, or "all" for all statuses (default: "all")
 * @param options.autoLoad - Whether to automatically load data on mount (default: true)
 * @returns Object containing:
 *   - `applications`: Array of application summaries matching the filter
 *   - `isLoading`: Boolean indicating if data is currently being fetched
 *   - `error`: Error message string, or null if no error
 *   - `availableJobs`: Array of jobs available for creating new applications
 *   - `reload`: Function to manually reload applications
 *   - `reloadJobs`: Function to manually reload available jobs
 * 
 * @example
 * ```typescript
 * function ApplicationsPage() {
 *   const { applications, isLoading, error, reload } = useApplications({ status: "Applied" });
 *   // Returns only applications with "Applied" status
 * }
 * ```
 */
export function useApplications(options: UseApplicationsOptions = {}) {
  const { status = 'all', autoLoad = true } = options;
  
  const [applications, setApplications] = useState<ApplicationSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [availableJobs, setAvailableJobs] = useState<Job[]>([]);

  const loadApplications = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<ApplicationSummary[]>("get_applications", {
        status: status === "all" ? null : status,
        jobId: null,
        activeOnly: true,
      });
      setApplications(result);
    } catch (err: any) {
      setError(err?.message || "Failed to load applications");
    } finally {
      setIsLoading(false);
    }
  }, [status]);

  const loadAvailableJobs = useCallback(async () => {
    try {
      const jobs = await invoke<Job[]>("get_job_list", {
        search: null,
        activeOnly: true,
        source: null,
      });
      setAvailableJobs(jobs);
    } catch (err) {
      // Ignore errors for now
    }
  }, []);

  useEffect(() => {
    if (autoLoad) {
      loadApplications();
      loadAvailableJobs();
    }
  }, [loadApplications, loadAvailableJobs, autoLoad]);

  return {
    applications,
    isLoading,
    error,
    availableJobs,
    reload: loadApplications,
    reloadJobs: loadAvailableJobs,
  };
}

