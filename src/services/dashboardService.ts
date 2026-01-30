import { invoke } from '@tauri-apps/api/core';
import type { DashboardData } from '../commands/dashboard';

export class ServiceError extends Error {
  constructor(message: string, public originalError?: unknown) {
    super(message);
    this.name = 'ServiceError';
  }
}

export class DashboardService {
  static async getDashboardData(): Promise<DashboardData> {
    try {
      return await invoke<DashboardData>('get_dashboard_data');
    } catch (error) {
      throw new ServiceError('Failed to fetch dashboard data', error);
    }
  }

  static async exportDashboardData(format: 'json' | 'csv'): Promise<void> {
    try {
      return await invoke('export_dashboard_data', { format });
    } catch (error) {
      throw new ServiceError(`Failed to export dashboard data as ${format}`, error);
    }
  }
}