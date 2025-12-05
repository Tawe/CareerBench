import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import "./Jobs.css";

interface JobSummary {
  id: number;
  title?: string;
  company?: string;
  location?: string;
  seniority?: string;
  domain_tags?: string;
  date_added: string;
}

interface Job {
  id?: number;
  title?: string;
  company?: string;
  location?: string;
  job_source?: string;
  posting_url?: string;
  raw_description?: string;
  parsed_json?: string;
  seniority?: string;
  domain_tags?: string;
  is_active: boolean;
  date_added: string;
  last_updated: string;
}

interface ParsedJob {
  titleSuggestion?: string | null;
  companySuggestion?: string | null;
  seniority?: string | null;
  location?: string | null;
  summary?: string | null;
  responsibilities: string[];
  requiredSkills: string[];
  niceToHaveSkills: string[];
  domainTags: string[];
  seniorityScore?: number | null;
  remoteFriendly?: boolean | null;
}

export default function Jobs() {
  const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [activeOnly, setActiveOnly] = useState(true);
  const [selectedJob, setSelectedJob] = useState<Job | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);

  useEffect(() => {
    loadJobs();
  }, [searchTerm, activeOnly]);

  async function loadJobs() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<JobSummary[]>("get_job_list", {
        search: searchTerm || null,
        activeOnly: activeOnly,
        source: null,
      });
      setJobs(result);
    } catch (err: any) {
      setError(err?.message || "Failed to load jobs");
    } finally {
      setIsLoading(false);
    }
  }

  async function loadJobDetail(id: number) {
    try {
      const job = await invoke<Job>("get_job_detail", { id });
      setSelectedJob(job);
    } catch (err: any) {
      setError(err?.message || "Failed to load job details");
    }
  }

  function handleJobClick(job: JobSummary) {
    loadJobDetail(job.id);
  }

  if (isLoading && jobs.length === 0) {
    return (
      <div className="jobs">
        <div className="loading">Loading jobs...</div>
      </div>
    );
  }

  return (
    <div className="jobs">
      <div className="jobs-header">
        <h1>Jobs</h1>
        <button onClick={() => setShowAddModal(true)} className="add-button">
          + Add Job
        </button>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="jobs-layout">
        <div className="jobs-sidebar">
          <div className="filters">
            <input
              type="text"
              placeholder="Search jobs..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="search-input"
            />
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={activeOnly}
                onChange={(e) => setActiveOnly(e.target.checked)}
              />
              Active only
            </label>
          </div>

          <div className="jobs-list">
            {jobs.length === 0 ? (
              <div className="empty-state">
                <p>No jobs found. Click "Add Job" to get started.</p>
              </div>
            ) : (
              jobs.map((job) => (
                <div
                  key={job.id}
                  className={`job-card ${selectedJob?.id === job.id ? "active" : ""}`}
                  onClick={() => handleJobClick(job)}
                >
                  <div className="job-card-content">
                    <h3>{job.title || "Untitled"}</h3>
                    <p className="job-company">{job.company || "Unknown Company"}</p>
                    {job.location && <p className="job-location">{job.location}</p>}
                    {job.seniority && (
                      <span className="job-badge">{job.seniority}</span>
                    )}
                    <p className="job-date">
                      {new Date(job.date_added).toLocaleDateString()}
                    </p>
                  </div>
                  <div className="job-card-actions" onClick={(e) => e.stopPropagation()}>
                    <div className="action-menu">
                      <button
                        className="action-button"
                        onClick={() => {
                          handleJobClick(job);
                          // Trigger parse action
                        }}
                        title="Parse with AI"
                      >
                        <span>🤖</span>
                      </button>
                      <button
                        className="action-button"
                        onClick={() => {
                          // Create application from job
                        }}
                        title="Create Application"
                      >
                        <span>📝</span>
                      </button>
                    </div>
                    <button className="menu-button" title="More options">
                      <span>⋯</span>
                    </button>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        <div className="job-detail">
          {selectedJob ? (
            <div className="content-constrained">
              <JobDetailView job={selectedJob} onUpdate={loadJobs} />
            </div>
          ) : (
            <div className="empty-detail">
              <p>Select a job to view details</p>
            </div>
          )}
        </div>
      </div>

      {showAddModal && (
        <AddJobSheet
          onClose={() => setShowAddModal(false)}
          onSuccess={() => {
            setShowAddModal(false);
            loadJobs();
          }}
        />
      )}
    </div>
  );
}

function JobDetailView({
  job,
  onUpdate,
}: {
  job: Job;
  onUpdate: () => void;
}) {
  const [isEditing, setIsEditing] = useState(false);
  const [formData, setFormData] = useState<Job>(job);
  const [isSaving, setIsSaving] = useState(false);
  const [isParsing, setIsParsing] = useState(false);
  const [parsedData, setParsedData] = useState<ParsedJob | null>(null);
  const [showGenerateModal, setShowGenerateModal] = useState(false);

  useEffect(() => {
    setFormData(job);
    if (job.parsed_json) {
      try {
        setParsedData(JSON.parse(job.parsed_json));
      } catch (e) {
        setParsedData(null);
      }
    } else {
      setParsedData(null);
    }
  }, [job]);

  async function saveJob() {
    setIsSaving(true);
    try {
      const updated = await invoke<Job>("update_job", {
        id: job.id,
        input: {
          title: formData.title,
          company: formData.company,
          location: formData.location,
          job_source: formData.job_source,
          posting_url: formData.posting_url,
          raw_description: formData.raw_description,
          is_active: formData.is_active,
        },
      });
      setFormData(updated);
      setIsEditing(false);
      onUpdate();
    } catch (err: any) {
      alert(err?.message || "Failed to save job");
    } finally {
      setIsSaving(false);
    }
  }

  async function parseWithAI() {
    if (!job.id) return;
    setIsParsing(true);
    try {
      const parsed = await invoke<ParsedJob>("parse_job_with_ai", { jobId: job.id });
      setParsedData(parsed);
      // Reload job to get updated parsed_json
      const updated = await invoke<Job>("get_job_detail", { id: job.id });
      setFormData(updated);
      onUpdate();
    } catch (err: any) {
      alert(err?.message || "Failed to parse job");
    } finally {
      setIsParsing(false);
    }
  }

  return (
    <div className="job-detail-view">
      <div className="detail-header">
        <h2>{formData.title || "Untitled Job"}</h2>
        <div className="detail-actions">
              {!isEditing && (
                <>
                  <button onClick={() => setIsEditing(true)} className="edit-button">
                    Edit
                  </button>
                  {formData.raw_description && (
                    <button
                      onClick={parseWithAI}
                      disabled={isParsing}
                      className="parse-button"
                    >
                      {isParsing ? "Parsing..." : "Parse with AI"}
                    </button>
                  )}
                  <button
                    onClick={() => setShowGenerateModal(true)}
                    className="generate-button"
                  >
                    Generate Resume/Cover Letter
                  </button>
                </>
              )}
          {isEditing && (
            <>
              <button onClick={() => setIsEditing(false)} className="cancel-button">
                Cancel
              </button>
              <button
                onClick={saveJob}
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
          <div className="section-header-with-edit">
            <h3>Basic Information</h3>
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
                <label>Title</label>
                <input
                  type="text"
                  value={formData.title || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, title: e.target.value })
                  }
                />
              </div>
              <div className="form-group">
                <label>Company</label>
                <input
                  type="text"
                  value={formData.company || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, company: e.target.value })
                  }
                />
              </div>
              <div className="form-group">
                <label>Location</label>
                <input
                  type="text"
                  value={formData.location || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, location: e.target.value })
                  }
                />
              </div>
              <div className="form-group">
                <label>Source</label>
                <select
                  value={formData.job_source || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, job_source: e.target.value })
                  }
                >
                  <option value="">Select...</option>
                  <option value="LinkedIn">LinkedIn</option>
                  <option value="Company Site">Company Site</option>
                  <option value="Referral">Referral</option>
                  <option value="Other">Other</option>
                </select>
              </div>
              <div className="form-group full-width">
                <label>Posting URL</label>
                <input
                  type="url"
                  value={formData.posting_url || ""}
                  onChange={(e) =>
                    setFormData({ ...formData, posting_url: e.target.value })
                  }
                />
              </div>
            </div>
          ) : (
            <div className="info-grid">
              <div>
                <strong>Company:</strong> {formData.company || "N/A"}
              </div>
              <div>
                <strong>Location:</strong> {formData.location || "N/A"}
              </div>
              <div>
                <strong>Source:</strong> {formData.job_source || "N/A"}
              </div>
              {formData.posting_url && (
                <div>
                  <strong>URL:</strong>{" "}
                  <a
                    href={formData.posting_url}
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    {formData.posting_url}
                  </a>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="detail-section">
          <div className="section-header-with-edit">
            <h3>Job Description</h3>
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
              value={formData.raw_description || ""}
              onChange={(e) =>
                setFormData({ ...formData, raw_description: e.target.value })
              }
              rows={15}
              className="description-textarea"
            />
          ) : (
            <div className="description-display">
              {formData.raw_description || (
                <p className="empty-text">No description provided</p>
              )}
            </div>
          )}
        </div>

        {parsedData && (
          <div className="detail-section">
            <h3>Parsed Insights</h3>
            <hr className="section-divider" />
            <div className="parsed-content">
              {parsedData.titleSuggestion && (
                <div>
                  <strong>Suggested Title:</strong> {parsedData.titleSuggestion}
                </div>
              )}
              {parsedData.companySuggestion && (
                <div>
                  <strong>Suggested Company:</strong> {parsedData.companySuggestion}
                </div>
              )}
              {parsedData.seniority && (
                <div>
                  <strong>Seniority:</strong> {parsedData.seniority}
                </div>
              )}
              {parsedData.location && (
                <div>
                  <strong>Location:</strong> {parsedData.location}
                </div>
              )}
              {parsedData.remoteFriendly !== null && (
                <div>
                  <strong>Remote Friendly:</strong> {parsedData.remoteFriendly ? "Yes" : "No"}
                </div>
              )}
              {parsedData.summary && (
                <div>
                  <strong>Summary:</strong>
                  <p>{parsedData.summary}</p>
                </div>
              )}
              {parsedData.responsibilities && parsedData.responsibilities.length > 0 && (
                <div>
                  <strong>Responsibilities:</strong>
                  <ul>
                    {parsedData.responsibilities.map((r: string, i: number) => (
                      <li key={i}>{r}</li>
                    ))}
                  </ul>
                </div>
              )}
              {parsedData.requiredSkills && parsedData.requiredSkills.length > 0 && (
                <div>
                  <strong>Required Skills:</strong>
                  <div className="tags">
                    {parsedData.requiredSkills.map((s: string, i: number) => (
                      <span key={i} className="tag">
                        {s}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              {parsedData.niceToHaveSkills && parsedData.niceToHaveSkills.length > 0 && (
                <div>
                  <strong>Nice to Have:</strong>
                  <div className="tags">
                    {parsedData.niceToHaveSkills.map((s: string, i: number) => (
                      <span key={i} className="tag">
                        {s}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              {parsedData.domainTags && parsedData.domainTags.length > 0 && (
                <div>
                  <strong>Domain Tags:</strong>
                  <div className="tags">
                    {parsedData.domainTags.map((tag: string, i: number) => (
                      <span key={i} className="tag">
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}
              {parsedData.seniorityScore !== null && parsedData.seniorityScore !== undefined && (
                <div>
                  <strong>Seniority Score:</strong> {(parsedData.seniorityScore * 100).toFixed(0)}%
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {showGenerateModal && (
        <GenerateResumeSheet
          jobId={job.id!}
          jobTitle={job.title || "Untitled"}
          company={job.company || "Unknown"}
          onClose={() => setShowGenerateModal(false)}
        />
      )}
    </div>
  );
}

function AddJobSheet({
  onClose,
  onSuccess,
}: {
  onClose: () => void;
  onSuccess: () => void;
}) {
  const [formData, setFormData] = useState({
    title: "",
    company: "",
    location: "",
    job_source: "",
    posting_url: "",
    raw_description: "",
  });
  const [isSaving, setIsSaving] = useState(false);

  async function handleSubmit() {
    if (!formData.title && !formData.company && !formData.raw_description) {
      alert("Please provide at least a title, company, or description");
      return;
    }

    setIsSaving(true);
    try {
      await invoke<Job>("create_job", {
        input: {
          title: formData.title || null,
          company: formData.company || null,
          location: formData.location || null,
          job_source: formData.job_source || null,
          posting_url: formData.posting_url || null,
          raw_description: formData.raw_description || null,
        },
      });
      onSuccess();
    } catch (err: any) {
      alert(err?.message || "Failed to create job");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <>
      <div className="sheet-overlay" onClick={onClose}></div>
      <div className="sheet-container" onClick={(e) => e.stopPropagation()}>
        <div className="sheet-header">
          <h2>Add Job</h2>
          <button onClick={onClose} className="sheet-close-button">
            ×
          </button>
        </div>
        <div className="sheet-body">
          <div className="form-grid">
            <div className="form-group">
              <label>Job Title</label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
                placeholder="Senior Software Engineer"
              />
            </div>
            <div className="form-group">
              <label>Company</label>
              <input
                type="text"
                value={formData.company}
                onChange={(e) =>
                  setFormData({ ...formData, company: e.target.value })
                }
                placeholder="Acme Corp"
              />
            </div>
            <div className="form-group">
              <label>Location</label>
              <input
                type="text"
                value={formData.location}
                onChange={(e) =>
                  setFormData({ ...formData, location: e.target.value })
                }
                placeholder="San Francisco, CA"
              />
            </div>
            <div className="form-group">
              <label>Source</label>
              <select
                value={formData.job_source}
                onChange={(e) =>
                  setFormData({ ...formData, job_source: e.target.value })
                }
              >
                <option value="">Select...</option>
                <option value="LinkedIn">LinkedIn</option>
                <option value="Company Site">Company Site</option>
                <option value="Referral">Referral</option>
                <option value="Other">Other</option>
              </select>
            </div>
            <div className="form-group full-width">
              <label>Posting URL</label>
              <input
                type="url"
                value={formData.posting_url}
                onChange={(e) =>
                  setFormData({ ...formData, posting_url: e.target.value })
                }
                placeholder="https://..."
              />
            </div>
            <div className="form-group full-width">
              <label>Job Description</label>
              <textarea
                value={formData.raw_description}
                onChange={(e) =>
                  setFormData({ ...formData, raw_description: e.target.value })
                }
                placeholder="Paste the full job description here..."
                rows={10}
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
            {isSaving ? "Saving..." : "Save & Add"}
          </button>
        </div>
      </div>
    </>
  );
}

function GenerateResumeSheet({
  jobId,
  jobTitle,
  company,
  onClose,
}: {
  jobId: number;
  jobTitle: string;
  company: string;
  onClose: () => void;
}) {
  const [artifactType, setArtifactType] = useState<"resume" | "cover_letter" | "both">("resume");
  const [options, setOptions] = useState({
    tone: "neutral",
    length: "standard",
    focus: "IC",
    audience: "hiring_manager",
  });
  const [isGenerating, setIsGenerating] = useState(false);
  const [generationProgress, setGenerationProgress] = useState<string>("");
  const [generatedResume, setGeneratedResume] = useState<any>(null);
  const [generatedLetter, setGeneratedLetter] = useState<any>(null);
  const [resumeContent, setResumeContent] = useState<string>("");
  const [letterContent, setLetterContent] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [aiSettings, setAiSettings] = useState<any>(null);
  const navigate = useNavigate();

  useEffect(() => {
    // Load AI settings to show provider status
    invoke("get_ai_settings")
      .then((settings: any) => setAiSettings(settings))
      .catch(() => {}); // Ignore errors, just don't show provider status
  }, []);

  async function handleGenerate() {
    setIsGenerating(true);
    setError(null);
    setGenerationProgress("");
    
    try {
      if (artifactType === "resume" || artifactType === "both") {
        setGenerationProgress("Generating resume...");
        const result = await invoke<any>("generate_resume_for_job", {
          jobId: jobId,
          applicationId: null,
          options: {
            tone: options.tone,
            length: options.length,
            focus: options.focus,
          },
        });
        setGeneratedResume(result.resume);
        setResumeContent(result.content);
        if (artifactType === "both") {
          setGenerationProgress("Resume generated. Generating cover letter...");
        }
      }

      if (artifactType === "cover_letter" || artifactType === "both") {
        if (artifactType === "cover_letter") {
          setGenerationProgress("Generating cover letter...");
        }
        const result = await invoke<any>("generate_cover_letter_for_job", {
          jobId: jobId,
          applicationId: null,
          options: {
            tone: options.tone,
            length: options.length,
            audience: options.audience,
          },
        });
        setGeneratedLetter(result.letter);
        setLetterContent(result.content);
        setGenerationProgress("");
      }
    } catch (err: any) {
      const errorMessage = err?.message || "Failed to generate";
      // Provide helpful error messages
      if (errorMessage.includes("not yet implemented") || errorMessage.includes("Local model") || errorMessage.includes("model path not configured") || errorMessage.includes("Failed to resolve provider") || errorMessage.includes("requires") || errorMessage.includes("not set up") || errorMessage.includes("not configured")) {
        setError(
          "AI provider is not configured. Please configure your AI provider in Settings."
        );
      } else if (errorMessage.includes("API key") || errorMessage.includes("Invalid") || errorMessage.includes("not set up")) {
        setError(
          "AI provider is not set up. Please configure your AI provider in Settings."
        );
      } else if (errorMessage.includes("Network") || errorMessage.includes("connection")) {
        setError(
          "Network error. Please check your internet connection and try again."
        );
      } else {
        setError(errorMessage);
      }
      setGenerationProgress("");
    } finally {
      setIsGenerating(false);
    }
  }

  return (
    <>
      <div className="sheet-overlay" onClick={onClose}></div>
      <div className="sheet-container" style={{ maxWidth: '800px' }} onClick={(e) => e.stopPropagation()}>
        <div className="sheet-header">
          <h2>Generate Resume & Cover Letter</h2>
          <button onClick={onClose} className="sheet-close-button">
            ×
          </button>
        </div>
        <div className="sheet-body">
          {error && (
            <div className="error-banner" style={{ marginBottom: "1rem" }}>
              {error}
              {error.includes("Settings") && (
                <div style={{ marginTop: "0.5rem" }}>
                  <a
                    href="#"
                    onClick={(e) => {
                      e.preventDefault();
                      navigate("/settings");
                    }}
                    style={{ color: "inherit", textDecoration: "underline", cursor: "pointer" }}
                  >
                    Go to Settings →
                  </a>
                </div>
              )}
            </div>
          )}
          
          {aiSettings && (
            <div className="provider-status" style={{ 
              marginBottom: "1rem", 
              padding: "0.75rem", 
              backgroundColor: "#f0f9ff", 
              borderRadius: "0.375rem",
              fontSize: "0.875rem"
            }}>
              <strong>AI Provider:</strong>{" "}
              {aiSettings.mode === "local" && "Local (Not yet available)"}
              {aiSettings.mode === "cloud" && `Cloud (${aiSettings.cloudProvider || "OpenAI"})`}
              {aiSettings.mode === "hybrid" && "Hybrid"}
            </div>
          )}

          {!generatedResume && !generatedLetter ? (
            <>
              <div className="generation-info">
                <p>
                  <strong>Job:</strong> {jobTitle} @ {company}
                </p>
              </div>
              
              {isGenerating && generationProgress && (
                <div className="generation-progress" style={{
                  marginBottom: "1rem",
                  padding: "1rem",
                  backgroundColor: "#f9fafb",
                  borderRadius: "0.375rem",
                  textAlign: "center"
                }}>
                  <div style={{ marginBottom: "0.5rem" }}>{generationProgress}</div>
                  <div className="loading-spinner" style={{
                    display: "inline-block",
                    width: "20px",
                    height: "20px",
                    border: "3px solid #e5e7eb",
                    borderTopColor: "#6366f1",
                    borderRadius: "50%",
                    animation: "spin 0.8s linear infinite"
                  }}></div>
                </div>
              )}

              <div className="form-grid">
                <div className="form-group full-width">
                  <label>Generate</label>
                  <select
                    value={artifactType}
                    onChange={(e) =>
                      setArtifactType(
                        e.target.value as "resume" | "cover_letter" | "both"
                      )
                    }
                  >
                    <option value="resume">Resume Only</option>
                    <option value="cover_letter">Cover Letter Only</option>
                    <option value="both">Both Resume & Cover Letter</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>Tone</label>
                  <select
                    value={options.tone}
                    onChange={(e) =>
                      setOptions({ ...options, tone: e.target.value })
                    }
                  >
                    <option value="neutral">Neutral</option>
                    <option value="confident">Confident</option>
                    <option value="friendly">Friendly</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>Length</label>
                  <select
                    value={options.length}
                    onChange={(e) =>
                      setOptions({ ...options, length: e.target.value })
                    }
                  >
                    <option value="concise">Concise</option>
                    <option value="standard">Standard</option>
                    <option value="detailed">Detailed</option>
                  </select>
                </div>

                <div className="form-group">
                  <label>Focus</label>
                  <select
                    value={options.focus}
                    onChange={(e) =>
                      setOptions({ ...options, focus: e.target.value })
                    }
                  >
                    <option value="IC">Individual Contributor</option>
                    <option value="Leadership">Leadership</option>
                    <option value="Hybrid">Hybrid</option>
                  </select>
                </div>

                {(artifactType === "cover_letter" || artifactType === "both") && (
                  <div className="form-group">
                    <label>Audience</label>
                    <select
                      value={options.audience}
                      onChange={(e) =>
                        setOptions({ ...options, audience: e.target.value })
                      }
                    >
                      <option value="hiring_manager">Hiring Manager</option>
                      <option value="recruiter">Recruiter</option>
                    </select>
                  </div>
                )}
              </div>

            </>
          ) : (
            <div className="generated-content">
              {generatedResume && (
                <div className="artifact-preview">
                  <h3>Generated Resume</h3>
                  {generatedResume.highlights && generatedResume.highlights.length > 0 && (
                    <div className="highlights">
                      <strong>Highlights:</strong>
                      <ul>
                        {generatedResume.highlights.map((h: string, i: number) => (
                          <li key={i}>{h}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                  <div className="content-preview">
                    <pre>{resumeContent}</pre>
                  </div>
                </div>
              )}

              {generatedLetter && (
                <div className="artifact-preview">
                  <h3>Generated Cover Letter</h3>
                  <div className="content-preview">
                    <pre>{letterContent}</pre>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
        <div className="sheet-footer">
          {!generatedResume && !generatedLetter ? (
            <>
              <button onClick={onClose} className="cancel-button">
                Cancel
              </button>
              <button
                onClick={handleGenerate}
                disabled={isGenerating}
                className="save-button"
              >
                {isGenerating ? (generationProgress || "Generating...") : "Generate"}
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
