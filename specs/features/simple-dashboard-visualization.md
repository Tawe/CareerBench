# Simple Dashboard Visualization Specification

This spec is intended to guide Cursor AI when generating code for the **Simple Dashboard** feature in the CareerBench Tauri desktop application.

The Simple Dashboard is a **read-only overview** of the user’s job search pipeline. It pulls from existing data (jobs, applications, events) and shows a small set of **high-signal metrics and charts**.

This is meant to be **MVP-friendly**: no heavy analytics engine, just straightforward queries and basic visualizations.

---

## 1. Goals & Scope

### 1.1 Primary Goals

- Give the user a quick, visual sense of:
    - How many jobs they have at each stage (Saved → Applied → Interviewing → Offer → Rejected/Ghosted).
    - How many applications they’ve created recently.
    - Simple activity trends (e.g., recent applications, interviews, offers).
- Use existing tables: `jobs`, `applications`, `application_events`.
- Provide a small, well-defined **Tauri API** for fetching aggregated stats.
- Provide a **clean, minimal UI** suitable for a Tauri desktop app (React).

### 1.2 In-Scope (MVP)

- A **Dashboard screen** accessible from main navigation.
- Backend aggregation endpoint(s) to compute:
    - Counts by application status.
    - Recent activity counts by day.
    - Basic funnel metrics (Applied → Interviewing → Offer → Accepted/Rejected, where data exists).
- Simple visualizations:
    - KPI tiles (big numbers).
    - One small bar chart or line chart for time-based trends.
    - A simple funnel-style bar visualization.

### 1.3 Out-of-Scope (for this spec)

- Advanced analytics (conversion percentages by channel, time-in-stage, etc.).
- AI analysis / coaching of metrics.
- Exporting dashboard data.

---

## 2. Data Sources

The dashboard reads from existing tables:
- `jobs`
- `applications`
- `application_events`

No new tables are strictly required.

### 2.1 Assumed Fields (from other specs)

`**applications**` (relevant fields):
- `id`
- `job_id`
- `status` (Saved, Draft, Applied, Interviewing, Offer, Rejected, Ghosted, Withdrawn)
- `date_saved`
- `date_applied`
- `last_activity_date`
- `archived`

`**application_events**` (relevant fields):
- `id`
- `application_id`
- `event_type` (ApplicationCreated, StatusChanged, Applied, InterviewScheduled, InterviewCompleted, OfferReceived, OfferAccepted, OfferDeclined, Rejected, MarkedGhosted, etc.)
- `event_date`    

---

## 3. Dashboard Data Model (Frontend)

Define a single payload for the dashboard:

```rust
export type ApplicationStatus =
  | "Saved"
  | "Draft"
  | "Applied"
  | "Interviewing"
  | "Offer"
  | "Rejected"
  | "Ghosted"
  | "Withdrawn";

export interface DashboardKpis {
  totalJobsTracked: number;          // jobs in the system
  totalApplications: number;        // applications in the system
  activeApplications: number;       // applications where archived = false
  applicationsLast30Days: number;   // created in last 30 days
  offersReceived: number;           // status = Offer
}

export interface StatusBucket {
  status: ApplicationStatus;
  count: number;
}

export interface DailyActivityPoint {
  date: string;        // ISO date (YYYY-MM-DD)
  applicationsCreated: number;
  interviewsCompleted: number;
  offersReceived: number;
}

export interface FunnelStep {
  label: string;       // e.g. "Applied", "Interviewing", "Offer"
  count: number;
}

export interface DashboardData {
  kpis: DashboardKpis;
  statusBreakdown: StatusBucket[];
  activityLast30Days: DailyActivityPoint[];
  funnel: FunnelStep[];
}
```

---

## 4. Tauri Command Design

### 4.1 `get_dashboard_data`

**Signature (Rust):**

```rust
#[tauri::command]
pub async fn get_dashboard_data() -> Result<DashboardData, String> {
    // implementation described below
}
```

**Behavior:**
1. Compute **KPI metrics**:
    - `totalJobsTracked` = `COUNT(*) FROM jobs`.
    - `totalApplications` = `COUNT(*) FROM applications`.
    - `activeApplications` = `COUNT(*) FROM applications WHERE archived = 0`.
    - `applicationsLast30Days` = `COUNT(*) FROM applications WHERE date_saved >= (today - 30 days)`.
    - `offersReceived` = `COUNT(*) FROM applications WHERE status = 'Offer'`.
2. Compute **status breakdown**:
    - Group applications by `status` (active only, `archived = 0`).
    - Generate one `StatusBucket` per status (0 is allowed).
3. Compute **activityLast30Days**:
    - For each day in the last 30 days:
        - `applicationsCreated` = `COUNT(*) FROM applications WHERE date(date_saved) = that_day`.
        - `interviewsCompleted` = `COUNT(*) FROM application_events WHERE event_type = 'InterviewCompleted' AND date(event_date) = that_day`.
        - `offersReceived` = `COUNT(*) FROM application_events WHERE event_type = 'OfferReceived' AND date(event_date) = that_day`.
    - Return as `DailyActivityPoint[]`, sorted by date ascending.
4. Compute **funnel** (simple counts):
    - Define steps:
        - `Applied` = `COUNT(*) FROM applications WHERE status IN ('Applied', 'Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn')`.
        - `Interviewing` = `COUNT(*) FROM applications WHERE status IN ('Interviewing', 'Offer', 'Rejected', 'Ghosted', 'Withdrawn')`.
        - `Offer` = `COUNT(*) FROM applications WHERE status = 'Offer'`.            
    - Return as:
    
    ```json
    [
      { "label": "Applied", "count": X },
      { "label": "Interviewing", "count": Y },
      { "label": "Offer", "count": Z }
    ]
    ```

