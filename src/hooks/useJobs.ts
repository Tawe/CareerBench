import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

export interface JobSummary {
  id: number;
  title?: string;
  company?: string;
  location?: string;
  seniority?: string;
  date_added: string;
}

export interface Job {
  id?: number;
  title?: string;
  company?: string;
  location?: string;
  raw_description?: string;
  parsed_json?: string;
  // ... other fields
}

interface UseJobsOptions {
  search?: string;
  activeOnly?: boolean;
  autoLoad?: boolean;
}

/**
 * Custom React hook for fetching and managing job listings.
 * 
 * Supports search filtering and active-only filtering. Automatically
 * reloads when search or activeOnly parameters change.
 * 
 * @param options - Configuration options for the hook
 * @param options.search - Search query string to filter jobs by title/company (default: "")
 * @param options.activeOnly - Whether to show only active jobs (default: true)
 * @param options.autoLoad - Whether to automatically load data on mount (default: true)
 * @returns Object containing:
 *   - `jobs`: Array of job summaries matching the filters
 *   - `isLoading`: Boolean indicating if data is currently being fetched
 *   - `error`: Error message string, or null if no error
 *   - `reload`: Function to manually reload jobs
 * 
 * @example
 * ```typescript
 * function JobsPage() {
 *   const [search, setSearch] = useState("");
 *   const { jobs, isLoading, reload } = useJobs({ search, activeOnly: true });
 *   // Automatically reloads when search changes
 * }
 * ```
 */
export function useJobs(options: UseJobsOptions = {}) {
  const { search = '', activeOnly = true, autoLoad = true } = options;
  
  const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadJobs = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<JobSummary[]>("get_job_list", {
        search: search || null,
        activeOnly: activeOnly,
        source: null,
      });
      setJobs(result);
    } catch (err: any) {
      setError(err?.message || "Failed to load jobs");
    } finally {
      setIsLoading(false);
    }
  }, [search, activeOnly]);

  useEffect(() => {
    if (autoLoad) {
      loadJobs();
    }
  }, [loadJobs, autoLoad]);

  return {
    jobs,
    isLoading,
    error,
    reload: loadJobs,
  };
}

