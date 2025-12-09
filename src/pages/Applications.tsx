import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { InlineEditable } from "../components/InlineEditable";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import "./Applications.css";

type ApplicationStatus =
  | "Saved"
  | "Draft"
  | "Applied"
  | "Interviewing"
  | "Offer"
  | "Rejected"
  | "Ghosted"
  | "Withdrawn";

interface Application {
  id?: number;
  job_id: number;
  status: ApplicationStatus;
  channel?: string;
  priority?: "Low" | "Medium" | "High" | "Dream";
  date_saved: string;
  date_applied?: string;
  last_activity_date?: string;
  next_action_date?: string;
  next_action_note?: string;
  notes_summary?: string;
  contact_name?: string;
  contact_email?: string;
  contact_linkedin?: string;
  location_override?: string;
  offer_compensation?: string;
  archived: boolean;
  created_at: string;
  updated_at: string;
}

interface ApplicationSummary {
  id: number;
  job_id: number;
  job_title?: string;
  company?: string;
  status: ApplicationStatus;
  priority?: "Low" | "Medium" | "High" | "Dream";
  date_saved: string;
  date_applied?: string;
  last_activity_date?: string;
}

interface ApplicationEvent {
  id?: number;
  application_id: number;
  event_type: string;
  event_date: string;
  from_status?: ApplicationStatus;
  to_status?: ApplicationStatus;
  title?: string;
  details?: string;
  created_at: string;
}

interface ApplicationDetail {
  application: Application;
  events: ApplicationEvent[];
}

interface Job {
  id?: number;
  title?: string;
  company?: string;
}

interface PortfolioItem {
  id?: number;
  title: string;
  url?: string;
  description?: string;
  role?: string;
  tech_stack?: string;
  highlighted: boolean;
}

interface EmailThread {
  id?: number;
  applicationId?: number;
  threadId: string;
  subject?: string;
  participants?: string;
  lastMessageDate?: string;
  messageCount: number;
  isArchived: boolean;
  createdAt: string;
  updatedAt: string;
}

