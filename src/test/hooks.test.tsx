import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { useDashboardData } from '../hooks/useDashboardData';
import { useApplications } from '../hooks/useApplications';
import { useJobs } from '../hooks/useJobs';

// Mock Tauri API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

import { invoke } from '@tauri-apps/api/core';

describe('Hooks', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('useDashboardData', () => {
    it('loads dashboard data successfully', async () => {
      const mockData = {
        kpis: {
          total_jobs_tracked: 10,
          total_applications: 5,
          active_applications: 3,
          applications_last_30_days: 2,
          offers_received: 1,
        },
        status_breakdown: [
          { status: 'Applied', count: 2 },
          { status: 'Interviewing', count: 1 },
        ],
        activity_last_30_days: [
          { date: '2024-01-15', count: 1 },
          { date: '2024-01-20', count: 1 },
        ],
        funnel: [
          { label: 'Applied', count: 2 },
          { label: 'Interviewing', count: 1 },
          { label: 'Offer', count: 1 },
        ],
      };

      vi.mocked(invoke).mockResolvedValue(mockData);

      const { result } = renderHook(() => useDashboardData());

      expect(result.current.isLoading).toBe(true);
      expect(result.current.data).toBeNull();

      // Fast-forward timers to trigger the initial load
      vi.advanceTimersByTime(500);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.data).toEqual(mockData);
      expect(result.current.error).toBeNull();
    });

    it('handles errors gracefully', async () => {
      const errorMessage = 'Failed to load dashboard';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useDashboardData());

      vi.advanceTimersByTime(500);

      // Wait for retries to complete (3 attempts)
      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      }, { timeout: 5000 });

      expect(result.current.data).toBeNull();
      expect(result.current.error).toBeTruthy();
    });

    it('provides reload function', async () => {
      const mockData = {
        kpis: {
          total_jobs_tracked: 10,
          total_applications: 5,
          active_applications: 3,
          applications_last_30_days: 2,
          offers_received: 1,
        },
        status_breakdown: [],
        activity_last_30_days: [],
        funnel: [],
      };

      vi.mocked(invoke).mockResolvedValue(mockData);

      const { result } = renderHook(() => useDashboardData());

      vi.advanceTimersByTime(500);

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      // Reload
      result.current.reload();

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(vi.mocked(invoke)).toHaveBeenCalledTimes(2);
    });
  });

  describe('useApplications', () => {
    it('loads applications successfully', async () => {
      const mockApplications = [
        {
          id: 1,
          job_id: 1,
          status: 'Applied' as const,
          job_title: 'Software Engineer',
          company: 'Tech Corp',
        },
        {
          id: 2,
          job_id: 2,
          status: 'Interviewing' as const,
          job_title: 'Senior Engineer',
          company: 'Startup Inc',
        },
      ];

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockApplications) // get_applications
        .mockResolvedValueOnce([]); // get_job_list

      const { result } = renderHook(() => useApplications({ status: 'all' }));

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.applications).toEqual(mockApplications);
      expect(result.current.error).toBeNull();
    });

    it('filters applications by status', async () => {
      const mockApplications = [
        {
          id: 1,
          job_id: 1,
          status: 'Applied' as const,
          job_title: 'Software Engineer',
          company: 'Tech Corp',
        },
      ];

      vi.mocked(invoke)
        .mockResolvedValueOnce(mockApplications)
        .mockResolvedValueOnce([]);

      const { result } = renderHook(() => useApplications({ status: 'Applied' }));

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(vi.mocked(invoke)).toHaveBeenCalledWith('get_applications', {
        status: 'Applied',
        jobId: null,
        activeOnly: true,
      });
    });

    it('handles errors gracefully', async () => {
      const errorMessage = 'Failed to load applications';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useApplications());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.applications).toEqual([]);
      expect(result.current.error).toBe(errorMessage);
    });

    it('supports manual loading with autoLoad=false', () => {
      vi.mocked(invoke).mockResolvedValue([]);

      const { result } = renderHook(() => useApplications({ autoLoad: false }));

      expect(result.current.isLoading).toBe(true);
      expect(vi.mocked(invoke)).not.toHaveBeenCalled();

      result.current.reload();

      expect(vi.mocked(invoke)).toHaveBeenCalled();
    });
  });

  describe('useJobs', () => {
    it('loads jobs successfully', async () => {
      const mockJobs = [
        {
          id: 1,
          title: 'Software Engineer',
          company: 'Tech Corp',
          location: 'San Francisco, CA',
          date_added: '2024-01-15',
        },
        {
          id: 2,
          title: 'Senior Engineer',
          company: 'Startup Inc',
          location: 'Remote',
          date_added: '2024-01-20',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockJobs);

      const { result } = renderHook(() => useJobs());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.jobs).toEqual(mockJobs);
      expect(result.current.error).toBeNull();
    });

    it('filters jobs by search term', async () => {
      const mockJobs = [
        {
          id: 1,
          title: 'Software Engineer',
          company: 'Tech Corp',
          date_added: '2024-01-15',
        },
      ];

      vi.mocked(invoke).mockResolvedValue(mockJobs);

      const { result } = renderHook(() => useJobs({ search: 'Engineer' }));

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(vi.mocked(invoke)).toHaveBeenCalledWith('get_job_list', {
        search: 'Engineer',
        activeOnly: true,
        source: null,
      });
    });

    it('handles errors gracefully', async () => {
      const errorMessage = 'Failed to load jobs';
      vi.mocked(invoke).mockRejectedValue(new Error(errorMessage));

      const { result } = renderHook(() => useJobs());

      await waitFor(() => {
        expect(result.current.isLoading).toBe(false);
      });

      expect(result.current.jobs).toEqual([]);
      expect(result.current.error).toBe(errorMessage);
    });

    it('supports manual loading with autoLoad=false', () => {
      vi.mocked(invoke).mockResolvedValue([]);

      const { result } = renderHook(() => useJobs({ autoLoad: false }));

      expect(result.current.isLoading).toBe(true);
      expect(vi.mocked(invoke)).not.toHaveBeenCalled();

      result.current.reload();

      expect(vi.mocked(invoke)).toHaveBeenCalled();
    });
  });
});