5. Build `DashboardData` and return.

**Error Handling:**
- Any DB error → return `Err("Failed to load dashboard data")`.
- Frontend will show an error message and allow retry.

---

## 5. SQL / Query Hints (Implementation Detail)

Cursor can implement the above using one or more queries. For MVP, simplicity is more important than hyper-optimized single-query solutions.

Examples (pseudo-SQL):

```sql
SELECT COUNT(*) FROM jobs;

SELECT COUNT(*) FROM applications;

SELECT COUNT(*) FROM applications WHERE archived = 0;

SELECT COUNT(*) FROM applications
 WHERE date_saved >= date('now', '-30 day');

SELECT COUNT(*) FROM applications
 WHERE status = 'Offer';

SELECT status, COUNT(*)
FROM applications
WHERE archived = 0
GROUP BY status;

-- Daily applications created in last 30 days
SELECT date(date_saved) AS day, COUNT(*) AS applicationsCreated
FROM applications
WHERE date_saved >= date('now', '-30 day')
GROUP BY day;

-- Daily interviews completed
SELECT date(event_date) AS day, COUNT(*) AS interviewsCompleted
FROM application_events
WHERE event_type = 'InterviewCompleted'
  AND event_date >= date('now', '-30 day')
GROUP BY day;

-- Daily offers received
SELECT date(event_date) AS day, COUNT(*) AS offersReceived
FROM application_events
WHERE event_type = 'OfferReceived'
  AND event_date >= date('now', '-30 day')
GROUP BY day;
```

Cursor should then merge these results into the `DailyActivityPoint[]` structure, filling missing dates with zero values.

---

## 6. Frontend UX – Dashboard Page

### 6.1 Layout

A simple, responsive layout:

1. **Header**
    - Title: `Dashboard`
    - Subtitle: `Quick overview of your job search`
    - Refresh button: `Reload` (calls `get_dashboard_data`).
        
2. **Top KPI Row** (cards/tiles)
    - Tile 1: `Active Applications`
    - Tile 2: `Applications (Last 30 Days)`
    - Tile 3: `Offers Received`
    - Tile 4: `Total Jobs Tracked`
    Each tile shows:
    - Big number (value from `DashboardKpis`).        
    - Short label.

3. **Middle Row**
    **Left:** Status Breakdown
    - Use a simple bar chart or stacked bar/list to show counts by status.
    - Alternatively, a horizontal list with colored badges:
        - Saved, Draft, Applied, Interviewing, Offer, Rejected, Ghosted, Withdrawn.
    **Right:** Simple Funnel
    - Three vertical bars labeled: `Applied`, `Interviewing`, `Offer`.        
    - Height proportional to `count` from `funnel`.

4. **Bottom Row**
    **Activity Over Time**
    - A simple line or bar chart for the last 30 days.
    - X-axis: dates.
    - Y-axis: counts.
    - MVP: Use a single metric (e.g., `applicationsCreated`).
    - Optionally allow toggling series (applications, interviews, offers).

### 6.2 Loading & Error States

- On initial load:
    - Show skeleton or spinner while `get_dashboard_data` is in progress.
- On error:
    - Show an error banner: “Failed to load dashboard data. [Retry]”.

### 6.3 React State Shape

```javascript
interface DashboardState {
  data: DashboardData | null;
  isLoading: boolean;
  error?: string;
}
```

---

## 7. Frontend Integration (Tauri)

### 7.1 TypeScript Helper

```javascript
import { invoke } from "@tauri-apps/api/tauri";

export async function fetchDashboardData(): Promise<DashboardData> {
  return await invoke<DashboardData>("get_dashboard_data");
}
```

### 7.2 React Hook

```javascript
import { useEffect, useState } from "react";

export function useDashboardData() {
  const [state, setState] = useState<DashboardState>({
    data: null,
    isLoading: true,
  });

  async function load() {
    setState((prev) => ({ ...prev, isLoading: true, error: undefined }));
    try {
      const data = await fetchDashboardData();
      setState({ data, isLoading: false, error: undefined });
    } catch (err: any) {
      setState({ data: null, isLoading: false, error: err?.message ?? "Failed to load" });
    }
  }

  useEffect(() => {
    load();
  }, []);

  return {
    ...state,
    reload: load,
  };
}
```

---

## 8. Testing Scenarios

1. **No Data**
    - No jobs or applications.
    - All KPIs show 0.
    - Status breakdown shows 0 for each status.
    - Activity chart is flat at 0.
2. **Some Data Across Statuses**
    - Applications in different statuses.
    - Status breakdown reflects counts correctly.
    - Funnel counts make sense (Interviewing <= Applied, Offer <= Interviewing).
3. **Recent Activity**
    - Create applications/events with dates in last 30 days.
    - Confirm `activityLast30Days` includes appropriate non-zero points.
4. **Archived Applications**
    - Archived applications are excluded from `activeApplications` and status breakdown, but still counted in `totalApplications`.
5. **Error Handling**
    - Simulate DB failure or Tauri command error.
    - Confirm UI shows error banner and reload works when backend is restored.

---

## 9. Implementation Order (Recommended)

1. Implement the `DashboardData` Rust and TS types.
2. Implement the `get_dashboard_data` Tauri command:
    - Write SQL queries for KPIs, status breakdown, daily activity, and funnel.
    - Combine into a `DashboardData` struct.
3. Implement `fetchDashboardData` helper and `useDashboardData` hook.
4. Build the Dashboard page layout:
    - KPI tiles.
    - Status breakdown component.
    - Funnel component.
    - Activity chart component.
5. Add navigation entry for Dashboard.
6. Test with empty and seeded data.