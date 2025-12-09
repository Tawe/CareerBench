import { memo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { InlineEditable } from "./InlineEditable";
import { showToast } from "./Toast";
import "./JobCard.css";

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
}

interface JobCardProps {
  job: JobSummary;
  isSelected: boolean;
  onSelect: (job: JobSummary) => void;
  onRefresh: () => void;
}

export const JobCard = memo(function JobCard({ job, isSelected, onSelect, onRefresh }: JobCardProps) {
  return (
    <div
      className={`job-card ${isSelected ? "active" : ""}`}
      onClick={() => onSelect(job)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onSelect(job);
        }
      }}
      tabIndex={0}
      role="button"
      aria-label={`Job: ${job.title || "Untitled"} at ${job.company || "Unknown Company"}`}
    >
      <div className="job-card-content" onClick={(e) => e.stopPropagation()}>
        <h3>
          <InlineEditable
            value={job.title || ""}
            onSave={async (newTitle) => {
              if (!job.id) return;
              try {
                await invoke<Job>("update_job", {
                  id: job.id,
                  input: {
                    title: newTitle || null,
                  },
                });
                onRefresh();
              } catch (err: any) {
                showToast(err?.message || "Failed to update job title", "error");
              }
            }}
            placeholder="Untitled"
            className="job-title-inline"
          />
        </h3>
        <p className="job-company">
          <InlineEditable
            value={job.company || ""}
            onSave={async (newCompany) => {
              if (!job.id) return;
              try {
                await invoke<Job>("update_job", {
                  id: job.id,
                  input: {
                    company: newCompany || null,
                  },
                });
                onRefresh();
              } catch (err: any) {
                showToast(err?.message || "Failed to update company", "error");
              }
            }}
            placeholder="Unknown Company"
            className="job-company-inline"
          />
        </p>
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
              onSelect(job);
            }}
            title="Parse with AI"
            aria-label={`Parse job ${job.title || 'Untitled'} with AI`}
          >
            <span aria-hidden="true">🤖</span>
          </button>
          <button
            className="action-button"
            onClick={() => {
              // Create application from job
            }}
            title="Create Application"
            aria-label={`Create application for ${job.title || 'Untitled'}`}
          >
            <span aria-hidden="true">📝</span>
          </button>
        </div>
      </div>
    </div>
  );
});

