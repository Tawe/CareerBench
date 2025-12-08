// Utility functions for application data manipulation
// These are pure functions that can be easily tested

export type ApplicationStatus =
  | "Saved"
  | "Draft"
  | "Applied"
  | "Interviewing"
  | "Offer"
  | "Rejected"
  | "Ghosted"
  | "Withdrawn";

export interface ApplicationSummary {
  id: number;
  job_id: number;
  status: ApplicationStatus;
  job_title?: string;
  company?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  date_applied?: string;
  archived?: boolean;
}

/**
 * Filters applications by a specific status.
 * 
 * @param applications - Array of application summaries to filter
 * @param status - Status to filter by, or "all" to return all applications
 * @returns Filtered array of applications matching the specified status
 * 
 * @example
 * ```typescript
 * const savedApps = filterApplicationsByStatus(applications, "Saved");
 * const allApps = filterApplicationsByStatus(applications, "all");
 * ```
 */
export function filterApplicationsByStatus(
  applications: ApplicationSummary[],
  status: ApplicationStatus | "all"
): ApplicationSummary[] {
  if (status === "all") {
    return applications;
  }
  return applications.filter((app) => app.status === status);
}

/**
 * Groups applications by their status into a record object.
 * 
 * @param applications - Array of application summaries to group
 * @returns Record mapping each status to an array of applications with that status
 * 
 * @example
 * ```typescript
 * const grouped = groupApplicationsByStatus(applications);
 * console.log(grouped["Applied"]); // Array of applications with "Applied" status
 * ```
 */
export function groupApplicationsByStatus(
  applications: ApplicationSummary[]
): Record<ApplicationStatus, ApplicationSummary[]> {
  const statuses: ApplicationStatus[] = [
    "Saved",
    "Draft",
    "Applied",
    "Interviewing",
    "Offer",
    "Rejected",
    "Ghosted",
    "Withdrawn",
  ];

  return statuses.reduce(
    (acc, status) => {
      acc[status] = applications.filter((app) => app.status === status);
      return acc;
    },
    {} as Record<ApplicationStatus, ApplicationSummary[]>
  );
}

/**
 * Formats a status label with its count for display.
 * 
 * @param status - The application status
 * @param count - The number of applications with this status
 * @returns Formatted string like "Applied (5)"
 * 
 * @example
 * ```typescript
 * const label = getStatusLabelWithCount("Applied", 5); // "Applied (5)"
 * ```
 */
export function getStatusLabelWithCount(
  status: ApplicationStatus,
  count: number
): string {
  return `${status} (${count})`;
}

/**
 * Gets a human-readable label for an application status.
 * Currently returns the status as-is, but can be extended for i18n or formatting.
 * 
 * @param status - The application status
 * @returns Human-readable status label
 * 
 * @example
 * ```typescript
 * const label = getStatusLabel("Interviewing"); // "Interviewing"
 * ```
 */
export function getStatusLabel(status: ApplicationStatus): string {
  return status;
}

/**
 * Sorts applications by their application date, most recent first.
 * Applications without a date are placed at the end.
 * 
 * @param applications - Array of application summaries to sort
 * @returns New sorted array (original array is not modified)
 * 
 * @example
 * ```typescript
 * const sorted = sortApplicationsByDate(applications);
 * // Most recent applications appear first
 * ```
 */
export function sortApplicationsByDate(
  applications: ApplicationSummary[]
): ApplicationSummary[] {
  return [...applications].sort((a, b) => {
    const dateA = a.date_applied ? new Date(a.date_applied).getTime() : 0;
    const dateB = b.date_applied ? new Date(b.date_applied).getTime() : 0;
    return dateB - dateA; // Most recent first
  });
}

/**
 * Sorts applications by priority level, highest priority first.
 * Priority order: Dream > High > Medium > Low > undefined
 * 
 * @param applications - Array of application summaries to sort
 * @returns New sorted array (original array is not modified)
 * 
 * @example
 * ```typescript
 * const sorted = sortApplicationsByPriority(applications);
 * // Dream jobs appear first, followed by High, Medium, Low
 * ```
 */
export function sortApplicationsByPriority(
  applications: ApplicationSummary[]
): ApplicationSummary[] {
  const priorityOrder: Record<string, number> = {
    Dream: 4,
    High: 3,
    Medium: 2,
    Low: 1,
  };

  return [...applications].sort((a, b) => {
    const priorityA = a.priority ? priorityOrder[a.priority] || 0 : 0;
    const priorityB = b.priority ? priorityOrder[b.priority] || 0 : 0;
    return priorityB - priorityA; // Higher priority first
  });
}

/**
 * Filters out archived applications, returning only active ones.
 * 
 * @param applications - Array of application summaries to filter
 * @returns Array containing only non-archived applications
 * 
 * @example
 * ```typescript
 * const active = filterActiveApplications(applications);
 * // Only returns applications where archived !== true
 * ```
 */
export function filterActiveApplications(
  applications: ApplicationSummary[]
): ApplicationSummary[] {
  return applications.filter((app) => !app.archived);
}

/**
 * Counts the number of active (non-archived) applications.
 * 
 * @param applications - Array of application summaries to count
 * @returns Number of active applications
 * 
 * @example
 * ```typescript
 * const count = getActiveApplicationsCount(applications);
 * console.log(`You have ${count} active applications`);
 * ```
 */
export function getActiveApplicationsCount(
  applications: ApplicationSummary[]
): number {
  return filterActiveApplications(applications).length;
}

