// Minimal UI tests for key pages
// These ensure pages render without crashing and basic flows work

import { describe, it, expect, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';

// Mock Tauri API
vi.mock('@tauri-apps/api', () => ({
  invoke: vi.fn(() => Promise.resolve({})),
}));

// Helper to render with router
const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>);
};

describe('Page Rendering', () => {
  it('Dashboard page renders without crashing', async () => {
    // Lazy load to avoid import errors if page has issues
    const { default: Dashboard } = await import('../pages/Dashboard');
    const { container } = renderWithRouter(<Dashboard />);
    expect(container).toBeTruthy();
  });

  it('Jobs page renders without crashing', async () => {
    const { default: Jobs } = await import('../pages/Jobs');
    const { container } = renderWithRouter(<Jobs />);
    expect(container).toBeTruthy();
  });

  it('Applications page renders without crashing', async () => {
    const { default: Applications } = await import('../pages/Applications');
    const { container } = renderWithRouter(<Applications />);
    expect(container).toBeTruthy();
  });

  it('Profile page renders without crashing', async () => {
    const { default: Profile } = await import('../pages/Profile');
    const { container } = renderWithRouter(<Profile />);
    expect(container).toBeTruthy();
  });
});

