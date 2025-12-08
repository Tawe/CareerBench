import { useState, useEffect, useCallback } from 'react';
import { invoke as tauriInvoke } from '@tauri-apps/api/core';

export interface DashboardKpis {
  total_jobs_tracked: number;
  total_applications: number;
  active_applications: number;
  applications_last_30_days: number;
  offers_received: number;
}

export interface StatusBucket {
  status: string;
  count: number;
}

export interface ActivityDay {
  date: string;
  count: number;
}

export interface FunnelStep {
  label: string;
  count: number;
}

export interface DashboardData {
  kpis: DashboardKpis;
  status_breakdown: StatusBucket[];
  activity_last_30_days: ActivityDay[];
  funnel: FunnelStep[];
}

/**
 * Custom React hook for fetching and managing dashboard data.
 * 
 * Provides loading state, error handling, and automatic retry logic.
 * Includes a 500ms initial delay to ensure Tauri API is initialized.
 * 
 * @returns Object containing:
 *   - `data`: DashboardData object with KPIs, status breakdown, activity, and funnel data
 *   - `isLoading`: Boolean indicating if data is currently being fetched
 *   - `error`: Error message string, or null if no error
 *   - `reload`: Function to manually reload dashboard data
 * 
 * @example
 * ```typescript
 * function Dashboard() {
 *   const { data, isLoading, error, reload } = useDashboardData();
 *   
 *   if (isLoading) return <Loading />;
 *   if (error) return <Error message={error} />;
 *   return <DashboardView data={data} onRefresh={reload} />;
 * }
 * ```
 */
export function useDashboardData() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadDashboard = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    
    // Retry logic: try up to 3 times with delays
    let attempts = 0;
    const maxAttempts = 3;
    
    while (attempts < maxAttempts) {
      try {
        const result = await tauriInvoke<DashboardData>("get_dashboard_data");
        setData(result);
        setIsLoading(false);
        return; // Success!
      } catch (err: any) {
        attempts++;
        const errorMessage = err?.message || String(err);
        console.error(`Dashboard load attempt ${attempts} failed:`, err);
        
        // If this is the last attempt, show the error
        if (attempts >= maxAttempts) {
          if (errorMessage.includes("invoke") || errorMessage.includes("undefined") || errorMessage.includes("Cannot read")) {
            setError("Tauri API not available. Make sure you're running 'npm run tauri dev' (not 'npm run dev'). If you are, try clicking Retry after waiting a few seconds.");
          } else {
            setError(errorMessage);
          }
          setIsLoading(false);
          return;
        }
        
        // Wait before retrying (exponential backoff)
        await new Promise(resolve => setTimeout(resolve, 1000 * attempts));
      }
    }
  }, []);

  useEffect(() => {
    // Initial load with delay to ensure Tauri API is initialized
    const timer = setTimeout(() => {
      loadDashboard();
    }, 500);
    return () => clearTimeout(timer);
  }, [loadDashboard]);

  return {
    data,
    isLoading,
    error,
    reload: loadDashboard,
  };
}

