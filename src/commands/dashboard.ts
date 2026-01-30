/**
 * Dashboard command types
 */

export interface DashboardKpis {
  totalJobsTracked: number;
  totalApplications: number;
  activeApplications: number;
  applicationsLast30Days: number;
  offersReceived: number;
}

export interface StatusBucket {
  status: string;
  count: number;
}

export interface DailyActivityPoint {
  date: string;
  applicationsCreated: number;
  interviewsCompleted: number;
  offersReceived: number;
}

export interface FunnelStep {
  label: string;
  count: number;
}

export interface DashboardData {
  kpis: DashboardKpis;
  statusBreakdown: StatusBucket[];
  activityLast30Days: DailyActivityPoint[];
  funnel: FunnelStep[];
}

export interface DashboardCommands {
  get_dashboard_data: {
    args: [];
    return: DashboardData;
  };
}