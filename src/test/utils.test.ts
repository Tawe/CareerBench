// Example unit tests for pure utility functions
// These tests demonstrate the pattern for testing TypeScript logic

describe('Utility Functions', () => {
  // Example: Test filtering logic
  describe('getActiveApplicationsCount', () => {
    it('computes active applications count correctly', () => {
      const apps = [
        { id: 1, archived: false },
        { id: 2, archived: true },
        { id: 3, archived: false },
      ];
      
      const activeCount = apps.filter(app => !app.archived).length;
      expect(activeCount).toBe(2);
    });

    it('returns 0 when all applications are archived', () => {
      const apps = [
        { id: 1, archived: true },
        { id: 2, archived: true },
      ];
      
      const activeCount = apps.filter(app => !app.archived).length;
      expect(activeCount).toBe(0);
    });
  });

  // Example: Test status label derivation
  describe('deriveStatusLabel', () => {
    it('maps status codes to human-readable labels', () => {
      const statusMap: Record<string, string> = {
        'applied': 'Applied',
        'interviewing': 'Interviewing',
        'offer': 'Offer',
        'rejected': 'Rejected',
      };

      expect(statusMap['applied']).toBe('Applied');
      expect(statusMap['interviewing']).toBe('Interviewing');
    });
  });
});

