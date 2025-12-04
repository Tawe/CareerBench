import { useEffect, useState } from "react";
import { invoke as tauriInvoke, isTauri } from "@tauri-apps/api/core";
import "./Dashboard.css";

interface DashboardKpis {
  total_jobs_tracked: number;
  total_applications: number;
  active_applications: number;
  applications_last_30_days: number;
  offers_received: number;
}

interface StatusBucket {
  status: string;
  count: number;
}

interface DailyActivityPoint {
  date: string;
  applications_created: number;
  interviews_completed: number;
  offers_received: number;
}

interface FunnelStep {
  label: string;
  count: number;
}

interface DashboardData {
  kpis: DashboardKpis;
  status_breakdown: StatusBucket[];
  activity_last_30_days: DailyActivityPoint[];
  funnel: FunnelStep[];
}

export default function Dashboard() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function loadDashboard() {
    setIsLoading(true);
    setError(null);
    
    // Retry logic: try up to 3 times with delays
    let attempts = 0;
    const maxAttempts = 3;
    
    while (attempts < maxAttempts) {
      try {
        // Try to call invoke - if Tauri is available, this will work
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
  }

  useEffect(() => {
    // Initial load with delay to ensure Tauri API is initialized
    const timer = setTimeout(() => {
      loadDashboard();
    }, 500);
    return () => clearTimeout(timer);
  }, []);

  if (isLoading) {
    return (
      <div className="dashboard">
        <div className="loading">Loading dashboard...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="dashboard">
        <div className="error">
          <p>{error}</p>
          <button onClick={loadDashboard}>Retry</button>
        </div>
      </div>
    );
  }

  if (!data) {
    return null;
  }

  return (
    <div className="dashboard">
      <div className="dashboard-header">
        <h1>Dashboard</h1>
        <button onClick={loadDashboard}>Reload</button>
      </div>

      <div className="kpi-row">
        <div className="kpi-card">
          <div className="kpi-value">{data.kpis.active_applications}</div>
          <div className="kpi-label">Active Applications</div>
        </div>
        <div className="kpi-card">
          <div className="kpi-value">{data.kpis.applications_last_30_days}</div>
          <div className="kpi-label">Applications (Last 30 Days)</div>
        </div>
        <div className="kpi-card">
          <div className="kpi-value">{data.kpis.offers_received}</div>
          <div className="kpi-label">Offers Received</div>
        </div>
        <div className="kpi-card">
          <div className="kpi-value">{data.kpis.total_jobs_tracked}</div>
          <div className="kpi-label">Total Jobs Tracked</div>
        </div>
      </div>

      <div className="dashboard-row">
        <div className="dashboard-section">
          <h2>Status Breakdown</h2>
          <div className="status-list">
            {data.status_breakdown.map((bucket) => (
              <div key={bucket.status} className="status-item">
                <span className="status-name">{bucket.status}</span>
                <span className="status-count">{bucket.count}</span>
              </div>
            ))}
          </div>
        </div>

        <div className="dashboard-section">
          <h2>Pipeline Funnel</h2>
          <div className="funnel">
            {data.funnel.map((step) => (
              <div key={step.label} className="funnel-step">
                <div className="funnel-label">{step.label}</div>
                <div className="funnel-bar" style={{ height: `${(step.count / (data.funnel[0]?.count || 1)) * 200}px` }}>
                  {step.count}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>

      <div className="dashboard-section">
        <h2>Activity Calendar (Last 30 Days)</h2>
        <div className="activity-calendar">
          {data.activity_last_30_days.map((point) => {
            const date = new Date(point.date);
            const dayOfWeek = date.toLocaleDateString('en-US', { weekday: 'short' });
            const day = date.getDate();
            const hasActivity = point.applications_created > 0 || point.interviews_completed > 0 || point.offers_received > 0;
            
            return (
              <div 
                key={point.date} 
                className={`calendar-day ${hasActivity ? 'has-activity' : ''}`}
                title={`${point.date}: ${point.applications_created} applications, ${point.interviews_completed} interviews, ${point.offers_received} offers`}
              >
                <div className="calendar-day-header">
                  <span className="calendar-day-name">{dayOfWeek}</span>
                  <span className="calendar-day-number">{day}</span>
                </div>
                {hasActivity && (
                  <div className="calendar-day-activity">
                    {point.applications_created > 0 && (
                      <span className="activity-dot applications" title={`${point.applications_created} applications`}></span>
                    )}
                    {point.interviews_completed > 0 && (
                      <span className="activity-dot interviews" title={`${point.interviews_completed} interviews`}></span>
                    )}
                    {point.offers_received > 0 && (
                      <span className="activity-dot offers" title={`${point.offers_received} offers`}></span>
                    )}
                  </div>
                )}
              </div>
            );
          })}
        </div>
        <div className="calendar-legend">
          <div className="legend-item">
            <span className="activity-dot applications"></span>
            <span>Applications</span>
          </div>
          <div className="legend-item">
            <span className="activity-dot interviews"></span>
            <span>Interviews</span>
          </div>
          <div className="legend-item">
            <span className="activity-dot offers"></span>
            <span>Offers</span>
          </div>
        </div>
      </div>
    </div>
  );
}

