import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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

export default function Applications() {
  const [applications, setApplications] = useState<ApplicationSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [selectedStatus, setSelectedStatus] = useState<ApplicationStatus | "all">("all");
  const [selectedApp, setSelectedApp] = useState<ApplicationDetail | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [availableJobs, setAvailableJobs] = useState<Job[]>([]);

  useEffect(() => {
    loadApplications();
    loadAvailableJobs();
  }, [selectedStatus]);

  async function loadApplications() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<ApplicationSummary[]>("get_applications", {
        status: selectedStatus === "all" ? null : selectedStatus,
        jobId: null,
        activeOnly: true,
      });
      setApplications(result);
    } catch (err: any) {
      setError(err?.message || "Failed to load applications");
    } finally {
      setIsLoading(false);
    }
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
        <div className="loading">Loading applications...</div>
      </div>
    );
  }

  return (
    <div className="applications">
      <div className="applications-header">
        <h1>Applications</h1>
        <button onClick={() => setShowCreateModal(true)} className="add-button">
          + Create Application
        </button>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="applications-layout">
        <div className="pipeline-view">
          <div className="status-filters">
            <button
              className={selectedStatus === "all" ? "active" : ""}
              onClick={() => setSelectedStatus("all")}
            >
              All
            </button>
            {statuses.map((status) => (
              <button
                key={status}
                className={selectedStatus === status ? "active" : ""}
                onClick={() => setSelectedStatus(status)}
              >
                {status} ({applicationsByStatus[status]?.length || 0})
              </button>
            ))}
          </div>

          {selectedStatus === "all" ? (
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
                    ))}
                  </div>
                </div>
              ))}
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

  useEffect(() => {
    setFormData(detail.application);
  }, [detail]);

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
      alert(err?.message || "Failed to save application");
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
      alert(err?.message || "Failed to archive application");
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
              {formData.notes_summary || (
                <p className="empty-text">No notes</p>
              )}
            </div>
          )}
        </div>

        <div className="detail-section">
          <div className="section-header">
            <h3>Timeline</h3>
            <button
              onClick={() => setShowEventModal(true)}
              className="add-event-button"
            >
              + Add Event
            </button>
          </div>
          <div className="timeline">
            {detail.events.length === 0 ? (
              <div className="empty-text">No events yet</div>
            ) : (
              detail.events.map((event) => (
                <div key={event.id || Math.random()} className="timeline-item">
                  <div className="timeline-date">
                    {new Date(event.event_date).toLocaleDateString()}
                  </div>
                  <div className="timeline-content">
                    <div className="timeline-type">{event.event_type}</div>
                    {event.title && <div className="timeline-title">{event.title}</div>}
                    {event.from_status && event.to_status && (
                      <div className="timeline-status-change">
                        {event.from_status} → {event.to_status}
                      </div>
                    )}
                    {event.details && (
                      <div className="timeline-details">{event.details}</div>
                    )}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {showEventModal && (
        <AddEventSheet
          applicationId={detail.application.id!}
          onClose={() => setShowEventModal(false)}
          onSuccess={() => {
            setShowEventModal(false);
            onUpdate();
          }}
        />
      )}
    </div>
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

  async function handleSubmit() {
    if (formData.job_id === 0) {
      alert("Please select a job");
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
      onSuccess();
    } catch (err: any) {
      alert(err?.message || "Failed to create application");
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
      alert(err?.message || "Failed to add event");
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
                rows={4}
                placeholder="Additional notes about this event..."
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
            {isSaving ? "Adding..." : "Add Event"}
          </button>
        </div>
      </div>
    </>
  );
}
