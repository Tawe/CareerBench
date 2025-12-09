import { useEffect, useState } from "react";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
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

interface DateRange {
  start_date: string;
  end_date: string;
}

interface DashboardData {
  kpis: DashboardKpis;
  status_breakdown: StatusBucket[];
  activity_last_30_days: DailyActivityPoint[];
  funnel: FunnelStep[];
  date_range?: DateRange;
}

interface ConversionRates {
  applicationToInterview: number;
  interviewToOffer: number;
  applicationToOffer: number;
  totalApplications: number;
  totalInterviews: number;
  totalOffers: number;
}

interface TimeInStage {
  stage: string;
  averageDays: number;
  medianDays: number;
  minDays?: number;
  maxDays?: number;
  sampleSize: number;
}

interface ChannelEffectiveness {
  channel: string | null;
  totalApplications: number;
  interviews: number;
  offers: number;
  interviewRate: number;
  offerRate: number;
  averageTimeToInterview?: number;
  averageTimeToOffer?: number;
}

interface Insight {
  category: string;
  title: string;
  message: string;
  priority: "high" | "medium" | "low";
  actionable: boolean;
}

export default function Dashboard() {
  const [data, setData] = useState<DashboardData | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [conversionRates, setConversionRates] = useState<ConversionRates | null>(null);
  const [timeInStage, setTimeInStage] = useState<TimeInStage[]>([]);
  const [channelEffectiveness, setChannelEffectiveness] = useState<ChannelEffectiveness[]>([]);
  const [insights, setInsights] = useState<Insight[]>([]);
  const [isLoadingAnalytics, setIsLoadingAnalytics] = useState(false);
  const [startDate, setStartDate] = useState<string>(() => {
    const date = new Date();
    date.setDate(date.getDate() - 30);
    return date.toISOString().split('T')[0];
  });
  const [endDate, setEndDate] = useState<string>(() => {
    return new Date().toISOString().split('T')[0];
  });

  async function loadDashboard() {
    setIsLoading(true);
    setError(null);
    
    // Retry logic: try up to 3 times with delays
    let attempts = 0;
    const maxAttempts = 3;
    
    while (attempts < maxAttempts) {
      try {
        // Try to call invoke - if Tauri is available, this will work
        const result = await tauriInvoke<DashboardData>("get_dashboard_data", {
          startDate: startDate || null,
          endDate: endDate || null,
        });
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

  async function handleExport() {
    try {
      const csv = await tauriInvoke<string>("export_dashboard_data", {
        startDate: startDate || null,
        endDate: endDate || null,
      });

      const filePath = await save({
        defaultPath: `careerbench-dashboard-${new Date().toISOString().split('T')[0]}.csv`,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });

      if (filePath) {
        await writeTextFile(filePath, csv);
        showToast("Dashboard data exported successfully", "success");
      }
    } catch (err: any) {
      showToast(err?.message || "Failed to export dashboard data", "error");
    }
  }

  async function loadAnalytics() {
    setIsLoadingAnalytics(true);
    try {
      const [conversion, timeInStageData, channels, insightsData] = await Promise.all([
        tauriInvoke<ConversionRates>("get_conversion_rates", {
          startDate: startDate || null,
          endDate: endDate || null,
        }),
        tauriInvoke<TimeInStage[]>("get_time_in_stage", {
          startDate: startDate || null,
          endDate: endDate || null,
        }),
        tauriInvoke<ChannelEffectiveness[]>("get_channel_effectiveness", {
          startDate: startDate || null,
          endDate: endDate || null,
        }),
        tauriInvoke<Insight[]>("get_analytics_insights", {
          startDate: startDate || null,
          endDate: endDate || null,
        }),
      ]);

      setConversionRates(conversion);
      setTimeInStage(timeInStageData);
      setChannelEffectiveness(channels);
      setInsights(insightsData);
    } catch (err: any) {
      console.error("Failed to load analytics:", err);
    } finally {
      setIsLoadingAnalytics(false);
    }
  }

  function handleDateRangeChange() {
    loadDashboard();
    loadAnalytics();
  }

  useEffect(() => {
    // Initial load with delay to ensure Tauri API is initialized
    const timer = setTimeout(() => {
      loadDashboard();
      loadAnalytics();
    }, 500);
    return () => clearTimeout(timer);
  }, [startDate, endDate]);

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
        <div className="dashboard-controls">
          <div className="date-range-selector">
            <label htmlFor="start-date">From:</label>
            <input
              id="start-date"
              type="date"
              value={startDate}
              onChange={(e) => setStartDate(e.target.value)}
              onBlur={handleDateRangeChange}
              max={endDate}
            />
            <label htmlFor="end-date">To:</label>
            <input
              id="end-date"
              type="date"
              value={endDate}
              onChange={(e) => setEndDate(e.target.value)}
              onBlur={handleDateRangeChange}
              min={startDate}
              max={new Date().toISOString().split('T')[0]}
            />
          </div>
          <button 
            onClick={loadDashboard}
            aria-label="Reload dashboard data"
          >
            Reload
          </button>
          <button 
            onClick={handleExport}
            aria-label="Export dashboard data"
            className="export-button"
          >
            Export CSV
          </button>
        </div>
      </div>

      <div className="kpi-row">
        <div className="kpi-card">
          <div className="kpi-value">{data.kpis.active_applications}</div>
          <div className="kpi-label">Active Applications</div>
        </div>
        <div className="kpi-card">
          <div className="kpi-value">{data.kpis.applications_last_30_days}</div>
          <div className="kpi-label">Applications (Date Range)</div>
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
        <h2>Activity Over Time{data.date_range ? ` (${data.date_range.start_date} to ${data.date_range.end_date})` : ' (Last 30 Days)'}</h2>
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

      {/* Analytics & Insights Section */}
      <div className="analytics-section">
        <h2>Analytics & Insights</h2>
        
        {/* Conversion Rates */}
        {conversionRates && (
          <div className="dashboard-section">
            <h3>Conversion Rates</h3>
            <div className="conversion-rates-grid">
              <div className="conversion-card">
                <div className="conversion-label">Application → Interview</div>
                <div className="conversion-value">{conversionRates.applicationToInterview.toFixed(1)}%</div>
                <div className="conversion-detail">
                  {conversionRates.totalInterviews} of {conversionRates.totalApplications} applications
                </div>
              </div>
              <div className="conversion-card">
                <div className="conversion-label">Interview → Offer</div>
                <div className="conversion-value">{conversionRates.interviewToOffer.toFixed(1)}%</div>
                <div className="conversion-detail">
                  {conversionRates.totalOffers} of {conversionRates.totalInterviews} interviews
                </div>
              </div>
              <div className="conversion-card">
                <div className="conversion-label">Application → Offer</div>
                <div className="conversion-value">{conversionRates.applicationToOffer.toFixed(1)}%</div>
                <div className="conversion-detail">
                  {conversionRates.totalOffers} of {conversionRates.totalApplications} applications
                </div>
              </div>
            </div>
          </div>
        )}

        {/* Time in Stage */}
        {timeInStage.length > 0 && (
          <div className="dashboard-section">
            <h3>Average Time in Stage</h3>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={timeInStage}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis 
                  dataKey="stage" 
                  angle={-45}
                  textAnchor="end"
                  height={80}
                  style={{ fontSize: "0.75rem" }}
                />
                <YAxis label={{ value: 'Days', angle: -90, position: 'insideLeft' }} />
                <Tooltip />
                <Legend />
                <Bar dataKey="averageDays" fill="#6366f1" name="Average Days" radius={[4, 4, 0, 0]} />
                <Bar dataKey="medianDays" fill="#8b5cf6" name="Median Days" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
            <div className="time-in-stage-table">
              <table>
                <thead>
                  <tr>
                    <th>Stage</th>
                    <th>Avg Days</th>
                    <th>Median Days</th>
                    <th>Range</th>
                    <th>Sample Size</th>
                  </tr>
                </thead>
                <tbody>
                  {timeInStage.map((stage) => (
                    <tr key={stage.stage}>
                      <td>{stage.stage}</td>
                      <td>{stage.averageDays.toFixed(0)}</td>
                      <td>{stage.medianDays.toFixed(0)}</td>
                      <td>
                        {stage.minDays !== undefined && stage.maxDays !== undefined
                          ? `${stage.minDays}-${stage.maxDays}`
                          : "-"}
                      </td>
                      <td>{stage.sampleSize}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* Channel Effectiveness */}
        {channelEffectiveness.length > 0 && (
          <div className="dashboard-section">
            <h3>Channel Effectiveness</h3>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={channelEffectiveness}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis 
                  dataKey="channel" 
                  angle={-45}
                  textAnchor="end"
                  height={80}
                  style={{ fontSize: "0.75rem" }}
                />
                <YAxis label={{ value: 'Rate (%)', angle: -90, position: 'insideLeft' }} />
                <Tooltip />
                <Legend />
                <Bar dataKey="interviewRate" fill="#6366f1" name="Interview Rate %" radius={[4, 4, 0, 0]} />
                <Bar dataKey="offerRate" fill="#10b981" name="Offer Rate %" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
            <div className="channel-table">
              <table>
                <thead>
                  <tr>
                    <th>Channel</th>
                    <th>Applications</th>
                    <th>Interviews</th>
                    <th>Offers</th>
                    <th>Interview Rate</th>
                    <th>Offer Rate</th>
                    <th>Avg Days to Interview</th>
                  </tr>
                </thead>
                <tbody>
                  {channelEffectiveness.map((channel, idx) => (
                    <tr key={idx}>
                      <td>{channel.channel || "Unknown"}</td>
                      <td>{channel.totalApplications}</td>
                      <td>{channel.interviews}</td>
                      <td>{channel.offers}</td>
                      <td>{channel.interviewRate.toFixed(1)}%</td>
                      <td>{channel.offerRate.toFixed(1)}%</td>
                      <td>{channel.averageTimeToInterview?.toFixed(0) || "-"} days</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* AI Insights */}
        {insights.length > 0 && (
          <div className="dashboard-section">
            <h3>AI Insights & Recommendations</h3>
            <div className="insights-list">
              {insights
                .sort((a, b) => {
                  const priorityOrder = { high: 3, medium: 2, low: 1 };
                  return priorityOrder[b.priority] - priorityOrder[a.priority];
                })
                .map((insight, idx) => (
                  <div
                    key={idx}
                    className={`insight-card insight-${insight.priority}`}
                  >
                    <div className="insight-header">
                      <span className="insight-category">{insight.category}</span>
                      <span className={`insight-priority priority-${insight.priority}`}>
                        {insight.priority.toUpperCase()}
                      </span>
                    </div>
                    <h4 className="insight-title">{insight.title}</h4>
                    <p className="insight-message">{insight.message}</p>
                    {insight.actionable && (
                      <span className="insight-actionable">💡 Actionable</span>
                    )}
                  </div>
                ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

