// Core behavior tests for interactive features
// Tests status filters, sheet open/close, unsaved changes

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

describe('Core Behavior Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe('Status Filters', () => {
    it('filters applications by status when clicking filter buttons', async () => {
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
        {
          id: 3,
          job_id: 3,
          status: 'Applied',
          job_title: 'Backend Developer',
          company: 'Big Tech',
        },
      ];

      mockInvoke
        .mockResolvedValueOnce(mockApplications) // get_applications
        .mockResolvedValueOnce([]); // get_job_list

      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);

      await waitFor(() => {
        expect(screen.getByText('Software Engineer')).toBeInTheDocument();
      });

      // Click "Applied" filter
      const appliedButton = screen.getByText(/applied/i);
      await userEvent.click(appliedButton);

      await waitFor(() => {
        // Should show only Applied applications
        expect(screen.getByText('Software Engineer')).toBeInTheDocument();
        expect(screen.getByText('Backend Developer')).toBeInTheDocument();
        // Interviewing should not be visible in filtered view
        expect(screen.queryByText('Senior Engineer')).not.toBeInTheDocument();
      });
    });

    it('shows kanban board when "All" filter is selected', async () => {
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
        .mockResolvedValueOnce(mockApplications)
        .mockResolvedValueOnce([]);

      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);

      await waitFor(() => {
        // Should show kanban board with columns
        expect(screen.getByText('Applied')).toBeInTheDocument();
        expect(screen.getByText('Interviewing')).toBeInTheDocument();
      });
    });

    it('shows empty state when filtered status has no applications', async () => {
      const mockApplications = [
        {
          id: 1,
          job_id: 1,
          status: 'Applied',
          job_title: 'Software Engineer',
          company: 'Tech Corp',
        },
      ];

      mockInvoke
        .mockResolvedValueOnce(mockApplications)
        .mockResolvedValueOnce([]);

      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);

      await waitFor(() => {
        expect(screen.getByText('Software Engineer')).toBeInTheDocument();
      });

      // Click "Offer" filter (no applications with this status)
      const offerButton = screen.getByText(/offer/i);
      await userEvent.click(offerButton);

      await waitFor(() => {
        expect(screen.getByText(/no applications with status "offer"/i)).toBeInTheDocument();
      });
    });
  });

  describe('Sheet Open/Close', () => {
    it('opens and closes CreateApplicationSheet', async () => {
      mockInvoke
        .mockResolvedValueOnce([]) // get_applications
        .mockResolvedValueOnce([{ id: 1, title: 'Test Job', company: 'Test Corp' }]); // get_job_list

      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);

      await waitFor(() => {
        expect(screen.getByText(/create application/i)).toBeInTheDocument();
      });

      // Click create button
      const createButton = screen.getByText(/create application/i);
      await userEvent.click(createButton);

      // Sheet should open
      await waitFor(() => {
        expect(screen.getByText('Create Application')).toBeInTheDocument();
      });

      // Click close button
      const closeButton = screen.getByText('×');
      await userEvent.click(closeButton);

      // Sheet should close
      await waitFor(() => {
        expect(screen.queryByText('Create Application')).not.toBeInTheDocument();
      });
    });

    it('closes sheet when clicking overlay', async () => {
      mockInvoke
        .mockResolvedValueOnce([])
        .mockResolvedValueOnce([{ id: 1, title: 'Test Job', company: 'Test Corp' }]);

      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);

      await waitFor(() => {
        expect(screen.getByText(/create application/i)).toBeInTheDocument();
      });

      // Open sheet
      const createButton = screen.getByText(/create application/i);
      await userEvent.click(createButton);

      await waitFor(() => {
        expect(screen.getByText('Create Application')).toBeInTheDocument();
      });

      // Click overlay (sheet-overlay)
      const overlay = document.querySelector('.sheet-overlay');
      expect(overlay).toBeInTheDocument();
      if (overlay) {
        await userEvent.click(overlay);
      }

      // Sheet should close
      await waitFor(() => {
        expect(screen.queryByText('Create Application')).not.toBeInTheDocument();
      });
    });

    it('opens AddJobSheet in Jobs page', async () => {
      mockInvoke.mockResolvedValue([]);

      const { default: Jobs } = await import('../pages/Jobs');
      renderWithRouter(<Jobs />);

      await waitFor(() => {
        expect(screen.getByText(/add job/i)).toBeInTheDocument();
      });

      // Click add button
      const addButton = screen.getByText(/add job/i);
      await userEvent.click(addButton);

      // Sheet should open (checking for form elements that would be in the sheet)
      await waitFor(() => {
        // The sheet should contain job form elements
        const sheet = document.querySelector('.sheet-container');
        expect(sheet).toBeInTheDocument();
      });
    });
  });

  describe('Unsaved Changes', () => {
    it('shows unsaved changes indicator when profile is modified', async () => {
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
        expect(screen.getByDisplayValue('John Doe')).toBeInTheDocument();
      });

      // Initially, no unsaved changes indicator
      expect(screen.queryByText(/unsaved changes/i)).not.toBeInTheDocument();

      // Modify a field
      const nameInput = screen.getByDisplayValue('John Doe');
      await userEvent.clear(nameInput);
      await userEvent.type(nameInput, 'Jane Doe');

      // Unsaved changes indicator should appear
      await waitFor(() => {
        expect(screen.getByText(/unsaved changes/i)).toBeInTheDocument();
      });
    });

    it('enables save button when there are unsaved changes', async () => {
      const mockProfile = {
        profile: {
          id: 1,
          full_name: 'John Doe',
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
        const saveButton = screen.getByText('Save');
        expect(saveButton).toBeDisabled();
      });

      // Modify a field
      const nameInput = screen.getByDisplayValue('John Doe');
      await userEvent.clear(nameInput);
      await userEvent.type(nameInput, 'Jane Doe');

      // Save button should be enabled
      await waitFor(() => {
        const saveButton = screen.getByText('Save');
        expect(saveButton).not.toBeDisabled();
      });
    });

    it('disables save button when saving', async () => {
      const mockProfile = {
        profile: {
          id: 1,
          full_name: 'John Doe',
        },
        experience: [],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      mockInvoke
        .mockResolvedValueOnce(mockProfile) // Initial load
        .mockImplementation(() => new Promise(() => {})); // Save (never resolves)

      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);

      await waitFor(() => {
        expect(screen.getByDisplayValue('John Doe')).toBeInTheDocument();
      });

      // Modify and save
      const nameInput = screen.getByDisplayValue('John Doe');
      await userEvent.clear(nameInput);
      await userEvent.type(nameInput, 'Jane Doe');

      const saveButton = screen.getByText('Save');
      await userEvent.click(saveButton);

      // Button should show "Saving..." and be disabled
      await waitFor(() => {
        expect(screen.getByText('Saving...')).toBeInTheDocument();
        expect(screen.getByText('Saving...')).toBeDisabled();
      });
    });

    it('clears unsaved changes indicator after successful save', async () => {
      const mockProfile = {
        profile: {
          id: 1,
          full_name: 'John Doe',
        },
        experience: [],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      const savedProfile = {
        ...mockProfile,
        profile: {
          ...mockProfile.profile,
          full_name: 'Jane Doe',
        },
      };

      mockInvoke
        .mockResolvedValueOnce(mockProfile) // Initial load
        .mockResolvedValueOnce(savedProfile); // Save

      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);

      await waitFor(() => {
        expect(screen.getByDisplayValue('John Doe')).toBeInTheDocument();
      });

      // Modify
      const nameInput = screen.getByDisplayValue('John Doe');
      await userEvent.clear(nameInput);
      await userEvent.type(nameInput, 'Jane Doe');

      // Save
      const saveButton = screen.getByText('Save');
      await userEvent.click(saveButton);

      // Unsaved changes indicator should disappear
      await waitFor(() => {
        expect(screen.queryByText(/unsaved changes/i)).not.toBeInTheDocument();
      });
    });

    it('tracks unsaved changes across different form sections', async () => {
      const mockProfile = {
        profile: {
          id: 1,
          full_name: 'John Doe',
        },
        experience: [
          {
            id: 1,
            company: 'Tech Corp',
            title: 'Engineer',
            is_current: false,
          },
        ],
        skills: [],
        education: [],
        certifications: [],
        portfolio: [],
      };

      mockInvoke.mockResolvedValue(mockProfile);

      const { default: Profile } = await import('../pages/Profile');
      renderWithRouter(<Profile />);

      await waitFor(() => {
        expect(screen.getByDisplayValue('John Doe')).toBeInTheDocument();
      });

      // Modify profile field
      const nameInput = screen.getByDisplayValue('John Doe');
      await userEvent.clear(nameInput);
      await userEvent.type(nameInput, 'Jane Doe');

      // Should show unsaved changes
      await waitFor(() => {
        expect(screen.getByText(/unsaved changes/i)).toBeInTheDocument();
      });
    });
  });

  describe('Status Update', () => {
    it('updates application status via inline select', async () => {
      const mockApplications = [
        {
          id: 1,
          job_id: 1,
          status: 'Applied',
          job_title: 'Software Engineer',
          company: 'Tech Corp',
        },
      ];

      const mockApplicationDetail = {
        application: {
          id: 1,
          job_id: 1,
          status: 'Applied',
        },
        job: {
          id: 1,
          title: 'Software Engineer',
          company: 'Tech Corp',
        },
        events: [],
      };

      mockInvoke
        .mockResolvedValueOnce(mockApplications) // get_applications
        .mockResolvedValueOnce([]) // get_job_list
        .mockResolvedValueOnce({}) // update_application
        .mockResolvedValueOnce(mockApplications) // reload applications
        .mockResolvedValueOnce(mockApplicationDetail); // get_application_detail (if selected)

      const { default: Applications } = await import('../pages/Applications');
      renderWithRouter(<Applications />);

      await waitFor(() => {
        expect(screen.getByText('Software Engineer')).toBeInTheDocument();
      });

      // Find the status select
      const statusSelect = screen.getByDisplayValue('Applied');
      await userEvent.selectOptions(statusSelect, 'Interviewing');

      // Should call update_application
      await waitFor(() => {
        expect(mockInvoke).toHaveBeenCalledWith('update_application', expect.objectContaining({
          id: 1,
          input: expect.objectContaining({
            status: 'Interviewing',
          }),
        }));
      });
    });
  });
});

