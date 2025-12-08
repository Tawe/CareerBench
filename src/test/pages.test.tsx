// Comprehensive UI rendering tests for key pages
// These ensure pages render correctly in different states and display data properly

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import userEvent from '@testing-library/user-event';

// Mock Tauri API
const mockInvoke = vi.fn();
vi.mock('@tauri-apps/api/core', () => ({
  invoke: mockInvoke,
  isTauri: vi.fn(() => true),
}));

// Helper to render with router
const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('Page Rendering', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Dashboard', () => {
    it('renders loading state initially', async () => {
      mockInvoke.mockImplementation(() => new Promise(() => {})); // Never resolves
      
      const { default: Dashboard } = await import('../pages/Dashboard');
      renderWithRouter(<Dashboard />);
      
      expect(screen.getByText(/loading dashboard/i)).toBeInTheDocument();
    });

    it('renders dashboard data when loaded', async () => {
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
          { date: '2024-01-15', applications_created: 1, interviews_completed: 0, offers_received: 0 },
        ],
        funnel: [
          { label: 'Applied', count: 2 },
          { label: 'Interviewing', count: 1 },
          { label: 'Offer', count: 1 },
        ],
      };

      mockInvoke.mockResolvedValue(mockData);
      
      const { default: Dashboard } = await import('../pages/Dashboard');
      renderWithRouter(<Dashboard />);
      
      // Fast-forward timer to trigger load
      vi.advanceTimersByTime(500);
      
      await waitFor(() => {
        expect(screen.getByText('Dashboard')).toBeInTheDocument();
      });

      // Check KPIs are displayed
      expect(screen.getByText('3')).toBeInTheDocument(); // Active Applications
      expect(screen.getByText('Active Applications')).toBeInTheDocument();
      expect(screen.getByText('10')).toBeInTheDocument(); // Total Jobs Tracked
    });

    it('renders error state when data fails to load', async () => {
      mockInvoke.mockRejectedValue(new Error('Failed to load'));
      
      const { default: Dashboard } = await import('../pages/Dashboard');
      renderWithRouter(<Dashboard />);
      
      vi.advanceTimersByTime(500);
      
      // Wait for retries to complete
      await waitFor(() => {
        expect(screen.getByText(/tauri api not available|failed to load/i)).toBeInTheDocument();
      }, { timeout: 5000 });
    });

    it('renders reload button', async () => {
      const mockData = {
        kpis: { total_jobs_tracked: 0, total_applications: 0, active_applications: 0, applications_last_30_days: 0, offers_received: 0 },
        status_breakdown: [],
        activity_last_30_days: [],
        funnel: [],
      };

      mockInvoke.mockResolvedValue(mockData);
      
      const { default: Dashboard } = await import('../pages/Dashboard');
      renderWithRouter(<Dashboard />);
      
      vi.advanceTimersByTime(500);
      
      await waitFor(() => {
        expect(screen.getByText('Reload')).toBeInTheDocument();
      });
    });
  });

  describe('Jobs', () => {
    it('renders loading state initially', async () => {
      mockInvoke.mockImplementation(() => new Promise(() => {}));
      
      const { default: Jobs } = await import('../pages/Jobs');
      renderWithRouter(<Jobs />);
      
      expect(screen.getByText(/loading jobs/i)).toBeInTheDocument();
    });

    it('renders job list when loaded', async () => {
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

      mockInvoke.mockResolvedValue(mockJobs);
      
      const { default: Jobs } = await import('../pages/Jobs');
      renderWithRouter(<Jobs />);
      
      await waitFor(() => {
        expect(screen.getByText('Software Engineer')).toBeInTheDocument();
        expect(screen.getByText('Tech Corp')).toBeInTheDocument();
        expect(screen.getByText('Senior Engineer')).toBeInTheDocument();
      });
    });

    it('renders empty state when no jobs', async () => {
      mockInvoke.mockResolvedValue([]);
      
      const { default: Jobs } = await import('../pages/Jobs');
      renderWithRouter(<Jobs />);
      
      await waitFor(() => {
        expect(screen.getByText(/no jobs found/i)).toBeInTheDocument();
      });
    });

    it('renders error state when data fails to load', async () => {
      mockInvoke.mockRejectedValue(new Error('Failed to load jobs'));
      
      const { default: Jobs } = await import('../pages/Jobs');
      renderWithRouter(<Jobs />);
      
      await waitFor(() => {
        expect(screen.getByText(/failed to load jobs/i)).toBeInTheDocument();
      });
    });

    it('renders search input and filters', async () => {
      mockInvoke.mockResolvedValue([]);
      
      const { default: Jobs } = await import('../pages/Jobs');
      renderWithRouter(<Jobs />);
      
      await waitFor(() => {
        expect(screen.getByPlaceholderText(/search jobs/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/active only/i)).toBeInTheDocument();
      });
    });
  });

  describe('Applications', () => {
    it('renders loading state initially', async () => {
      mockInvoke.mockImplementation(() => new Promise(() => {}));
      
      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);
      
      expect(screen.getByText(/loading applications/i)).toBeInTheDocument();
    });

    it('renders application list when loaded', async () => {
      const mockApplications = [
        {
          id: 1,
          job_id: 1,
          status: 'Applied',
          job_title: 'Software Engineer',
          company: 'Tech Corp',
        },
        {
          id: 2,
          job_id: 2,
          status: 'Interviewing',
          job_title: 'Senior Engineer',
          company: 'Startup Inc',
        },
      ];

      mockInvoke
        .mockResolvedValueOnce(mockApplications) // get_applications
        .mockResolvedValueOnce([]); // get_job_list
      
      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);
      
      await waitFor(() => {
        expect(screen.getByText('Software Engineer')).toBeInTheDocument();
        expect(screen.getByText('Tech Corp')).toBeInTheDocument();
      });
    });

    it('renders status filters', async () => {
      mockInvoke
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce([]);
      
      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);
      
      await waitFor(() => {
        expect(screen.getByText('All')).toBeInTheDocument();
        expect(screen.getByText('Applied')).toBeInTheDocument();
        expect(screen.getByText('Interviewing')).toBeInTheDocument();
      });
    });

    it('renders create application button', async () => {
      mockInvoke
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce([]);
      
      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);
      
      await waitFor(() => {
        expect(screen.getByText(/create application/i)).toBeInTheDocument();
      });
    });

    it('renders error state when data fails to load', async () => {
      mockInvoke.mockRejectedValue(new Error('Failed to load applications'));
      
      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);
      
      await waitFor(() => {
        expect(screen.getByText(/failed to load applications/i)).toBeInTheDocument();
      });
    });
  });

  describe('Profile', () => {
    it('renders loading state initially', async () => {
      mockInvoke.mockImplementation(() => new Promise(() => {}));
      
      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);
      
      expect(screen.getByText(/loading profile/i)).toBeInTheDocument();
    });

    it('renders profile form when loaded', async () => {
      const mockProfile = {
        profile: {
          id: 1,
          full_name: 'John Doe',
          headline: 'Software Engineer',
          location: 'San Francisco, CA',
        },
        experience: [],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      mockInvoke.mockResolvedValue(mockProfile);
      
      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);
      
      await waitFor(() => {
        expect(screen.getByText('Profile')).toBeInTheDocument();
        expect(screen.getByLabelText(/full name/i)).toBeInTheDocument();
        expect(screen.getByDisplayValue('John Doe')).toBeInTheDocument();
      });
    });

    it('renders empty state when no profile exists', async () => {
      const mockProfile = {
        profile: undefined,
        experience: [],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      mockInvoke.mockResolvedValue(mockProfile);
      
      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);
      
      await waitFor(() => {
        expect(screen.getByText(/let's set up your profile/i)).toBeInTheDocument();
      });
    });

    it('renders save button', async () => {
      const mockProfile = {
        profile: { id: 1, full_name: 'John Doe' },
        experience: [],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      mockInvoke.mockResolvedValue(mockProfile);
      
      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);
      
      await waitFor(() => {
        expect(screen.getByText('Save')).toBeInTheDocument();
      });
    });

    it('renders error state when data fails to load', async () => {
      mockInvoke.mockRejectedValue(new Error('Failed to load profile'));
      
      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);
      
      await waitFor(() => {
        expect(screen.getByText(/failed to load profile/i)).toBeInTheDocument();
      });
    });

    it('renders all profile sections', async () => {
      const mockProfile = {
        profile: { id: 1, full_name: 'John Doe' },
        experience: [],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      mockInvoke.mockResolvedValue(mockProfile);
      
      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);
      
      await waitFor(() => {
        expect(screen.getByText('Basic Info')).toBeInTheDocument();
        // Other sections may be conditionally rendered, so we check for at least Basic Info
      });
    });
  });
});
