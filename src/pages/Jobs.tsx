import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { JobCard } from "../components/JobCard";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import { ProgressIndicator, ProgressStep } from "../components/ProgressIndicator";
import { formatErrorForUser, formatErrorWithSuggestions } from "../utils/errorUtils";
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

interface PaginatedJobList {
  jobs: JobSummary[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export default function Jobs() {
  const [jobs, setJobs] = useState<JobSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [activeOnly, setActiveOnly] = useState(true);
  const [selectedJob, setSelectedJob] = useState<Job | null>(null);
  const [showAddModal, setShowAddModal] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize] = useState(50);
  const [totalPages, setTotalPages] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    loadJobs();
  }, [searchTerm, activeOnly, currentPage]);

  const loadJobs = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<PaginatedJobList>("get_job_list", {
        search: searchTerm || null,
        activeOnly: activeOnly,
        source: null,
        page: currentPage,
        pageSize: pageSize,
      });
      setJobs(result.jobs);
      setTotal(result.total);
      setTotalPages(result.total_pages);
    } catch (err: any) {
      setError(err?.message || "Failed to load jobs");
    } finally {
      setIsLoading(false);
    }
  }, [searchTerm, activeOnly, currentPage, pageSize]);

  async function loadJobDetail(id: number) {
    try {
      const job = await invoke<Job>("get_job_detail", { id });
      setSelectedJob(job);
    } catch (err: any) {
      setError(err?.message || "Failed to load job details");
    }
  }

  const handleJobClick = useCallback((job: JobSummary) => {
    loadJobDetail(job.id);
  }, []);

  function handlePageChange(newPage: number) {
    setCurrentPage(newPage);
    // Scroll to top of list
    const sidebar = document.querySelector('.jobs-sidebar');
    if (sidebar) {
      sidebar.scrollTop = 0;
    }
  }

  if (isLoading && jobs.length === 0) {
    return (
      <div className="jobs">
        <div className="jobs-header">
          <LoadingSkeleton width="150px" height="2rem" />
        </div>
        <div className="jobs-layout">
          <div className="jobs-sidebar">
            <LoadingSkeleton variant="list" lines={5} />
          </div>
          <div className="job-detail">
            <LoadingSkeleton variant="card" />
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="jobs">
      <div className="jobs-header">
        <h1>Jobs</h1>
        <button 
          onClick={() => setShowAddModal(true)} 
          className="add-button"
          aria-label="Add new job"
        >
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
              <>
                {jobs.map((job) => (
                  <JobCard
                    key={job.id}
                    job={job}
                    isSelected={selectedJob?.id === job.id}
                    onSelect={handleJobClick}
                    onRefresh={loadJobs}
                  />
                ))}
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
              </>
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
      showToast(err?.message || "Failed to save job", "error");
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
      showToast(err?.message || "Failed to parse job", "error");
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
                  <button 
                    onClick={() => setIsEditing(true)} 
                    className="edit-button"
                    aria-label="Edit job details"
                  >
                    Edit
                  </button>
                  {formData.raw_description && (
                    <button
                      onClick={parseWithAI}
                      disabled={isParsing}
                      className="parse-button"
                      aria-label={isParsing ? "Parsing job description" : "Parse job description with AI"}
                      style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}
                    >
                      {isParsing && (
                        <ProgressIndicator variant="compact" message="Parsing..." />
                      )}
                      {isParsing ? "Parsing..." : "Parse with AI"}
                    </button>
                  )}
                  <button
                    onClick={() => setShowGenerateModal(true)}
                    className="generate-button"
                    aria-label="Generate resume and cover letter for this job"
                  >
                    Generate Resume/Cover Letter
                  </button>
                </>
              )}
          {isEditing && (
            <>
              <button 
                onClick={() => setIsEditing(false)} 
                className="cancel-button"
                aria-label="Cancel editing job"
              >
                Cancel
              </button>
              <button
                onClick={saveJob}
                disabled={isSaving}
                className="save-button"
                aria-label={isSaving ? "Saving job" : "Save job changes"}
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
                aria-label="Edit this section"
              >
                <span aria-hidden="true">✏️</span>
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
                aria-label="Edit this section"
              >
                <span aria-hidden="true">✏️</span>
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
  const [isScraping, setIsScraping] = useState(false);

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

  async function handleScrapeUrl() {
    if (!formData.posting_url) {
      showToast("Please enter a URL first", "warning");
      return;
    }

    setIsScraping(true);
    try {
      const scraped = await invoke<{
        title?: string;
        company?: string;
        location?: string;
        description: string;
        source: string;
      }>("scrape_job_url", { url: formData.posting_url });

      // Populate form with scraped data
      setFormData((prev) => ({
        ...prev,
        title: prev.title || scraped.title || "",
        company: prev.company || scraped.company || "",
        location: prev.location || scraped.location || "",
        job_source: prev.job_source || scraped.source || "",
        raw_description: prev.raw_description || scraped.description || "",
      }));

      showToast(`Successfully scraped job from ${scraped.source}`, "success");
    } catch (err: any) {
      showToast(err?.message || "Failed to scrape job URL", "error");
    } finally {
      setIsScraping(false);
    }
  }

  async function handleSubmit() {
    if (!formData.title && !formData.company && !formData.raw_description) {
      showToast("Please provide at least a title, company, or description", "warning");
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
      showToast("Job created successfully", "success");
      onSuccess();
    } catch (err: any) {
      showToast(err?.message || "Failed to create job", "error");
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
          <button 
            onClick={onClose} 
            className="sheet-close-button"
            aria-label="Close dialog"
          >
            <span aria-hidden="true">×</span>
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
                onKeyDown={handleKeyDown}
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
                onKeyDown={handleKeyDown}
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
              <div style={{ display: "flex", gap: "0.5rem" }}>
                <input
                  type="url"
                  value={formData.posting_url}
                  onChange={(e) =>
                    setFormData({ ...formData, posting_url: e.target.value })
                  }
                  placeholder="https://..."
                  style={{ flex: 1 }}
                  onKeyDown={handleKeyDown}
                />
                <button
                  type="button"
                  onClick={handleScrapeUrl}
                  disabled={!formData.posting_url || isScraping}
                  className="scrape-button"
                  aria-label={isScraping ? "Scraping job from URL" : "Scrape job details from URL"}
                  title="Extract job details from URL"
                >
                  {isScraping ? "Scraping..." : "🔍 Scrape"}
                </button>
              </div>
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
                onKeyDown={(e) => {
                  // Allow Enter in textarea, but Cmd/Ctrl+Enter submits
                  if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                    e.preventDefault();
                    handleSubmit();
                  }
                }}
              />
            </div>
          </div>
        </div>
        <div className="sheet-footer">
          <button 
            onClick={onClose} 
            className="cancel-button"
            aria-label="Cancel adding job"
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={isSaving}
            className="save-button"
            aria-label={isSaving ? "Saving job" : "Save and add job"}
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
  const [generationSteps, setGenerationSteps] = useState<ProgressStep[]>([]);
  const [generatedResume, setGeneratedResume] = useState<any>(null);
  const [generatedLetter, setGeneratedLetter] = useState<any>(null);
  const [resumeContent, setResumeContent] = useState<string>("");
  const [letterContent, setLetterContent] = useState<string>("");
  const [error, setError] = useState<string | null>(null);
  const [aiSettings, setAiSettings] = useState<any>(null);
  const [isLocalAvailable, setIsLocalAvailable] = useState<boolean>(false);
  const [resumeName, setResumeName] = useState<string>("");
  const [letterName, setLetterName] = useState<string>("");
  const [isSaving, setIsSaving] = useState<boolean>(false);
  const [savedArtifacts, setSavedArtifacts] = useState<any[]>([]);
  const [resumeViewMode, setResumeViewMode] = useState<"structured" | "text">("structured");
  const [letterViewMode, setLetterViewMode] = useState<"structured" | "text">("structured");
  const navigate = useNavigate();

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

  useEffect(() => {
    // Load AI settings to show provider status
    invoke("get_ai_settings")
      .then((settings: any) => {
        setAiSettings(settings);
        // Check if local provider is available
        if (settings?.mode === "local") {
          invoke<boolean>("check_local_provider_availability")
            .then((available) => setIsLocalAvailable(available))
            .catch(() => setIsLocalAvailable(false));
        }
      })
      .catch(() => {}); // Ignore errors, just don't show provider status
    
    // Load saved artifacts for this job
    loadSavedArtifacts();
  }, [jobId]);

  async function loadSavedArtifacts() {
    try {
      const artifacts = await invoke<any[]>("get_artifacts_for_job", { jobId });
      setSavedArtifacts(artifacts || []);
    } catch (err) {
      console.error("Failed to load artifacts:", err);
    }
  }

  async function handleGenerate() {
    setIsGenerating(true);
    setError(null);
    setGenerationProgress("");
    
    // Set up progress steps based on artifact type
    if (artifactType === "both") {
      setGenerationSteps([
        { id: "resume", label: "Generating Resume", status: "active" },
        { id: "letter", label: "Generating Cover Letter", status: "pending" },
      ]);
    } else if (artifactType === "resume") {
      setGenerationSteps([
        { id: "resume", label: "Generating Resume", status: "active" },
      ]);
    } else {
      setGenerationSteps([
        { id: "letter", label: "Generating Cover Letter", status: "active" },
      ]);
    }
    
    try {
      if (artifactType === "resume" || artifactType === "both") {
        setGenerationProgress("Analyzing job requirements and generating resume...");
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
        // Set default resume name
        const defaultResumeName = `${jobTitle} @ ${company} - ${new Date().toLocaleDateString()}`;
        setResumeName(defaultResumeName);
        
        // Update progress steps
        if (artifactType === "both") {
          setGenerationSteps([
            { id: "resume", label: "Generating Resume", status: "completed" },
            { id: "letter", label: "Generating Cover Letter", status: "active" },
          ]);
          setGenerationProgress("Resume generated. Analyzing job requirements and generating cover letter...");
        } else {
          setGenerationSteps([
            { id: "resume", label: "Generating Resume", status: "completed" },
          ]);
          setGenerationProgress("");
        }
      }

      if (artifactType === "cover_letter" || artifactType === "both") {
        if (artifactType === "cover_letter") {
          setGenerationProgress("Analyzing job requirements and generating cover letter...");
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
        // Set default letter name
        const defaultLetterName = `Cover Letter - ${jobTitle} @ ${company} - ${new Date().toLocaleDateString()}`;
        setLetterName(defaultLetterName);
        
        // Update progress steps
        setGenerationSteps((prev) =>
          prev.map((step) =>
            step.id === "letter" ? { ...step, status: "completed" as const } : step
          )
        );
        setGenerationProgress("");
      }
    } catch (err: any) {
      const errorMessage = err?.message || "Failed to generate";
      const errorInfo = formatErrorForUser(errorMessage);
      
      // Set error with suggestions
      setError(formatErrorWithSuggestions(errorInfo));
      
      // Show toast with short message
      showToast(errorInfo.message, "error");
      
      setGenerationProgress("");
      // Mark current step as error
      setGenerationSteps((prev) =>
        prev.map((step) =>
          step.status === "active" ? { ...step, status: "error" as const } : step
        )
      );
    } finally {
      setIsGenerating(false);
    }
  }

  async function handleSaveResume() {
    if (!generatedResume || !resumeName.trim()) {
      setError("Please enter a name for the resume");
      return;
    }

    setIsSaving(true);
    setError(null);

    try {
      await invoke("save_resume", {
        jobId: jobId,
        applicationId: null,
        resume: generatedResume,
        title: resumeName.trim(),
      });
      
      // Reload artifacts and reset state
      await loadSavedArtifacts();
      setGeneratedResume(null);
      setResumeContent("");
      setResumeName("");
    } catch (err: any) {
      setError(err?.message || "Failed to save resume");
    } finally {
      setIsSaving(false);
    }
  }

  async function handleSaveLetter() {
    if (!generatedLetter || !letterName.trim()) {
      setError("Please enter a name for the cover letter");
      return;
    }

    setIsSaving(true);
    setError(null);

    try {
      await invoke("save_cover_letter", {
        jobId: jobId,
        applicationId: null,
        letter: generatedLetter,
        title: letterName.trim(),
      });
      
      // Reload artifacts and reset state
      await loadSavedArtifacts();
      setGeneratedLetter(null);
      setLetterContent("");
      setLetterName("");
    } catch (err: any) {
      setError(err?.message || "Failed to save cover letter");
    } finally {
      setIsSaving(false);
    }
  }

  function handleDiscard() {
    setGeneratedResume(null);
    setGeneratedLetter(null);
    setResumeContent("");
    setLetterContent("");
    setResumeName("");
    setLetterName("");
    setError(null);
  }

  return (
    <>
      <div className="sheet-overlay" onClick={onClose}></div>
      <div className="sheet-container" style={{ maxWidth: '800px' }} onClick={(e) => e.stopPropagation()}>
        <div className="sheet-header">
          <h2>Generate Resume & Cover Letter</h2>
          <button 
            onClick={onClose} 
            className="sheet-close-button"
            aria-label="Close dialog"
          >
            <span aria-hidden="true">×</span>
          </button>
        </div>
        <div className="sheet-body">
          {error && (() => {
            const errorInfo = formatErrorForUser(error);
            return (
              <div className="error-banner" style={{ marginBottom: "1rem" }}>
                <div style={{ marginBottom: errorInfo.suggestions.length > 0 ? "0.75rem" : "0" }}>
                  <strong>{errorInfo.message}</strong>
                </div>
                {errorInfo.suggestions.length > 0 && (
                  <div style={{ marginTop: "0.5rem", fontSize: "0.875rem" }}>
                    <div style={{ marginBottom: "0.25rem", fontWeight: "500" }}>Suggestions:</div>
                    <ul style={{ margin: 0, paddingLeft: "1.25rem" }}>
                      {errorInfo.suggestions.map((suggestion, idx) => (
                        <li key={idx} style={{ marginBottom: "0.25rem" }}>{suggestion}</li>
                      ))}
                    </ul>
                  </div>
                )}
                {errorInfo.requiresAction && (
                  <div style={{ marginTop: "0.75rem" }}>
                    <button
                      onClick={(e) => {
                        e.preventDefault();
                        navigate("/settings");
                      }}
                      className="btn-primary"
                      style={{ fontSize: "0.875rem", padding: "0.375rem 0.75rem" }}
                    >
                      Go to Settings →
                    </button>
                  </div>
                )}
                <button 
                  onClick={() => setError(null)} 
                  style={{ 
                    position: "absolute", 
                    top: "0.5rem", 
                    right: "0.5rem",
                    background: "none",
                    border: "none",
                    fontSize: "1.5rem",
                    cursor: "pointer",
                    color: "inherit",
                    padding: "0",
                    width: "24px",
                    height: "24px",
                    lineHeight: "1"
                  }}
                  aria-label="Dismiss error"
                >
                  ×
                </button>
              </div>
            );
          })()}
          
          {aiSettings && (
            <div className="provider-status" style={{ 
              marginBottom: "1rem", 
              padding: "0.75rem", 
              backgroundColor: "#f0f9ff", 
              borderRadius: "0.375rem",
              fontSize: "0.875rem"
            }}>
              <strong>AI Provider:</strong>{" "}
              {aiSettings.mode === "local" && (
                isLocalAvailable 
                  ? "Local (Ready)" 
                  : aiSettings.localModelPath 
                    ? "Local (Model file not found)" 
                    : "Local (Model path not configured)"
              )}
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
              
              {isGenerating && (
                <div style={{ marginBottom: "1rem" }}>
                  {generationSteps.length > 1 ? (
                    <ProgressIndicator
                      variant="steps"
                      steps={generationSteps}
                      message={generationProgress}
                    />
                  ) : (
                    <ProgressIndicator
                      variant="spinner"
                      message={generationProgress}
                    />
                  )}
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
                <div className="artifact-preview" style={{ marginBottom: "1.5rem" }}>
                  <h3>Generated Resume</h3>
                  {generatedResume.highlights && generatedResume.highlights.length > 0 && (
                    <div className="highlights" style={{ marginBottom: "1rem" }}>
                      <strong>Highlights:</strong>
                      <ul>
                        {generatedResume.highlights.map((h: string, i: number) => (
                          <li key={i}>{h}</li>
                        ))}
                      </ul>
                    </div>
                  )}
                  <div className="form-group" style={{ marginBottom: "1rem" }}>
                    <label>Resume Name</label>
                    <input
                      type="text"
                      value={resumeName}
                      onChange={(e) => setResumeName(e.target.value)}
                      placeholder="Enter a name for this resume"
                      style={{ width: "100%", padding: "0.5rem" }}
                    />
                  </div>
                  
                  {/* Preview view toggle */}
                  <div style={{ marginBottom: "0.5rem", display: "flex", gap: "0.5rem" }}>
                    <button
                      onClick={() => setResumeViewMode("structured")}
                      style={{
                        padding: "0.375rem 0.75rem",
                        backgroundColor: resumeViewMode === "structured" ? "#6366f1" : "#e5e7eb",
                        color: resumeViewMode === "structured" ? "white" : "#374151",
                        border: "none",
                        borderRadius: "0.375rem",
                        cursor: "pointer",
                        fontSize: "0.875rem"
                      }}
                    >
                      Structured View
                    </button>
                    <button
                      onClick={() => setResumeViewMode("text")}
                      style={{
                        padding: "0.375rem 0.75rem",
                        backgroundColor: resumeViewMode === "text" ? "#6366f1" : "#e5e7eb",
                        color: resumeViewMode === "text" ? "white" : "#374151",
                        border: "none",
                        borderRadius: "0.375rem",
                        cursor: "pointer",
                        fontSize: "0.875rem"
                      }}
                    >
                      Raw Text View
                    </button>
                  </div>
                  
                  <div className="content-preview" style={{ 
                    maxHeight: "400px", 
                    overflow: "auto",
                    backgroundColor: "#f9fafb",
                    padding: "1rem",
                    borderRadius: "0.375rem",
                    marginBottom: "1rem"
                  }}>
                    {resumeViewMode === "structured" ? (
                      <div className="structured-resume-view">
                        {generatedResume.headline && (
                          <h2 style={{ marginBottom: "0.5rem", fontSize: "1.25rem", fontWeight: "bold" }}>
                            {generatedResume.headline}
                          </h2>
                        )}
                        {generatedResume.summary && (
                          <p style={{ marginBottom: "1rem", color: "#4b5563" }}>
                            {generatedResume.summary}
                          </p>
                        )}
                        {generatedResume.sections.map((section: any, sectionIdx: number) => (
                          <div key={sectionIdx} style={{ marginBottom: "1.5rem" }}>
                            <h3 style={{ 
                              marginBottom: "0.75rem", 
                              fontSize: "1.125rem", 
                              fontWeight: "600",
                              borderBottom: "1px solid #e5e7eb",
                              paddingBottom: "0.25rem"
                            }}>
                              {section.title}
                            </h3>
                            {section.items.map((item: any, itemIdx: number) => (
                              <div key={itemIdx} style={{ marginBottom: "1rem" }}>
                                <h4 style={{ marginBottom: "0.25rem", fontSize: "1rem", fontWeight: "500" }}>
                                  {item.heading}
                                </h4>
                                {item.subheading && (
                                  <p style={{ fontSize: "0.875rem", color: "#6b7280", marginBottom: "0.5rem" }}>
                                    {item.subheading}
                                  </p>
                                )}
                                {item.bullets && item.bullets.length > 0 && (
                                  <ul style={{ marginLeft: "1.5rem", marginTop: "0.25rem" }}>
                                    {item.bullets.map((bullet: string, bulletIdx: number) => (
                                      <li key={bulletIdx} style={{ marginBottom: "0.25rem", color: "#374151" }}>
                                        {bullet}
                                      </li>
                                    ))}
                                  </ul>
                                )}
                              </div>
                            ))}
                          </div>
                        ))}
                      </div>
                    ) : (
                      <pre style={{ margin: 0, whiteSpace: "pre-wrap" }}>{resumeContent}</pre>
                    )}
                  </div>
                </div>
              )}

              {generatedLetter && (
                <div className="artifact-preview" style={{ marginBottom: "1.5rem" }}>
                  <h3>Generated Cover Letter</h3>
                  <div className="form-group" style={{ marginBottom: "1rem" }}>
                    <label>Cover Letter Name</label>
                    <input
                      type="text"
                      value={letterName}
                      onChange={(e) => setLetterName(e.target.value)}
                      placeholder="Enter a name for this cover letter"
                      style={{ width: "100%", padding: "0.5rem" }}
                    />
                  </div>
                  
                  {/* Preview view toggle */}
                  <div style={{ marginBottom: "0.5rem", display: "flex", gap: "0.5rem" }}>
                    <button
                      onClick={() => setLetterViewMode("structured")}
                      style={{
                        padding: "0.375rem 0.75rem",
                        backgroundColor: letterViewMode === "structured" ? "#6366f1" : "#e5e7eb",
                        color: letterViewMode === "structured" ? "white" : "#374151",
                        border: "none",
                        borderRadius: "0.375rem",
                        cursor: "pointer",
                        fontSize: "0.875rem"
                      }}
                    >
                      Structured View
                    </button>
                    <button
                      onClick={() => setLetterViewMode("text")}
                      style={{
                        padding: "0.375rem 0.75rem",
                        backgroundColor: letterViewMode === "text" ? "#6366f1" : "#e5e7eb",
                        color: letterViewMode === "text" ? "white" : "#374151",
                        border: "none",
                        borderRadius: "0.375rem",
                        cursor: "pointer",
                        fontSize: "0.875rem"
                      }}
                    >
                      Raw Text View
                    </button>
                  </div>
                  
                  <div className="content-preview" style={{ 
                    maxHeight: "400px", 
                    overflow: "auto",
                    backgroundColor: "#f9fafb",
                    padding: "1rem",
                    borderRadius: "0.375rem",
                    marginBottom: "1rem"
                  }}>
                    {letterViewMode === "structured" ? (
                      <div className="structured-letter-view">
                        {generatedLetter.subject && (
                          <div style={{ marginBottom: "1rem" }}>
                            <strong style={{ color: "#6b7280", fontSize: "0.875rem" }}>Subject:</strong>
                            <p style={{ marginTop: "0.25rem", fontWeight: "500" }}>{generatedLetter.subject}</p>
                          </div>
                        )}
                        {generatedLetter.greeting && (
                          <p style={{ marginBottom: "1rem" }}>{generatedLetter.greeting}</p>
                        )}
                        {generatedLetter.body_paragraphs.map((paragraph: string, idx: number) => (
                          <p key={idx} style={{ marginBottom: "1rem", lineHeight: "1.6", color: "#374151" }}>
                            {paragraph}
                          </p>
                        ))}
                        {generatedLetter.closing && (
                          <p style={{ marginBottom: "0.5rem", marginTop: "1rem" }}>{generatedLetter.closing}</p>
                        )}
                        {generatedLetter.signature && (
                          <p style={{ marginTop: "1rem" }}>{generatedLetter.signature}</p>
                        )}
                      </div>
                    ) : (
                      <pre style={{ margin: 0, whiteSpace: "pre-wrap" }}>{letterContent}</pre>
                    )}
                  </div>
                </div>
              )}

              {savedArtifacts.length > 0 && (
                <div style={{ marginTop: "2rem", paddingTop: "1.5rem", borderTop: "1px solid #e5e7eb" }}>
                  <h4 style={{ marginBottom: "1rem" }}>Saved Resumes & Cover Letters</h4>
                  <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
                    {savedArtifacts.map((artifact) => (
                      <div 
                        key={artifact.id} 
                        style={{ 
                          padding: "0.75rem", 
                          backgroundColor: "#f9fafb", 
                          borderRadius: "0.375rem",
                          display: "flex",
                          justifyContent: "space-between",
                          alignItems: "center"
                        }}
                      >
                        <div>
                          <strong>{artifact.title}</strong>
                          <div style={{ fontSize: "0.875rem", color: "#6b7280", marginTop: "0.25rem" }}>
                            {artifact.type} • {new Date(artifact.created_at).toLocaleDateString()}
                          </div>
                        </div>
                      </div>
                    ))}
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
            <>
              <button 
                onClick={handleDiscard} 
                className="cancel-button"
                disabled={isSaving}
              >
                Discard
              </button>
              <div style={{ display: "flex", gap: "0.5rem" }}>
                {generatedResume && (
                  <button
                    onClick={handleSaveResume}
                    disabled={isSaving || !resumeName.trim()}
                    className="save-button"
                  >
                    {isSaving ? "Saving..." : "Save Resume"}
                  </button>
                )}
                {generatedLetter && (
                  <button
                    onClick={handleSaveLetter}
                    disabled={isSaving || !letterName.trim()}
                    className="save-button"
                  >
                    {isSaving ? "Saving..." : "Save Letter"}
                  </button>
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </>
  );
}
