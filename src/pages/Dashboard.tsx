import { useEffect, useState } from "react";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import {
  LineChart,
  Line,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
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
        <div className="dashboard-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="kpi-row">
          {Array.from({ length: 4 }).map((_, i) => (
            <LoadingSkeleton key={i} variant="card" width="100%" height="120px" />
          ))}
        </div>
        <div className="dashboard-row">
          <LoadingSkeleton variant="card" width="100%" height="300px" />
          <LoadingSkeleton variant="card" width="100%" height="300px" />
        </div>
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
        <button 
          onClick={loadDashboard}
          aria-label="Reload dashboard data"
        >
          Reload
        </button>
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
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={data.status_breakdown}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis 
                dataKey="status" 
                angle={-45}
                textAnchor="end"
                height={80}
                style={{ fontSize: "0.75rem" }}
              />
              <YAxis />
              <Tooltip />
              <Bar dataKey="count" fill="#6366f1" radius={[4, 4, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>

        <div className="dashboard-section">
          <h2>Pipeline Funnel</h2>
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={data.funnel} layout="vertical">
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis type="number" />
              <YAxis dataKey="label" type="category" width={100} />
              <Tooltip />
              <Bar dataKey="count" fill="#8b5cf6" radius={[0, 4, 4, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>

      <div className="dashboard-section">
        <h2>Activity Over Time (Last 30 Days)</h2>
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={data.activity_last_30_days.map(point => ({
            ...point,
            date: new Date(point.date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
          }))}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis 
              dataKey="date" 
              angle={-45}
              textAnchor="end"
              height={80}
              style={{ fontSize: "0.75rem" }}
            />
            <YAxis />
            <Tooltip />
            <Legend />
            <Line 
              type="monotone" 
              dataKey="applications_created" 
              stroke="#6366f1" 
              strokeWidth={2}
              name="Applications"
              dot={{ r: 4 }}
            />
            <Line 
              type="monotone" 
              dataKey="interviews_completed" 
              stroke="#8b5cf6" 
              strokeWidth={2}
              name="Interviews"
              dot={{ r: 4 }}
            />
            <Line 
              type="monotone" 
              dataKey="offers_received" 
              stroke="#10b981" 
              strokeWidth={2}
              name="Offers"
              dot={{ r: 4 }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

