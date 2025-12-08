import { describe, it, expect } from 'vitest';
import {
  filterApplicationsByStatus,
  groupApplicationsByStatus,
  getStatusLabelWithCount,
  getStatusLabel,
  sortApplicationsByDate,
  sortApplicationsByPriority,
  filterActiveApplications,
  getActiveApplicationsCount,
  type ApplicationSummary,
  type ApplicationStatus,
} from '../utils/applicationUtils';

describe('Application Utilities', () => {
  const mockApplications: ApplicationSummary[] = [
    {
      id: 1,
      job_id: 1,
      status: 'Applied',
      job_title: 'Software Engineer',
      company: 'Tech Corp',
      priority: 'High',
      date_applied: '2024-01-15',
      archived: false,
    },
    {
      id: 2,
      job_id: 2,
      status: 'Interviewing',
      job_title: 'Senior Engineer',
      company: 'Startup Inc',
      priority: 'Dream',
      date_applied: '2024-01-20',
      archived: false,
    },
    {
      id: 3,
      job_id: 3,
      status: 'Applied',
      job_title: 'Backend Developer',
      company: 'Big Tech',
      priority: 'Medium',
      date_applied: '2024-01-10',
      archived: true,
    },
    {
      id: 4,
      job_id: 4,
      status: 'Rejected',
      job_title: 'Frontend Dev',
      company: 'Small Co',
      priority: 'Low',
      date_applied: '2024-01-05',
      archived: false,
    },
  ];

  describe('filterApplicationsByStatus', () => {
    it('filters applications by specific status', () => {
      const result = filterApplicationsByStatus(mockApplications, 'Applied');
      expect(result).toHaveLength(2);
      expect(result.every((app) => app.status === 'Applied')).toBe(true);
    });

    it('returns all applications when status is "all"', () => {
      const result = filterApplicationsByStatus(mockApplications, 'all');
      expect(result).toHaveLength(4);
    });

    it('returns empty array when no applications match status', () => {
      const result = filterApplicationsByStatus(mockApplications, 'Offer');
      expect(result).toHaveLength(0);
    });
  });

  describe('groupApplicationsByStatus', () => {
    it('groups applications by status', () => {
      const result = groupApplicationsByStatus(mockApplications);
      
      expect(result['Applied']).toHaveLength(2);
      expect(result['Interviewing']).toHaveLength(1);
      expect(result['Rejected']).toHaveLength(1);
      expect(result['Saved']).toHaveLength(0);
    });

    it('includes all statuses even with zero applications', () => {
      const result = groupApplicationsByStatus(mockApplications);
      
      const statuses: ApplicationStatus[] = [
        'Saved',
        'Draft',
        'Applied',
        'Interviewing',
        'Offer',
        'Rejected',
        'Ghosted',
        'Withdrawn',
      ];
      
      statuses.forEach((status) => {
        expect(result).toHaveProperty(status);
        expect(Array.isArray(result[status])).toBe(true);
      });
    });
  });

  describe('getStatusLabelWithCount', () => {
    it('formats status label with count', () => {
      expect(getStatusLabelWithCount('Applied', 5)).toBe('Applied (5)');
      expect(getStatusLabelWithCount('Interviewing', 0)).toBe('Interviewing (0)');
    });
  });

  describe('getStatusLabel', () => {
    it('returns the status as-is', () => {
      expect(getStatusLabel('Applied')).toBe('Applied');
      expect(getStatusLabel('Interviewing')).toBe('Interviewing');
    });
  });

  describe('sortApplicationsByDate', () => {
    it('sorts applications by date (most recent first)', () => {
      const result = sortApplicationsByDate(mockApplications);
      
      expect(result[0].date_applied).toBe('2024-01-20');
      expect(result[1].date_applied).toBe('2024-01-15');
      expect(result[2].date_applied).toBe('2024-01-10');
      expect(result[3].date_applied).toBe('2024-01-05');
    });

    it('handles applications without dates (puts them last)', () => {
      const appsWithoutDates: ApplicationSummary[] = [
        { id: 1, job_id: 1, status: 'Saved', date_applied: '2024-01-15' },
        { id: 2, job_id: 2, status: 'Saved' }, // No date
        { id: 3, job_id: 3, status: 'Saved', date_applied: '2024-01-20' },
      ];
      
      const result = sortApplicationsByDate(appsWithoutDates);
      expect(result[0].date_applied).toBe('2024-01-20');
      expect(result[1].date_applied).toBe('2024-01-15');
      expect(result[2].date_applied).toBeUndefined();
    });
  });

  describe('sortApplicationsByPriority', () => {
    it('sorts applications by priority (highest first)', () => {
      const result = sortApplicationsByPriority(mockApplications);
      
      expect(result[0].priority).toBe('Dream');
      expect(result[1].priority).toBe('High');
      expect(result[2].priority).toBe('Medium');
      expect(result[3].priority).toBe('Low');
    });

    it('handles applications without priority (puts them last)', () => {
      const appsWithoutPriority: ApplicationSummary[] = [
        { id: 1, job_id: 1, status: 'Saved', priority: 'High' },
        { id: 2, job_id: 2, status: 'Saved' }, // No priority
        { id: 3, job_id: 3, status: 'Saved', priority: 'Dream' },
      ];
      
      const result = sortApplicationsByPriority(appsWithoutPriority);
      expect(result[0].priority).toBe('Dream');
      expect(result[1].priority).toBe('High');
      expect(result[2].priority).toBeUndefined();
    });
  });

  describe('filterActiveApplications', () => {
    it('filters out archived applications', () => {
      const result = filterActiveApplications(mockApplications);
      
      expect(result).toHaveLength(3);
      expect(result.every((app) => !app.archived)).toBe(true);
    });

    it('returns all applications when none are archived', () => {
      const activeOnly = mockApplications.filter((app) => !app.archived);
      const result = filterActiveApplications(activeOnly);
      
      expect(result).toHaveLength(activeOnly.length);
    });
  });

  describe('getActiveApplicationsCount', () => {
    it('returns count of active (non-archived) applications', () => {
      expect(getActiveApplicationsCount(mockApplications)).toBe(3);
    });

    it('returns 0 when all applications are archived', () => {
      const allArchived = mockApplications.map((app) => ({ ...app, archived: true }));
      expect(getActiveApplicationsCount(allArchived)).toBe(0);
    });

    it('returns total count when no applications are archived', () => {
      const allActive = mockApplications.map((app) => ({ ...app, archived: false }));
      expect(getActiveApplicationsCount(allActive)).toBe(4);
    });
  });
});