interface PaginatedApplicationList {
  applications: ApplicationSummary[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export default function Applications() {
  const [applications, setApplications] = useState<ApplicationSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedStatus, setSelectedStatus] = useState<ApplicationStatus | "all">("all");
  const [selectedApp, setSelectedApp] = useState<ApplicationDetail | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [availableJobs, setAvailableJobs] = useState<Job[]>([]);
  const [viewMode, setViewMode] = useState<"kanban" | "table">("kanban");
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize] = useState(50);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    loadApplications();
    loadAvailableJobs();
  }, [selectedStatus, currentPage]);

  async function loadApplications() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<PaginatedApplicationList>("get_applications", {
        status: selectedStatus === "all" ? null : selectedStatus,
        jobId: null,
        activeOnly: true,
        page: currentPage,
        pageSize: pageSize,
      });
      setApplications(result.applications);
      setTotal(result.total);
      setTotalPages(result.total_pages);
    } catch (err: any) {
      setError(err?.message || "Failed to load applications");
    } finally {
      setIsLoading(false);
    }
  }

  function handlePageChange(newPage: number) {
    setCurrentPage(newPage);
    // Scroll to top
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }

  async function loadAvailableJobs() {
    try {
      const jobs = await invoke<Job[]>("get_job_list", {
        search: null,
        activeOnly: true,
        source: null,
      });
      setAvailableJobs(jobs);
    } catch (err) {
      // Ignore errors for now
    }
  }

  async function loadApplicationDetail(id: number) {
    try {
      const detail = await invoke<ApplicationDetail>("get_application_detail", { id });
      setSelectedApp(detail);
    } catch (err: any) {
      setError(err?.message || "Failed to load application details");
    }
  }

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

  const applicationsByStatus = statuses.reduce((acc, status) => {
    acc[status] = applications.filter((app) => app.status === status);
    return acc;
  }, {} as Record<ApplicationStatus, ApplicationSummary[]>);

  if (isLoading && applications.length === 0) {
    return (
      <div className="applications">
        <div className="applications-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="applications-layout">
          <LoadingSkeleton variant="list" lines={8} />
        </div>
      </div>
    );
  }

  return (
    <div className="applications">
      <div className="applications-header">
        <h1>Applications</h1>
        <button 
          onClick={() => setShowCreateModal(true)} 
          className="add-button"
          aria-label="Create new application"
        >
          + Create Application
        </button>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button 
            onClick={() => setError(null)}
            aria-label="Dismiss error message"
          >
            <span aria-hidden="true">×</span>
          </button>
        </div>
      )}

      <div className="applications-layout">
        <div className="pipeline-view">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1rem" }}>
            <div className="status-filters">
              <button
                className={selectedStatus === "all" ? "active" : ""}
                onClick={() => setSelectedStatus("all")}
                aria-label="Show all applications"
                aria-pressed={selectedStatus === "all"}
              >
                All
              </button>
              {statuses.map((status) => (
                <button
                  key={status}
                  className={selectedStatus === status ? "active" : ""}
                  onClick={() => setSelectedStatus(status)}
                  aria-label={`Filter applications by ${status} status`}
                  aria-pressed={selectedStatus === status}
                >
                  {status} ({applicationsByStatus[status]?.length || 0})
                </button>
              ))}
            </div>
            
            {selectedStatus === "all" && (
              <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
                <span style={{ fontSize: "0.875rem", color: "#6b7280" }}>View:</span>
                <button
                  onClick={() => setViewMode("kanban")}
                  aria-label="Switch to kanban view"
                  aria-pressed={viewMode === "kanban"}
                  style={{
                    padding: "0.375rem 0.75rem",
                    backgroundColor: viewMode === "kanban" ? "#6366f1" : "#e5e7eb",
                    color: viewMode === "kanban" ? "white" : "#374151",
                    border: "none",
                    borderRadius: "0.375rem",
                    cursor: "pointer",
                    fontSize: "0.875rem"
                  }}
                >
                  Kanban
                </button>
                <button
                  onClick={() => setViewMode("table")}
                  aria-label="Switch to table view"
                  aria-pressed={viewMode === "table"}
                  style={{
                    padding: "0.375rem 0.75rem",
                    backgroundColor: viewMode === "table" ? "#6366f1" : "#e5e7eb",
                    color: viewMode === "table" ? "white" : "#374151",
                    border: "none",
                    borderRadius: "0.375rem",
                    cursor: "pointer",
                    fontSize: "0.875rem"
                  }}
                >
                  Table
                </button>
              </div>
            )}
          </div>

          {selectedStatus === "all" && viewMode === "kanban" ? (
            <div className="kanban-board">
              {statuses.map((status) => (
                <div key={status} className="kanban-column">
                  <div className="column-header">
                    <h3>{status}</h3>
                    <span className="count">{applicationsByStatus[status]?.length || 0}</span>
                  </div>
                  <div className="column-cards">
                    {applicationsByStatus[status]?.map((app) => (
                      <div
                        key={app.id}
                        className={`application-card ${selectedApp?.application.id === app.id ? "active" : ""}`}
                        onClick={() => loadApplicationDetail(app.id)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter" || e.key === " ") {
                            e.preventDefault();
                            loadApplicationDetail(app.id);
                          }
                        }}
                        tabIndex={0}
                        role="button"
                        aria-label={`Application: ${app.job_title || "Untitled"} at ${app.company || "Unknown Company"}`}
                      >
                        <div className="application-card-content">
                          <div className="application-header">
                            <h4>{app.job_title || "Untitled"}</h4>
                            <select
                              className="status-select-inline"
                              value={app.status}
                              onChange={async (e) => {
                                e.stopPropagation();
                                try {
                                  await invoke<Application>("update_application", {
                                    id: app.id,
                                    input: { status: e.target.value as ApplicationStatus },
                                  });
                                  loadApplications();
                                  if (selectedApp?.application.id === app.id) {
                                    loadApplicationDetail(app.id);
                                  }
                                } catch (err: any) {
                                  showToast(err?.message || "Failed to update status", "error");
                                }
                              }}
                              onClick={(e) => e.stopPropagation()}
                              aria-label={`Change status for ${app.job_title || 'Untitled'} application`}
                            >
                              <option value="Saved">Saved</option>
                              <option value="Draft">Draft</option>
                              <option value="Applied">Applied</option>
                              <option value="Interviewing">Interviewing</option>
                              <option value="Offer">Offer</option>
                              <option value="Rejected">Rejected</option>
                              <option value="Ghosted">Ghosted</option>
                              <option value="Withdrawn">Withdrawn</option>
                            </select>
                          </div>
                          <p className="company">{app.company || "Unknown Company"}</p>
                          {app.priority && (
                            <span className={`priority-badge ${app.priority.toLowerCase()}`}>
                              {app.priority}
                            </span>
                          )}
                          {app.date_applied && (
                            <p className="date">
                              Applied: {new Date(app.date_applied).toLocaleDateString()}
                            </p>
                          )}
                        </div>
                        <div className="application-card-actions" onClick={(e) => e.stopPropagation()}>
                          <button 
                            className="menu-button" 
                            title="More options"
                            aria-label={`More options for ${app.job_title || 'Untitled'} application`}
                          >
                            <span aria-hidden="true">⋯</span>
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          ) : selectedStatus === "all" && viewMode === "table" ? (
            <div className="table-view" style={{ flex: 1, overflow: "auto" }}>
              <table style={{ width: "100%", borderCollapse: "collapse", backgroundColor: "white", borderRadius: "0.5rem" }}>
                <thead>
                  <tr style={{ borderBottom: "2px solid #e5e7eb", backgroundColor: "#f9fafb" }}>
                    <th style={{ padding: "0.75rem", textAlign: "left", fontSize: "0.875rem", fontWeight: "600", color: "#6b7280" }}>Job Title</th>
                    <th style={{ padding: "0.75rem", textAlign: "left", fontSize: "0.875rem", fontWeight: "600", color: "#6b7280" }}>Company</th>
                    <th style={{ padding: "0.75rem", textAlign: "left", fontSize: "0.875rem", fontWeight: "600", color: "#6b7280" }}>Status</th>
                    <th style={{ padding: "0.75rem", textAlign: "left", fontSize: "0.875rem", fontWeight: "600", color: "#6b7280" }}>Priority</th>
                    <th style={{ padding: "0.75rem", textAlign: "left", fontSize: "0.875rem", fontWeight: "600", color: "#6b7280" }}>Date Applied</th>
                  </tr>
                </thead>
                <tbody>
                  {applications.map((app) => (
                    <tr
                      key={app.id}
                      onClick={() => loadApplicationDetail(app.id)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter" || e.key === " ") {
                          e.preventDefault();
                          loadApplicationDetail(app.id);
                        }
                      }}
                      tabIndex={0}
                      role="button"
                      aria-label={`Application: ${app.job_title || "Untitled"} at ${app.company || "Unknown Company"}`}
                      style={{
                        cursor: "pointer",
                        borderBottom: "1px solid #e5e7eb",
                        transition: "background-color 0.15s ease"
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.backgroundColor = "#f9fafb";
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.backgroundColor = "white";
                      }}
                      className={selectedApp?.application.id === app.id ? "active" : ""}
                    >
                      <td style={{ padding: "0.75rem", fontWeight: "500" }}>{app.job_title || "Untitled"}</td>
                      <td style={{ padding: "0.75rem", color: "#6b7280" }}>{app.company || "Unknown Company"}</td>
                      <td style={{ padding: "0.75rem" }}>
                        <select
                          value={app.status}
                          onChange={async (e) => {
                            e.stopPropagation();
                            try {
                              await invoke<Application>("update_application", {
                                id: app.id,
                                input: { status: e.target.value as ApplicationStatus },
                              });
                              loadApplications();
                              if (selectedApp?.application.id === app.id) {
                                loadApplicationDetail(app.id);
                              }
                            } catch (err: any) {
                              alert(err?.message || "Failed to update status");
                            }
                          }}
                          onClick={(e) => e.stopPropagation()}
                          style={{
                            padding: "0.25rem 0.5rem",
                            backgroundColor: "white",
                            border: "1px solid #e5e7eb",
                            borderRadius: "0.25rem",
                            fontSize: "0.8125rem",
                            cursor: "pointer"
                          }}
                        >
                          <option value="Saved">Saved</option>
                          <option value="Draft">Draft</option>
                          <option value="Applied">Applied</option>
                          <option value="Interviewing">Interviewing</option>
                          <option value="Offer">Offer</option>
                          <option value="Rejected">Rejected</option>
                          <option value="Ghosted">Ghosted</option>
                          <option value="Withdrawn">Withdrawn</option>
                        </select>
                      </td>
                      <td style={{ padding: "0.75rem" }}>
                        {app.priority && (
                          <span style={{
                            padding: "0.25rem 0.5rem",
                            borderRadius: "0.25rem",
                            fontSize: "0.75rem",
                            fontWeight: "500",
                            backgroundColor: app.priority === "Dream" ? "#fef3c7" : app.priority === "High" ? "#fee2e2" : app.priority === "Medium" ? "#dbeafe" : "#e5e7eb",
                            color: app.priority === "Dream" ? "#92400e" : app.priority === "High" ? "#991b1b" : app.priority === "Medium" ? "#1e40af" : "#374151"
                          }}>
                            {app.priority}
                          </span>
                        )}
                      </td>
                      <td style={{ padding: "0.75rem", color: "#6b7280", fontSize: "0.875rem" }}>
                        {app.date_applied ? new Date(app.date_applied).toLocaleDateString() : "-"}
                      </td>
                    </tr>
                  ))}
                  {applications.length === 0 && (
                    <tr>
                      <td colSpan={5} style={{ padding: "2rem", textAlign: "center", color: "#6b7280" }}>
                        No applications found
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
              {totalPages > 1 && (
                <div className="pagination">
                  <button
                    onClick={() => handlePageChange(currentPage - 1)}
                    disabled={currentPage === 1}
                    aria-label="Previous page"
                  >
                    Previous
                  </button>
                  <span className="pagination-info">
                    Page {currentPage} of {totalPages} ({total} total)
                  </span>
                  <button
                    onClick={() => handlePageChange(currentPage + 1)}
                    disabled={currentPage >= totalPages}
                    aria-label="Next page"
                  >
                    Next
                  </button>
                </div>
              )}
            </div>
          ) : (
            <div className="list-view">
              {applicationsByStatus[selectedStatus]?.length === 0 ? (
                <div className="empty-state">
                  <p>No applications with status "{selectedStatus}"</p>
                </div>
              ) : (
                applicationsByStatus[selectedStatus]?.map((app) => (
                  <div
                    key={app.id}
                    className={`application-card ${selectedApp?.application.id === app.id ? "active" : ""}`}
                    onClick={() => loadApplicationDetail(app.id)}
                  >
                    <div className="application-card-content">
                      <div className="application-header">
                        <h4>{app.job_title || "Untitled"}</h4>
                        <select
                          className="status-select-inline"
                          value={app.status}
                          onChange={async (e) => {
                            e.stopPropagation();
                            try {
                              await invoke<Application>("update_application", {
                                id: app.id,
                                input: { status: e.target.value as ApplicationStatus },
                              });
                              loadApplications();
                              if (selectedApp?.application.id === app.id) {
                                loadApplicationDetail(app.id);
                              }
                            } catch (err: any) {
                              alert(err?.message || "Failed to update status");
                            }
                          }}
                          onClick={(e) => e.stopPropagation()}
                        >
                          <option value="Saved">Saved</option>
                          <option value="Draft">Draft</option>
                          <option value="Applied">Applied</option>
                          <option value="Interviewing">Interviewing</option>
                          <option value="Offer">Offer</option>
                          <option value="Rejected">Rejected</option>
                          <option value="Ghosted">Ghosted</option>
                          <option value="Withdrawn">Withdrawn</option>
                        </select>
                      </div>
                      <p className="company">{app.company || "Unknown Company"}</p>
                      {app.priority && (
                        <span className={`priority-badge ${app.priority.toLowerCase()}`}>
                          {app.priority}
                        </span>
                      )}
                      {app.date_applied && (
                        <p className="date">
                          Applied: {new Date(app.date_applied).toLocaleDateString()}
                        </p>
                      )}
                    </div>
                    <div className="application-card-actions" onClick={(e) => e.stopPropagation()}>
                      <button className="menu-button" title="More options">
                        <span>⋯</span>
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}
        </div>

        <div className="application-detail-panel">
          {selectedApp ? (
            <ApplicationDetailView
              detail={selectedApp}
              onUpdate={() => {
                loadApplications();
                if (selectedApp.application.id) {
                  loadApplicationDetail(selectedApp.application.id);
                }
              }}
            />
          ) : (
            <div className="empty-detail">
              <p>Select an application to view details</p>
            </div>
          )}
        </div>
      </div>

      {showCreateModal && (
        <CreateApplicationSheet
          availableJobs={availableJobs}
          onClose={() => setShowCreateModal(false)}
          onSuccess={() => {
            setShowCreateModal(false);
            loadApplications();
          }}
        />
      )}
    </div>
  );
}

interface Artifact {
  id: number;
  application_id?: number;
  job_id?: number;
  type: string;
  title: string;
  content?: string;
  format?: string;
  ai_payload?: string;
  ai_model?: string;
  source?: string;
  version?: number;
  created_at: string;
  updated_at: string;
}

function ApplicationDetailView({
  detail,
  onUpdate,
}: {
  detail: ApplicationDetail;
  onUpdate: () => void;
}) {
  const [isEditing, setIsEditing] = useState(false);
  const [formData, setFormData] = useState<Application>(detail.application);
  const [isSaving, setIsSaving] = useState(false);
  const [showEventModal, setShowEventModal] = useState(false);
  const [artifacts, setArtifacts] = useState<Artifact[]>([]);
  const [selectedArtifact, setSelectedArtifact] = useState<Artifact | null>(null);
  const [isLoadingArtifacts, setIsLoadingArtifacts] = useState(false);
  const [portfolioItems, setPortfolioItems] = useState<PortfolioItem[]>([]);
  const [linkedPortfolioIds, setLinkedPortfolioIds] = useState<number[]>([]);
  const [isLoadingPortfolio, setIsLoadingPortfolio] = useState(false);
  const [showPortfolioLinkModal, setShowPortfolioLinkModal] = useState(false);
  const [emailThreads, setEmailThreads] = useState<EmailThread[]>([]);
  const [isLoadingEmailThreads, setIsLoadingEmailThreads] = useState(false);
  const [timelineFilter, setTimelineFilter] = useState<string>("all");
  const [timelineSort, setTimelineSort] = useState<"newest" | "oldest">("newest");

  useEffect(() => {
    setFormData(detail.application);
    loadArtifacts();
    loadPortfolio();
    loadEmailThreads();
  }, [detail]);

  async function loadArtifacts() {
    if (!detail.application.id) return;
    setIsLoadingArtifacts(true);
    try {
      const result = await invoke<Artifact[]>("get_artifacts_for_application", {
        application_id: detail.application.id,
      });
      setArtifacts(result);
    } catch (err: any) {
      console.error("Failed to load artifacts:", err);
    } finally {
      setIsLoadingArtifacts(false);
    }
  }

  async function loadPortfolio() {
    if (!detail.application.id) return;
    setIsLoadingPortfolio(true);
    try {
      // Load all portfolio items
      const profileData = await invoke<{ portfolio: PortfolioItem[] }>("get_user_profile_data");
      setPortfolioItems(profileData.portfolio);

      // Load linked portfolio items
      const linked = await invoke<PortfolioItem[]>("get_portfolio_for_application", {
        application_id: detail.application.id,
      });
      setLinkedPortfolioIds(linked.map(p => p.id!).filter((id): id is number => id !== undefined));
    } catch (err: any) {
      console.error("Failed to load portfolio:", err);
    } finally {
      setIsLoadingPortfolio(false);
    }
  }

  async function savePortfolioLinks() {
    if (!detail.application.id) return;
    try {
      await invoke("link_portfolio_to_application", {
        application_id: detail.application.id,
        portfolio_item_ids: linkedPortfolioIds,
      });
      showToast("Portfolio links updated", "success");
      setShowPortfolioLinkModal(false);
      loadPortfolio();
    } catch (err: any) {
      showToast(err?.message || "Failed to update portfolio links", "error");
    }
  }

  async function loadEmailThreads() {
    if (!detail.application.id) return;
    setIsLoadingEmailThreads(true);
    try {
      const threads = await invoke<EmailThread[]>("get_email_threads_for_application", {
        application_id: detail.application.id,
      });
      setEmailThreads(threads);
    } catch (err: any) {
      console.error("Failed to load email threads:", err);
    } finally {
      setIsLoadingEmailThreads(false);
    }
  }

  async function saveApplication() {
    setIsSaving(true);
    try {
      const updated = await invoke<Application>("update_application", {
        id: detail.application.id,
        input: {
          status: formData.status,
          channel: formData.channel,
          priority: formData.priority,
          date_applied: formData.date_applied,
          next_action_date: formData.next_action_date,
          next_action_note: formData.next_action_note,
          notes_summary: formData.notes_summary,
          contact_name: formData.contact_name,
          contact_email: formData.contact_email,
          contact_linkedin: formData.contact_linkedin,
          location_override: formData.location_override,
          offer_compensation: formData.offer_compensation,
        },
      });
      setFormData(updated);
      setIsEditing(false);
      onUpdate();
    } catch (err: any) {
      showToast(err?.message || "Failed to save application", "error");
    } finally {
      setIsSaving(false);
    }
  }

  async function archiveApplication() {
    if (!confirm("Are you sure you want to archive this application?")) return;
    try {
      await invoke<Application>("archive_application", { id: detail.application.id });
      onUpdate();
    } catch (err: any) {
      showToast(err?.message || "Failed to archive application", "error");
    }
  }

  return (
    <div className="application-detail-view">
      <div className="detail-header">
        <h2>Application Details</h2>
        <div className="detail-actions">
          {!isEditing && (
            <>
              <button onClick={() => setIsEditing(true)} className="edit-button">
                Edit
              </button>
              <button onClick={archiveApplication} className="archive-button">
                Archive
              </button>
            </>
          )}
          {isEditing && (
            <>
              <button onClick={() => setIsEditing(false)} className="cancel-button">
                Cancel
              </button>
              <button
                onClick={saveApplication}
                disabled={isSaving}
                className="save-button"
              >
                {isSaving ? "Saving..." : "Save"}
              </button>
            </>
          )}
        </div>
      </div>

      <div className="detail-content">
        <div className="detail-section">
          <h3>Status & Priority</h3>
          {isEditing ? (
            <div className="form-grid">
              <div className="form-group">
                <label>Status</label>
                <select
                  value={formData.status}
                  onChange={(e) =>
                    setFormData({ ...formData, status: e.target.value as ApplicationStatus })
                  }
                >
                  <option value="Saved">Saved</option>
                  <option value="Draft">Draft</option>
                  <option value="Applied">Applied</option>
                  <option value="Interviewing">Interviewing</option>
                  <option value="Offer">Offer</option>
                  <option value="Rejected">Rejected</option>
                  <option value="Ghosted">Ghosted</option>
                  <option value="Withdrawn">Withdrawn</option>
                </select>
              </div>
              <div className="form-group">
                <label>Priority</label>
                <select
                  value={formData.priority || ""}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      priority: e.target.value as "Low" | "Medium" | "High" | "Dream" | undefined,
                    })
                  }
                >
                  <option value="">None</option>
                  <option value="Low">Low</option>
                  <option value="Medium">Medium</option>
                  <option value="High">High</option>
                  <option value="Dream">Dream</option>
                </select>
              </div>
              <div className="form-group">
                <label>Channel</label>
                <input
                  type="text"
                  value={formData.channel || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, channel: e.target.value })
                  }
                  placeholder="LinkedIn, Company Site, etc."
                />
              </div>
            </div>
          ) : (
            <div className="info-grid">
              <div>
                <strong>Status:</strong> <span className={`status-badge ${formData.status.toLowerCase()}`}>{formData.status}</span>
              </div>
              {formData.priority && (
                <div>
                  <strong>Priority:</strong>{" "}
                  <span className={`priority-badge ${formData.priority.toLowerCase()}`}>
                    {formData.priority}
                  </span>
                </div>
              )}
              {formData.channel && (
                <div>
                  <strong>Channel:</strong> {formData.channel}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="detail-section">
          <div className="section-header-with-edit">
            <h3>Dates & Next Actions</h3>
            {!isEditing && (
              <button
                className="section-edit-button"
                onClick={() => setIsEditing(true)}
                title="Edit"
              >
                <span>✏️</span>
              </button>
            )}
          </div>
          <hr className="section-divider" />
          {isEditing ? (
            <div className="form-grid">
              <div className="form-group">
                <label>Date Applied</label>
                <input
                  type="date"
                  value={formData.date_applied?.split("T")[0] || ""}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      date_applied: e.target.value ? `${e.target.value}T00:00:00Z` : undefined,
                    })
                  }
                />
              </div>
              <div className="form-group">
                <label>Next Action Date</label>
                <input
                  type="date"
                  value={formData.next_action_date?.split("T")[0] || ""}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      next_action_date: e.target.value ? `${e.target.value}T00:00:00Z` : undefined,
                    })
                  }
                />
              </div>
              <div className="form-group full-width">
                <label>Next Action Note</label>
                <input
                  type="text"
                  value={formData.next_action_note || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, next_action_note: e.target.value })
                  }
                  placeholder="Follow up with recruiter..."
                />
              </div>
            </div>
          ) : (
            <div className="info-grid">
              <div>
                <strong>Date Saved:</strong>{" "}
                {new Date(formData.date_saved).toLocaleDateString()}
              </div>
              {formData.date_applied && (
                <div>
                  <strong>Date Applied:</strong>{" "}
                  {new Date(formData.date_applied).toLocaleDateString()}
                </div>
              )}
              {formData.last_activity_date && (
                <div>
                  <strong>Last Activity:</strong>{" "}
                  {new Date(formData.last_activity_date).toLocaleDateString()}
                </div>
              )}
              {formData.next_action_date && (
                <div>
                  <strong>Next Action:</strong>{" "}
                  {new Date(formData.next_action_date).toLocaleDateString()}
                  {formData.next_action_note && ` - ${formData.next_action_note}`}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="detail-section">
          <div className="section-header-with-edit">
            <h3>Contact Information</h3>
            {!isEditing && (
              <button
                className="section-edit-button"
                onClick={() => setIsEditing(true)}
                title="Edit"
              >
                <span>✏️</span>
              </button>
            )}
          </div>
          <hr className="section-divider" />
          {isEditing ? (
            <div className="form-grid">
              <div className="form-group">
                <label>Contact Name</label>
                <input
                  type="text"
                  value={formData.contact_name || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, contact_name: e.target.value })
                  }
                />
              </div>
              <div className="form-group">
                <label>Contact Email</label>
                <input
                  type="email"
                  value={formData.contact_email || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, contact_email: e.target.value })
                  }
                />
              </div>
              <div className="form-group full-width">
                <label>LinkedIn</label>
                <input
                  type="url"
                  value={formData.contact_linkedin || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, contact_linkedin: e.target.value })
                  }
                />
              </div>
            </div>
          ) : (
            <div className="info-grid">
              {formData.contact_name && (
                <div>
                  <strong>Name:</strong> {formData.contact_name}
                </div>
              )}
              {formData.contact_email && (
                <div>
                  <strong>Email:</strong>{" "}
                  <a href={`mailto:${formData.contact_email}`}>{formData.contact_email}</a>
                </div>
              )}
              {formData.contact_linkedin && (
                <div>
                  <strong>LinkedIn:</strong>{" "}
                  <a
                    href={formData.contact_linkedin}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {formData.contact_linkedin}
                  </a>
                </div>
              )}
              {!formData.contact_name && !formData.contact_email && !formData.contact_linkedin && (
                <div className="empty-text">No contact information</div>
              )}
            </div>
          )}
        </div>

        {formData.status === "Offer" && (
          <div className="detail-section">
            <div className="section-header-with-edit">
              <h3>Offer Details</h3>
              {!isEditing && (
                <button
                  className="section-edit-button"
                  onClick={() => setIsEditing(true)}
                  title="Edit"
                >
                  <span>✏️</span>
                </button>
              )}
            </div>
            <hr className="section-divider" />
            {isEditing ? (
              <div className="form-group">
                <label>Compensation</label>
                <input
                  type="text"
                  value={formData.offer_compensation || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, offer_compensation: e.target.value })
                  }
                  placeholder="$120,000 + equity"
                />
              </div>
            ) : (
              <div>
                {formData.offer_compensation ? (
                  <>
                    <strong>Compensation:</strong> {formData.offer_compensation}
                  </>
                ) : (
                  <div className="empty-text">No compensation details</div>
                )}
              </div>
            )}
          </div>
        )}

        <div className="detail-section">
          <div className="section-header-with-edit">
            <h3>Notes</h3>
            {!isEditing && (
              <button
                className="section-edit-button"
                onClick={() => setIsEditing(true)}
                title="Edit"
              >
                <span>✏️</span>
              </button>
            )}
          </div>
          <hr className="section-divider" />
          {isEditing ? (
            <textarea
              value={formData.notes_summary || ""}
              onChange={(e) =>
                setFormData({ ...formData, notes_summary: e.target.value })
              }
              rows={6}
              className="notes-textarea"
            />
          ) : (
            <div className="notes-display">
              <InlineEditable
                value={formData.notes_summary || ""}
                onSave={async (newValue) => {
                  try {
                    const updated = await invoke<Application>("update_application", {
                      id: detail.application.id,
                      input: {
                        notes_summary: newValue || null,
                      },
                    });
                    setFormData(updated);
                    onUpdate();
                  } catch (err: any) {
                    showToast(err?.message || "Failed to save notes", "error");
                  }
                }}
                placeholder="Click to add notes..."
                multiline={true}
                rows={4}
                className="notes-inline-editable"
              />
            </div>
          )}
        </div>

        <div className="detail-section">
          <div className="section-header-with-edit">
            <h3>Artifacts</h3>
          </div>
          <hr className="section-divider" />
          {isLoadingArtifacts ? (
            <div className="empty-text">Loading artifacts...</div>
          ) : artifacts.length === 0 ? (
            <div className="empty-text">No artifacts yet. Generate a resume or cover letter from the job page.</div>
          ) : (
            <div className="artifacts-list">
              {artifacts.map((artifact) => (
                <div
                  key={artifact.id}
                  className="artifact-item"
                  onClick={() => setSelectedArtifact(artifact)}
                >
                  <div className="artifact-header">
                    <span className="artifact-type-badge">{artifact.type}</span>
                    <span className="artifact-title">{artifact.title}</span>
                  </div>
                  <div className="artifact-meta">
                    {artifact.created_at && (
                      <span className="artifact-date">
                        Created: {new Date(artifact.created_at).toLocaleDateString()}
                      </span>
                    )}
                    {artifact.ai_model && (
                      <span className="artifact-model">Model: {artifact.ai_model}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Email Threads Section */}
        <div className="detail-section">
          <div className="section-header-with-edit">
            <h3>Email Threads</h3>
          </div>
          <hr className="section-divider" />
          {isLoadingEmailThreads ? (
            <div className="empty-text">Loading email threads...</div>
          ) : emailThreads.length === 0 ? (
            <div className="empty-text">No email threads linked to this application. Connect your email in Settings to automatically track application-related emails.</div>
          ) : (
            <div className="email-threads-list">
              {emailThreads.map((thread) => (
                <div key={thread.id} className="email-thread-card" style={{ padding: "1rem", border: "1px solid #e5e7eb", borderRadius: "0.5rem", marginBottom: "0.75rem" }}>
                  <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start", marginBottom: "0.5rem" }}>
                    <div>
                      <div style={{ fontWeight: "600", marginBottom: "0.25rem" }}>{thread.subject || "No Subject"}</div>
                      {thread.participants && (
                        <div style={{ fontSize: "0.875rem", color: "#6b7280" }}>{thread.participants}</div>
                      )}
                    </div>
                    <div style={{ fontSize: "0.875rem", color: "#6b7280" }}>
                      {thread.messageCount} message{thread.messageCount !== 1 ? "s" : ""}
                    </div>
                  </div>
                  {thread.lastMessageDate && (
                    <div style={{ fontSize: "0.75rem", color: "#9ca3af" }}>
                      Last message: {new Date(thread.lastMessageDate).toLocaleDateString()}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="detail-section">
          <div className="section-header">
            <h3>Timeline</h3>
            <div style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
              {/* Filter dropdown */}
              <select
                value={timelineFilter}
                onChange={(e) => setTimelineFilter(e.target.value)}
                style={{
                  padding: "0.375rem 0.75rem",
                  backgroundColor: "white",
                  border: "1px solid #e5e7eb",
                  borderRadius: "0.375rem",
                  fontSize: "0.875rem",
                  cursor: "pointer"
                }}
              >
                <option value="all">All Events</option>
                <option value="InterviewScheduled">Interviews Scheduled</option>
                <option value="InterviewCompleted">Interviews Completed</option>
                <option value="OfferReceived">Offers Received</option>
                <option value="Rejected">Rejections</option>
                <option value="FollowUpSent">Follow Ups</option>
              </select>
              
              {/* Sort dropdown */}
              <select
                value={timelineSort}
                onChange={(e) => setTimelineSort(e.target.value as "newest" | "oldest")}
                style={{
                  padding: "0.375rem 0.75rem",
                  backgroundColor: "white",
                  border: "1px solid #e5e7eb",
                  borderRadius: "0.375rem",
                  fontSize: "0.875rem",
                  cursor: "pointer"
                }}
              >
                <option value="newest">Newest First</option>
                <option value="oldest">Oldest First</option>
              </select>
              
              <button
                onClick={() => setShowEventModal(true)}
                className="add-event-button"
              >
                + Add Event
              </button>
            </div>
          </div>
          <div className="timeline">
            {(() => {
              // Filter events
              let filteredEvents = detail.events;
              if (timelineFilter !== "all") {
                filteredEvents = filteredEvents.filter(
                  (event) => event.event_type === timelineFilter
                );
              }
              
              // Sort events
              filteredEvents = [...filteredEvents].sort((a, b) => {
                const dateA = new Date(a.event_date).getTime();
                const dateB = new Date(b.event_date).getTime();
                return timelineSort === "newest" ? dateB - dateA : dateA - dateB;
              });
              
              if (filteredEvents.length === 0) {
                return <div className="empty-text">No events found</div>;
              }
              
              return filteredEvents.map((event, index) => {
                const eventDate = new Date(event.event_date);
                const isRecent = (Date.now() - eventDate.getTime()) < 7 * 24 * 60 * 60 * 1000; // Last 7 days
                const isToday = eventDate.toDateString() === new Date().toDateString();
                
                return (
                  <div 
                    key={event.id || index} 
                    className="timeline-item"
                    style={{
                      position: "relative",
                      paddingLeft: "2rem",
                      paddingBottom: "1.5rem",
                      borderLeft: index < filteredEvents.length - 1 ? "2px solid #e5e7eb" : "none"
                    }}
                  >
                    {/* Timeline dot */}
                    <div style={{
                      position: "absolute",
                      left: "-6px",
                      top: "0.25rem",
                      width: "12px",
                      height: "12px",
                      borderRadius: "50%",
                      backgroundColor: isRecent ? "#6366f1" : "#9ca3af",
                      border: "2px solid white",
                      boxShadow: "0 0 0 2px " + (isRecent ? "#6366f1" : "#9ca3af")
                    }}></div>
                    
                    <div className="timeline-date" style={{
                      fontSize: "0.875rem",
                      color: "#6b7280",
                      marginBottom: "0.5rem",
                      fontWeight: isToday ? "600" : "400"
                    }}>
                      {isToday ? "Today" : eventDate.toLocaleDateString("en-US", {
                        month: "short",
                        day: "numeric",
                        year: "numeric"
                      })}
                      {!isToday && (
                        <span style={{ marginLeft: "0.5rem", color: "#9ca3af" }}>
                          ({eventDate.toLocaleTimeString("en-US", { hour: "numeric", minute: "2-digit" })})
                        </span>
                      )}
                    </div>
                    
                    <div className="timeline-content" style={{
                      backgroundColor: "#f9fafb",
                      padding: "0.75rem",
                      borderRadius: "0.375rem",
                      border: "1px solid #e5e7eb"
                    }}>
                      <div className="timeline-type" style={{
                        fontSize: "0.875rem",
                        fontWeight: "600",
                        color: "#374151",
                        marginBottom: "0.25rem",
                        textTransform: "capitalize"
                      }}>
                        {event.event_type.replace(/([A-Z])/g, " $1").trim()}
                      </div>
                      {event.title && (
                        <div className="timeline-title" style={{
                          fontSize: "0.9375rem",
                          fontWeight: "500",
                          color: "#111827",
                          marginBottom: "0.25rem"
                        }}>
                          {event.title}
                        </div>
                      )}
                      {event.from_status && event.to_status && (
                        <div className="timeline-status-change" style={{
                          fontSize: "0.8125rem",
                          color: "#6366f1",
                          marginBottom: "0.25rem",
                          padding: "0.25rem 0.5rem",
                          backgroundColor: "#eef2ff",
                          borderRadius: "0.25rem",
                          display: "inline-block"
                        }}>
                          {event.from_status} → {event.to_status}
                        </div>
                      )}
                      {event.details && (
                        <div className="timeline-details" style={{
                          fontSize: "0.875rem",
                          color: "#6b7280",
                          marginTop: "0.5rem",
                          lineHeight: "1.5"
                        }}>
                          {event.details}
                        </div>
                      )}
                    </div>
                  </div>
                );
              });
            })()}
          </div>
        </div>
      </div>

      {showEventModal && (
        <AddEventModal
          applicationId={detail.application.id!}
          onClose={() => setShowEventModal(false)}
          onSuccess={() => {
            setShowEventModal(false);
            onUpdate();
          }}
        />
      )}

      {selectedArtifact && (
        <ArtifactViewSheet
          artifact={selectedArtifact}
          onClose={() => setSelectedArtifact(null)}
          onUpdate={() => {
            loadArtifacts();
            onUpdate();
          }}
        />
      )}

      {showPortfolioLinkModal && (
        <PortfolioLinkModal
          portfolioItems={portfolioItems}
          linkedIds={linkedPortfolioIds}
          onUpdate={(ids) => setLinkedPortfolioIds(ids)}
          onSave={savePortfolioLinks}
          onClose={() => setShowPortfolioLinkModal(false)}
        />
      )}
    </div>
  );
}

function PortfolioLinkModal({
  portfolioItems,
  linkedIds,
  onUpdate,
  onSave,
  onClose,
}: {
  portfolioItems: PortfolioItem[];
  linkedIds: number[];
  onUpdate: (ids: number[]) => void;
  onSave: () => void;
  onClose: () => void;
}) {
  function togglePortfolioItem(id: number) {
    if (linkedIds.includes(id)) {
      onUpdate(linkedIds.filter(lid => lid !== id));
    } else {
      onUpdate([...linkedIds, id]);
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>Link Portfolio Items</h3>
          <button onClick={onClose} aria-label="Close">×</button>
        </div>
        <div className="modal-body">
          {portfolioItems.length === 0 ? (
            <p>No portfolio items available. Add portfolio items in your Profile.</p>
          ) : (
            <div className="portfolio-link-list">
              {portfolioItems.map((item) => (
                <label key={item.id} className="portfolio-link-item">
                  <input
                    type="checkbox"
                    checked={item.id ? linkedIds.includes(item.id) : false}
                    onChange={() => item.id && togglePortfolioItem(item.id)}
                  />
                  <div className="portfolio-link-info">
                    <div className="portfolio-link-title">
                      {item.title}
                      {item.highlighted && <span className="highlight-badge">★</span>}
                    </div>
                    {item.role && (
                      <div className="portfolio-link-role">{item.role}</div>
                    )}
                  </div>
                </label>
              ))}
            </div>
          )}
        </div>
        <div className="modal-actions">
          <button onClick={onClose}>Cancel</button>
          <button onClick={onSave} className="save-button">Save</button>
        </div>
      </div>
    </div>
  );
}

function ArtifactViewSheet({
  artifact,
  onClose,
  onUpdate,
}: {
  artifact: Artifact;
  onClose: () => void;
  onUpdate: () => void;
}) {
  const [isEditing, setIsEditing] = useState(false);
  const [content, setContent] = useState(artifact.content || "");
  const [isSaving, setIsSaving] = useState(false);

  // Handle Escape key to close sheet
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape" && !isEditing) {
        onClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose, isEditing]);

  async function saveArtifact() {
    setIsSaving(true);
    try {
      await invoke<Artifact>("update_artifact", {
        id: artifact.id,
        content: content,
      });
      setIsEditing(false);
      onUpdate();
    } catch (err: any) {
      showToast(err?.message || "Failed to save artifact", "error");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <>
      <div className="sheet-overlay" onClick={onClose}></div>
      <div className="sheet artifact-sheet">
        <div className="sheet-header">
          <h2>{artifact.title}</h2>
          <button onClick={onClose} className="sheet-close-button">
            ×
          </button>
        </div>
        <div className="sheet-content">
          <div className="artifact-meta-info">
            <div>
              <strong>Type:</strong> {artifact.type}
            </div>
            {artifact.ai_model && (
              <div>
                <strong>Model:</strong> {artifact.ai_model}
              </div>
            )}
            {artifact.created_at && (
              <div>
                <strong>Created:</strong> {new Date(artifact.created_at).toLocaleString()}
              </div>
            )}
            {artifact.updated_at && (
              <div>
                <strong>Updated:</strong> {new Date(artifact.updated_at).toLocaleString()}
              </div>
            )}
          </div>
          <div className="artifact-content-section">
            <div className="section-header-with-edit">
              <h3>Content</h3>
              {!isEditing && (
                <button
                  className="section-edit-button"
                  onClick={() => setIsEditing(true)}
                  title="Edit"
                >
                  <span>✏️</span>
                </button>
              )}
            </div>
            <hr className="section-divider" />
            {isEditing ? (
              <textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={20}
                className="artifact-content-textarea"
              />
            ) : (
              <pre className="artifact-content-display">{content || "No content"}</pre>
            )}
          </div>
        </div>
        <div className="sheet-footer">
          {isEditing ? (
            <>
              <button onClick={() => setIsEditing(false)} className="cancel-button">
                Cancel
              </button>
              <button
                onClick={saveArtifact}
                disabled={isSaving}
                className="save-button"
              >
                {isSaving ? "Saving..." : "Save"}
              </button>
            </>
          ) : (
            <button onClick={onClose} className="save-button">
              Close
            </button>
          )}
        </div>
      </div>
    </>
  );
}

function CreateApplicationSheet({
  availableJobs,
  onClose,
  onSuccess,
}: {
  availableJobs: Job[];
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [formData, setFormData] = useState({
    job_id: 0,
    status: "Saved" as ApplicationStatus,
    channel: "",
    priority: "" as "Low" | "Medium" | "High" | "Dream" | "",
  });
  const [isSaving, setIsSaving] = useState(false);

  // Handle Escape key to close sheet
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  // Handle Enter key to submit form (but not in textarea)
  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && e.target instanceof HTMLInputElement && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  async function handleSubmit() {
    if (formData.job_id === 0) {
      showToast("Please select a job", "warning");
      return;
    }

    setIsSaving(true);
    try {
      await invoke<Application>("create_application", {
        input: {
          jobId: formData.job_id,
          status: formData.status,
          channel: formData.channel || null,
          priority: formData.priority || null,
        },
      });
      showToast("Application created successfully", "success");
      onSuccess();
    } catch (err: any) {
      showToast(err?.message || "Failed to create application", "error");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <>
      <div className="sheet-overlay" onClick={onClose}></div>
      <div className="sheet-container" onClick={(e) => e.stopPropagation()}>
        <div className="sheet-header">
          <h2>Create Application</h2>
          <button onClick={onClose} className="sheet-close-button">
            ×
          </button>
        </div>
        <div className="sheet-body">
          <div className="form-grid">
            <div className="form-group full-width">
              <label>
                Job <span className="required">*</span>
              </label>
              <select
                value={formData.job_id}
                onChange={(e) =>
                  setFormData({ ...formData, job_id: parseInt(e.target.value) })
                }
                required
              >
                <option value={0}>Select a job...</option>
                {availableJobs.map((job) => (
                  <option key={job.id} value={job.id}>
                    {job.title || "Untitled"} @ {job.company || "Unknown"}
                  </option>
                ))}
              </select>
            </div>
            <div className="form-group">
              <label>Status</label>
              <select
                value={formData.status}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    status: e.target.value as ApplicationStatus,
                  })
                }
              >
                <option value="Saved">Saved</option>
                <option value="Draft">Draft</option>
                <option value="Applied">Applied</option>
              </select>
            </div>
            <div className="form-group">
              <label>Priority</label>
              <select
                value={formData.priority}
                onChange={(e) =>
                  setFormData({
                    ...formData,
                    priority: e.target.value as "Low" | "Medium" | "High" | "Dream" | "",
                  })
                }
              >
                <option value="">None</option>
                <option value="Low">Low</option>
                <option value="Medium">Medium</option>
                <option value="High">High</option>
                <option value="Dream">Dream</option>
              </select>
            </div>
            <div className="form-group">
              <label>Channel</label>
              <input
                type="text"
                value={formData.channel}
                onChange={(e) =>
                  setFormData({ ...formData, channel: e.target.value })
                }
                onKeyDown={handleKeyDown}
                placeholder="LinkedIn, Company Site, etc."
              />
            </div>
          </div>
        </div>
        <div className="sheet-footer">
          <button onClick={onClose} className="cancel-button">
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isSaving}
            className="save-button"
          >
            {isSaving ? "Creating..." : "Create"}
          </button>
        </div>
      </div>
    </>
  );
}

function AddEventModal({
  applicationId,
  onClose,
  onSuccess,
}: {
  applicationId: number;
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [formData, setFormData] = useState({
    event_type: "InterviewScheduled",
    event_date: new Date().toISOString().split("T")[0],
    title: "",
    details: "",
  });
  const [isSaving, setIsSaving] = useState(false);

  // Handle Escape key to close sheet
  useEffect(() => {
    function handleKeyDown(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [onClose]);

  // Handle Enter key to submit form (but not in textarea)
  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Enter" && e.target instanceof HTMLInputElement && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  async function handleSubmit() {
    setIsSaving(true);
    try {
      await invoke<ApplicationEvent>("add_application_event", {
        input: {
          applicationId: applicationId,
          eventType: formData.event_type,
          eventDate: `${formData.event_date}T00:00:00Z`,
          title: formData.title || null,
          details: formData.details || null,
        },
      });
      onSuccess();
    } catch (err: any) {
      showToast(err?.message || "Failed to add event", "error");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <>
      <div className="sheet-overlay" onClick={onClose}></div>
      <div className="sheet-container" onClick={(e) => e.stopPropagation()}>
        <div className="sheet-header">
          <h2>Add Event</h2>
          <button onClick={onClose} className="sheet-close-button">
            ×
          </button>
        </div>
        <div className="sheet-body">
          <div className="form-grid">
            <div className="form-group">
              <label>Event Type</label>
              <select
                value={formData.event_type}
                onChange={(e) =>
                  setFormData({ ...formData, event_type: e.target.value })
                }
              >
                <option value="InterviewScheduled">Interview Scheduled</option>
                <option value="InterviewCompleted">Interview Completed</option>
                <option value="FollowUpSent">Follow Up Sent</option>
                <option value="FeedbackReceived">Feedback Received</option>
                <option value="OfferReceived">Offer Received</option>
                <option value="OfferAccepted">Offer Accepted</option>
                <option value="OfferDeclined">Offer Declined</option>
                <option value="Rejected">Rejected</option>
                <option value="MarkedGhosted">Marked as Ghosted</option>
              </select>
            </div>
            <div className="form-group">
              <label>Event Date</label>
              <input
                type="date"
                value={formData.event_date}
                onChange={(e) =>
                  setFormData({ ...formData, event_date: e.target.value })
                }
              />
            </div>
            <div className="form-group full-width">
              <label>Title</label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
                onKeyDown={handleKeyDown}
                placeholder="e.g., Recruiter Phone Screen"
              />
            </div>
            <div className="form-group full-width">
              <label>Details</label>
              <textarea
                value={formData.details}
                onChange={(e) =>
                  setFormData({ ...formData, details: e.target.value })
                }
                onKeyDown={(e) => {
                  // Allow Enter in textarea, but Cmd/Ctrl+Enter submits
                  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                    e.preventDefault();
                    handleSubmit();
                  }
                }}
                rows={4}
                placeholder="Additional notes about this event..."
              />
            </div>
          </div>
        </div>
        <div className="sheet-footer">
          <button 
            onClick={onClose} 
            className="cancel-button"
            aria-label="Cancel adding event"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isSaving}
            className="save-button"
            aria-label={isSaving ? "Adding event" : "Add event"}
          >
            {isSaving ? "Adding..." : "Add Event"}
          </button>
        </div>
      </div>
    </>
  );
}
