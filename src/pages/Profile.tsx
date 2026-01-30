import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { InlineEditable } from "../components/InlineEditable";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import { validate } from "../validation/utils";
import { userProfileDataSchema } from "../validation/schemas";
import "./Profile.css";

interface UserProfile {
  id?: number;
  full_name: string;
  headline?: string;
  location?: string;
  summary?: string;
  current_role_title?: string;
  current_company?: string;
  seniority?: string;
  open_to_roles?: string;
}

interface Experience {
  id?: number;
  company: string;
  title: string;
  location?: string;
  start_date?: string;
  end_date?: string;
  is_current: boolean;
  description?: string;
  achievements?: string;
  tech_stack?: string;
}

interface Skill {
  id?: number;
  name: string;
  category?: string;
  self_rating?: number;
  priority?: string;
  years_experience?: number;
  notes?: string;
}

interface Education {
  id?: number;
  institution: string;
  degree?: string;
  field_of_study?: string;
  start_date?: string;
  end_date?: string;
  grade?: string;
  description?: string;
}

interface Certification {
  id?: number;
  name: string;
  issuing_organization?: string;
  issue_date?: string;
  expiration_date?: string;
  credential_id?: string;
  credential_url?: string;
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

interface UserProfileData {
  profile?: UserProfile;
  experience: Experience[];
  skills: Skill[];
  education: Education[];
  certifications: Certification[];
  portfolio: PortfolioItem[];
}

export default function Profile() {
  const [data, setData] = useState<UserProfileData>({
    profile: undefined,
    experience: [],
    skills: [],
    education: [],
    certifications: [],
    portfolio: [],
  });
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [validationErrors, setValidationErrors] = useState<Record<string, string>>({});
  const [showImportModal, setShowImportModal] = useState(false);
  const [isImporting, setIsImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);

  useEffect(() => {
    loadProfile();
  }, []);

  async function loadProfile() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<UserProfileData>("get_user_profile_data");
      setData(result);
      setIsDirty(false);
    } catch (err: any) {
      setError(err?.message || "Failed to load profile");
    } finally {
      setIsLoading(false);
    }
  }

  function validateProfile(): boolean {
    // Convert snake_case from backend to camelCase for validation
    const dataForValidation = {
      profile: data.profile ? {
        id: data.profile.id,
        fullName: data.profile.full_name,
        headline: data.profile.headline,
        location: data.profile.location,
        summary: data.profile.summary,
        currentRoleTitle: data.profile.current_role_title,
        currentCompany: data.profile.current_company,
        seniority: data.profile.seniority,
        openToRoles: data.profile.open_to_roles,
      } : null,
      experience: data.experience.map(exp => ({
        id: exp.id,
        company: exp.company,
        title: exp.title,
        location: exp.location,
        startDate: exp.start_date,
        endDate: exp.end_date,
        isCurrent: exp.is_current,
        description: exp.description,
        achievements: exp.achievements,
        techStack: exp.tech_stack,
      })),
      skills: data.skills.map(skill => ({
        id: skill.id,
        name: skill.name,
        category: skill.category,
        selfRating: skill.self_rating,
        priority: skill.priority as "Low" | "Medium" | "High" | undefined,
        yearsExperience: skill.years_experience,
        notes: skill.notes,
      })),
      education: data.education.map(edu => ({
        id: edu.id,
        institution: edu.institution,
        degree: edu.degree,
        fieldOfStudy: edu.field_of_study,
        startDate: edu.start_date,
        endDate: edu.end_date,
        grade: edu.grade,
        description: edu.description,
      })),
      certifications: data.certifications.map(cert => ({
        id: cert.id,
        name: cert.name,
        issuingOrganization: cert.issuing_organization,
        issueDate: cert.issue_date,
        expirationDate: cert.expiration_date,
        credentialId: cert.credential_id,
        credentialUrl: cert.credential_url,
      })),
      portfolio: data.portfolio.map(item => ({
        id: item.id,
        title: item.title,
        url: item.url,
        description: item.description,
        role: item.role,
        techStack: item.tech_stack,
        highlighted: item.highlighted,
      })),
    };
    
    // Use Zod validation
    const result = validate(userProfileDataSchema, dataForValidation);
    
    if (!result.success && result.errors) {
      // Convert camelCase errors back to snake_case for display
      const convertedErrors: Record<string, string> = {};
      Object.entries(result.errors).forEach(([key, value]) => {
        // Convert field paths like "profile.fullName" to "profile.full_name"
        const convertedKey = key
          .replace(/fullName/g, 'full_name')
          .replace(/currentRoleTitle/g, 'current_role_title')
          .replace(/currentCompany/g, 'current_company')
          .replace(/openToRoles/g, 'open_to_roles')
          .replace(/createdAt/g, 'created_at')
          .replace(/updatedAt/g, 'updated_at')
          .replace(/startDate/g, 'start_date')
          .replace(/endDate/g, 'end_date')
          .replace(/isCurrent/g, 'is_current')
          .replace(/techStack/g, 'tech_stack')
          .replace(/fieldOfStudy/g, 'field_of_study')
          .replace(/issuingOrganization/g, 'issuing_organization')
          .replace(/issueDate/g, 'issue_date')
          .replace(/expirationDate/g, 'expiration_date')
          .replace(/credentialId/g, 'credential_id')
          .replace(/credentialUrl/g, 'credential_url')
          .replace(/selfRating/g, 'self_rating')
          .replace(/yearsExperience/g, 'years_experience');
        convertedErrors[convertedKey] = value;
      });
      setValidationErrors(convertedErrors);
      return false;
    }
    
    setValidationErrors({});
    return true;
  }

  async function saveProfile() {
    // Validate before saving
    if (!validateProfile()) {
      setError("Please fix the validation errors before saving");
      // Scroll to first error
      const firstErrorField = document.querySelector('[data-error]');
      if (firstErrorField) {
        firstErrorField.scrollIntoView({ behavior: 'smooth', block: 'center' });
      }
      return;
    }
    
    setIsSaving(true);
    setError(null);
    setValidationErrors({});
    try {
      const result = await invoke<UserProfileData>("save_user_profile_data", { data });
      setData(result);
      setIsDirty(false);
    } catch (err: any) {
      const errorMessage = err?.message || "Failed to save profile";
      setError(errorMessage);
      // If it's a validation error from backend, try to parse it
      if (errorMessage.includes("required") || errorMessage.includes("invalid")) {
        setValidationErrors({ _general: errorMessage });
      }
    } finally {
      setIsSaving(false);
    }
  }

  function updateProfile(field: keyof UserProfile, value: string) {
    setData((prev) => ({
      ...prev,
      profile: {
        ...prev.profile,
        full_name: prev.profile?.full_name || "",
        [field]: value || undefined,
      } as UserProfile,
    }));
    setIsDirty(true);
  }

  async function handleImportResume() {
    try {
      console.log("Opening file picker...");
      
      // Open file picker using Tauri dialog plugin
      const selected = await open({
        multiple: false,
        filters: [{
          name: "Resume Files",
          extensions: ["pdf", "txt", "docx", "doc"]
        }],
        title: "Select Resume/CV File"
      });

      console.log("File picker result:", selected);

      if (!selected) {
        // User cancelled
        console.log("User cancelled file selection");
        return;
      }

      // Handle different return types from Tauri dialog
      // In Tauri 2.0, open() returns string | string[] | null for file selection
      let filePath: string;
      
      if (typeof selected === "string") {
        filePath = selected;
      } else if (Array.isArray(selected) && selected.length > 0) {
        filePath = selected[0];
      } else {
        console.error("Unexpected file selection result:", selected);
        showToast("Unexpected file selection format", "error");
        return;
      }
      
      if (!filePath || filePath.trim() === "") {
        showToast("No file selected", "info");
        return;
      }
      
      console.log("Selected file path:", filePath);

      showToast("Extracting text from resume...", "info");
      setIsImporting(true);

      try {
        // Extract text from resume
        console.log("Calling extract_resume_text with path:", filePath);
        const parsed = await invoke<{ text: string; file_path: string }>("extract_resume_text", {
          filePath
        });

        console.log("Text extracted, length:", parsed.text.length);

        if (!parsed.text || parsed.text.trim().length === 0) {
          throw new Error("No text could be extracted from the resume file. The file might be empty or corrupted.");
        }

        showToast("Extracting profile data with AI...", "info");

        // Extract profile data using AI
        try {
          const extracted = await invoke<UserProfileData>("extract_profile_from_resume", {
            resumeText: parsed.text
          });

          console.log("Profile extracted:", extracted);

          // Merge extracted data with existing data
          setData((prev) => ({
            profile: extracted.profile || prev.profile,
            experience: [...prev.experience, ...extracted.experience],
            skills: [...prev.skills, ...extracted.skills],
            education: [...prev.education, ...extracted.education],
            certifications: [...prev.certifications, ...extracted.certifications],
            portfolio: [...prev.portfolio, ...extracted.portfolio],
          }));
          setIsDirty(true);
          showToast("Profile imported successfully! Review and save your changes.", "success");
          setShowImportModal(false);
        } catch (aiErr: any) {
          console.error("Error processing resume with AI:", aiErr);
          const errorMsg = aiErr?.message || "Failed to process resume with AI.";
          
          // Check if the error is about invalid model filename/path and automatically clean it up
          // The error message may be prefixed with "AI error on chunk X: " or "Failed to resolve provider: "
          const hasQueryParams = errorMsg.includes("?download=true") || errorMsg.includes("query parameters");
          const isInvalidModelError = (errorMsg.includes("Invalid model filename") || 
                                      errorMsg.includes("Invalid model path") ||
                                      errorMsg.includes("Model file not found")) && 
                                     hasQueryParams;
          
          if (isInvalidModelError) {
            console.log("Detected invalid model file/path, attempting to clean up...");
            try {
              // Clean up invalid files
              const cleaned = await invoke<string[]>("cleanup_invalid_model_files");
              if (cleaned && cleaned.length > 0) {
                console.log(`Cleaned up ${cleaned.length} invalid model file(s):`, cleaned);
              }
              
              // Clear invalid path from settings
              const pathCleared = await invoke<boolean>("clear_invalid_model_path");
              if (pathCleared) {
                console.log("Cleared invalid model path from settings");
              }
              
              showToast(
                `Cleaned up invalid model file and path. Please go to Settings and download the model again, then try importing your resume.`,
                "info"
              );
              setImportError(
                "Invalid model file and path detected and removed. Please go to Settings and download the model again, then try importing your resume."
              );
            } catch (cleanupErr: any) {
              console.error("Failed to clean up invalid model files/path:", cleanupErr);
              setImportError(
                `${errorMsg}\n\nPlease go to Settings, clear the model path, and re-download the model.`
              );
              showToast("Failed to automatically clean up. Please clear the model path in Settings and re-download.", "error");
            }
          } else {
            setImportError(errorMsg);
            showToast(errorMsg, "error");
          }
          
          // If AI fails but text was extracted, show the text so user can see what was extracted
          if (parsed.text) {
            console.log("Extracted text (AI processing failed):", parsed.text.substring(0, 500));
          }
        }
      } catch (extractErr: any) {
        console.error("Error extracting text from resume:", extractErr);
        const errorMsg = extractErr?.message || "Failed to extract text from resume file.";
        setImportError(errorMsg);
        showToast(errorMsg, "error");
      } finally {
        setIsImporting(false);
      }
    } catch (err: any) {
      console.error("Import error:", err);
      const errorMsg = err?.message || "Failed to import resume. Please check the console for details.";
      showToast(errorMsg, "error");
      setImportError(errorMsg);
      setIsImporting(false);
    }
  }

  if (isLoading) {
    return (
      <div className="profile">
        <div className="profile-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="profile-content content-constrained">
          <LoadingSkeleton variant="card" className="profile-section" />
          <LoadingSkeleton variant="card" className="profile-section" />
          <LoadingSkeleton variant="card" className="profile-section" />
        </div>
      </div>
    );
  }

  return (
    <div className="profile">
      <div className="profile-header">
        <h1>Profile</h1>
        <div className="profile-actions">
          <button
            onClick={handleImportResume}
            className="import-button"
            aria-label="Import profile from resume/CV"
            title="Import profile from PDF or TXT resume file"
          >
            📄 Import Resume
          </button>
          {isDirty && (
            <span className="unsaved-indicator">Unsaved changes</span>
          )}
          <button
            onClick={saveProfile}
            disabled={!isDirty || isSaving}
            className="save-button"
            aria-label={isSaving ? "Saving profile" : "Save profile changes"}
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
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

      <div className="profile-content content-constrained">
        <BasicInfoSection 
          profile={data.profile} 
          onUpdate={updateProfile}
          validationErrors={validationErrors}
        />

        <ExperienceSection
          experience={data.experience}
          onUpdate={(exp) => {
            setData((prev) => ({ ...prev, experience: exp }));
            setIsDirty(true);
          }}
          validationErrors={validationErrors}
        />

        <SkillsSection
          skills={data.skills}
          experience={data.experience}
          education={data.education}
          portfolio={data.portfolio}
          onUpdate={(newSkills) => {
            setData((prev) => {
              // Create a completely new array to ensure React detects the change
              return { ...prev, skills: [...newSkills] };
            });
            setIsDirty(true);
          }}
        />

        <EducationSection
          education={data.education}
          certifications={data.certifications}
          onUpdate={(edu, certs) => {
            setData((prev) => ({
              ...prev,
              education: edu,
              certifications: certs,
            }));
            setIsDirty(true);
          }}
          validationErrors={validationErrors}
        />

        <PortfolioSection
          portfolio={data.portfolio}
          onUpdate={(portfolio) => {
            setData((prev) => ({ ...prev, portfolio }));
            setIsDirty(true);
          }}
          validationErrors={validationErrors}
        />

        <CareerPreferencesSection
          profile={data.profile}
          onUpdate={updateProfile}
        />
      </div>
    </div>
  );
}

// Basic Info Section Component
function BasicInfoSection({
  profile,
  onUpdate,
  validationErrors = {},
}: {
  profile?: UserProfile;
  onUpdate: (field: keyof UserProfile, value: string) => void;
  validationErrors?: Record<string, string>;
}) {
  return (
    <div className="profile-section">
      <h2>Basic Info</h2>
      {!profile && (
        <div className="empty-state">
          <p>Let's set up your profile so CareerBench can tailor resumes and advice to you.</p>
        </div>
      )}
      <div className="form-grid">
        <div className="form-group">
          <label>
            Full Name <span className="required">*</span>
          </label>
          <div style={{ position: "relative" }}>
            <InlineEditable
              value={profile?.full_name || ""}
              onSave={(newValue) => onUpdate("full_name", newValue)}
              placeholder="John Doe"
              className={validationErrors.full_name ? "inline-editable-error" : ""}
            />
            {validationErrors.full_name && (
              <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem", display: "block" }}>
                {validationErrors.full_name}
              </span>
            )}
          </div>
        </div>

        <div className="form-group">
          <label>Location</label>
          <InlineEditable
            value={profile?.location || ""}
            onSave={(newValue) => onUpdate("location", newValue)}
            placeholder="San Francisco, CA"
          />
        </div>

        <div className="form-group">
          <label>Pronouns (optional)</label>
          <input
            type="text"
            value={profile?.headline || ""}
            onChange={(e) => onUpdate("headline", e.target.value)}
            placeholder="She/Her, He/Him, They/Them"
          />
        </div>

        <div className="form-group">
          <label>Preferred Work Style</label>
          <select
            value={profile?.current_role_title || ""}
            onChange={(e) => onUpdate("current_role_title", e.target.value)}
          >
            <option value="">Select...</option>
            <option value="Remote">Remote</option>
            <option value="Hybrid">Hybrid</option>
            <option value="Onsite">Onsite</option>
          </select>
        </div>

        <div className="form-group">
          <label>Preferred Locations</label>
          <input
            type="text"
            value={profile?.current_company || ""}
            onChange={(e) => onUpdate("current_company", e.target.value)}
            placeholder="Cities, regions, or countries"
          />
        </div>

        <div className="form-group full-width">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
            <label>Professional Summary</label>
            <button
              type="button"
              onClick={async () => {
                try {
                  const summary = await invoke<string>("generate_profile_summary");
                  onUpdate("summary", summary);
                } catch (err: any) {
                  showToast(err?.message || "Failed to generate summary", "error");
                }
              }}
              aria-label="Generate professional summary with AI"
              style={{
                padding: "0.375rem 0.75rem",
                backgroundColor: "#6366f1",
                color: "white",
                border: "none",
                borderRadius: "0.375rem",
                cursor: "pointer",
                fontSize: "0.875rem"
              }}
            >
              <span aria-hidden="true">✨</span> Generate with AI
            </button>
          </div>
          <textarea
            value={profile?.summary || ""}
            onChange={(e) => onUpdate("summary", e.target.value)}
            placeholder="2-6 paragraph professional summary..."
            rows={6}
          />
        </div>
      </div>
    </div>
  );
}

// Career Preferences Section
function CareerPreferencesSection({
  profile,
  onUpdate,
}: {
  profile?: UserProfile;
  onUpdate: (field: keyof UserProfile, value: string) => void;
}) {
  return (
    <div className="profile-section">
      <h2>Career Preferences</h2>
      <div className="form-grid">
        <div className="form-group full-width">
          <label>Target Roles</label>
          <input
            type="text"
            value={profile?.open_to_roles || ""}
            onChange={(e) => onUpdate("open_to_roles", e.target.value)}
            placeholder="Engineering Manager, Director of Engineering, VP of Engineering"
          />
        </div>

        <div className="form-group">
          <label>Seniority Level Seeking</label>
          <select
            value={profile?.seniority || ""}
            onChange={(e) => onUpdate("seniority", e.target.value)}
          >
            <option value="">Select...</option>
            <option value="Associate">Associate</option>
            <option value="Mid">Mid</option>
            <option value="Senior">Senior</option>
            <option value="Lead">Lead</option>
            <option value="Manager">Manager</option>
            <option value="Director">Director</option>
            <option value="VP">VP</option>
            <option value="C-Level">C-Level</option>
          </select>
        </div>

        <div className="form-group full-width">
          <label>Industries of Interest</label>
          <input
            type="text"
            value={profile?.location ? "" : "" /* placeholder mapping slot */}
            onChange={() => {
              /* reserved for future backend fields */
            }}
            placeholder="FinTech, AI Tools, SaaS"
          />
        </div>
      </div>
    </div>
  );
}

// Experience Section Component
function ExperienceSection({
  experience,
  onUpdate,
  validationErrors = {},
}: {
  experience: Experience[];
  onUpdate: (exp: Experience[]) => void;
  validationErrors?: Record<string, string>;
}) {
  const [editingIndex, setEditingIndex] = useState<number | "new" | null>(null);
  const [formData, setFormData] = useState<Experience>({
    company: "",
    title: "",
    location: "",
    start_date: "",
    end_date: "",
    is_current: false,
    description: "",
    achievements: "",
    tech_stack: "",
  });

  function startEdit(exp?: Experience, index?: number) {
    if (exp != null && index != null) {
      setFormData(exp);
      setEditingIndex(index);
    } else {
      // Adding new experience
      setFormData({
        company: "",
        title: "",
        location: "",
        start_date: "",
        end_date: "",
        is_current: false,
        description: "",
        achievements: "",
        tech_stack: "",
      });
      setEditingIndex("new");
    }
    setLocalErrors({});
  }

  function cancelEdit() {
    setEditingIndex(null);
    setLocalErrors({});
    setFormData({
      company: "",
      title: "",
      location: "",
      start_date: "",
      end_date: "",
      is_current: false,
      description: "",
      achievements: "",
      tech_stack: "",
    });
  }

  const [localErrors, setLocalErrors] = useState<Record<string, string>>({});

  function saveExperience() {
    const errors: Record<string, string> = {};
    
    if (!formData.company || formData.company.trim() === "") {
      errors.company = "Company is required";
    }
    if (!formData.title || formData.title.trim() === "") {
      errors.title = "Job title is required";
    }
    if (formData.start_date && formData.end_date && !formData.is_current) {
      const start = new Date(formData.start_date);
      const end = new Date(formData.end_date);
      if (start > end) {
        errors.dates = "End date must be after start date";
      }
    }
    
    if (Object.keys(errors).length > 0) {
      setLocalErrors(errors);
      return;
    }
    
    setLocalErrors({});

    const updated = [...experience];
    if (editingIndex === "new") {
      // Adding new experience
      updated.push({ ...formData });
    } else if (editingIndex !== null && typeof editingIndex === "number" && editingIndex >= 0 && editingIndex < updated.length) {
      // Updating existing experience at index
      const existing = updated[editingIndex];
      updated[editingIndex] = { ...formData, id: existing.id };
    }
    onUpdate(updated);
    cancelEdit();
  }

  function deleteExperience(index: number) {
    // Remove confirm dialog - it was blocking deletion in Tauri
    const updated = experience.filter((_, i) => i !== index);
    onUpdate(updated);
  }

  return (
    <div className="profile-section">
      <div className="section-header">
        <h2>Experience</h2>
        <button onClick={() => startEdit()} className="add-button">
          + Add Experience
        </button>
      </div>

      {editingIndex !== null && (
        <div className="edit-form">
          <h3>{editingIndex === "new" ? "Add Experience" : "Edit Experience"}</h3>
          <div className="form-grid">
            <div className="form-group">
              <label>
                Company <span className="required">*</span>
              </label>
              <input
                type="text"
                value={formData.company}
                onChange={(e) => {
                  setFormData({ ...formData, company: e.target.value });
                  if (localErrors.company) setLocalErrors({ ...localErrors, company: "" });
                }}
                required
                style={{
                  borderColor: (localErrors.company || (editingIndex !== null && typeof editingIndex === "number" && validationErrors[`experience_${editingIndex}_company`])) ? "#ef4444" : undefined
                }}
              />
              {(localErrors.company || (editingIndex !== null && typeof editingIndex === "number" && validationErrors[`experience_${editingIndex}_company`])) && (
                <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                  {localErrors.company || validationErrors[`experience_${editingIndex}_company`]}
                </span>
              )}
            </div>

            <div className="form-group">
              <label>
                Job Title <span className="required">*</span>
              </label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) => {
                  setFormData({ ...formData, title: e.target.value });
                  if (localErrors.title) setLocalErrors({ ...localErrors, title: "" });
                }}
                required
                style={{
                  borderColor: (localErrors.title || (editingIndex !== null && typeof editingIndex === "number" && validationErrors[`experience_${editingIndex}_title`])) ? "#ef4444" : undefined
                }}
              />
              {(localErrors.title || (editingIndex !== null && typeof editingIndex === "number" && validationErrors[`experience_${editingIndex}_title`])) && (
                <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                  {localErrors.title || validationErrors[`experience_${editingIndex}_title`]}
                </span>
              )}
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
              <label>Start Date</label>
              <input
                type="date"
                value={formData.start_date ? (formData.start_date.includes('-') && formData.start_date.length === 7 ? `${formData.start_date}-01` : formData.start_date) : ""}
                onChange={(e) => {
                  const value = e.target.value;
                  // Store full date (YYYY-MM-DD) instead of just YYYY-MM
                  setFormData({ ...formData, start_date: value || "" });
                }}
              />
            </div>

            <div className="form-group">
              <label>End Date</label>
              <input
                type="date"
                value={formData.end_date ? (formData.end_date.includes('-') && formData.end_date.length === 7 ? `${formData.end_date}-01` : formData.end_date) : ""}
                onChange={(e) => {
                  const value = e.target.value;
                  // Store full date (YYYY-MM-DD) instead of just YYYY-MM
                  setFormData({ ...formData, end_date: value || "" });
                }}
                disabled={formData.is_current}
              />
              {localErrors.dates && (
                <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                  {localErrors.dates}
                </span>
              )}
            </div>

            <div className="form-group">
              <label>
                <input
                  type="checkbox"
                  checked={formData.is_current}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      is_current: e.target.checked,
                      end_date: e.target.checked ? "" : formData.end_date,
                    })
                  }
                />
                This is my current role
              </label>
            </div>

            <div className="form-group full-width">
              <label>Description</label>
              <textarea
                value={formData.description || ""}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                rows={3}
              />
            </div>

            <div className="form-group full-width">
              <label>Achievements (one per line)</label>
              <textarea
                value={formData.achievements || ""}
                onChange={(e) =>
                  setFormData({ ...formData, achievements: e.target.value })
                }
                placeholder="Enter achievements, one per line"
                rows={4}
              />
            </div>

            <div className="form-group full-width">
              <label>Tech Stack (comma-separated)</label>
              <input
                type="text"
                value={formData.tech_stack || ""}
                onChange={(e) =>
                  setFormData({ ...formData, tech_stack: e.target.value })
                }
                placeholder="React, Node.js, TypeScript, AWS"
              />
            </div>
          </div>

          <div className="form-actions">
            <button onClick={cancelEdit} className="cancel-button">
              Cancel
            </button>
            <button onClick={saveExperience} className="save-button">
              Save
            </button>
          </div>
        </div>
      )}

      <div className="items-list">
        {experience.length === 0 ? (
          <div className="empty-state">
            <p>No experience entries yet. Click "Add Experience" to get started.</p>
          </div>
        ) : (
          experience.map((exp, index) => (
            <div key={exp.id || Math.random()} className="item-card">
              <div className="item-header">
                <div>
                  <h3>{exp.title}</h3>
                  <p className="item-subtitle">{exp.company}</p>
                  <p className="item-meta">
                    {exp.start_date && (
                      <span>
                        {(() => {
                          const dateStr = exp.start_date.length === 7 ? `${exp.start_date}-01` : exp.start_date;
                          const date = new Date(dateStr);
                          const month = String(date.getMonth() + 1).padStart(2, '0');
                          const year = date.getFullYear();
                          return `${month}-${year}`;
                        })()}
                      </span>
                    )}
                    {exp.start_date && (exp.is_current || exp.end_date) && " - "}
                    {exp.is_current ? (
                      <span>Present</span>
                    ) : (
                      exp.end_date && (
                        <span>
                          {(() => {
                            const dateStr = exp.end_date.length === 7 ? `${exp.end_date}-01` : exp.end_date;
                            const date = new Date(dateStr);
                            const month = String(date.getMonth() + 1).padStart(2, '0');
                            const year = date.getFullYear();
                            return `${month}-${year}`;
                          })()}
                        </span>
                      )
                    )}
                    {exp.location && ` • ${exp.location}`}
                  </p>
                </div>
                <div className="item-actions">
                  <button
                    onClick={() => startEdit(exp, index)}
                    className="edit-button"
                    aria-label={`Edit experience at ${exp.company}`}
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => deleteExperience(index)}
                    className="delete-button"
                    type="button"
                    aria-label={`Delete experience at ${exp.company}`}
                  >
                    Delete
                  </button>
                </div>
              </div>
              {exp.description && (
                <p className="item-description">{exp.description}</p>
              )}
              {exp.achievements && (
                <ul className="item-bullets">
                  {exp.achievements.split("\n").filter((b) => b.trim()).map((bullet, i) => (
                    <li key={i}>{bullet}</li>
                  ))}
                </ul>
              )}
              {exp.tech_stack && (
                <div className="item-tags">
                  {exp.tech_stack.split(",").map((tech, i) => (
                    <span key={i} className="tag">
                      {tech.trim()}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// Skills Section Component
function SkillsSection({
  skills,
  experience,
  education,
  portfolio,
  onUpdate,
}: {
  skills: Skill[];
  experience: Experience[];
  education: Education[];
  portfolio: PortfolioItem[];
  onUpdate: (skills: Skill[]) => void;
}) {
  const [editingIndex, setEditingIndex] = useState<number | null>(null);
  const [formData, setFormData] = useState<Skill>({
    name: "",
  });
  const [quickAddName, setQuickAddName] = useState("");
  const [linkedExperienceIds, setLinkedExperienceIds] = useState<number[]>([]);
  const [linkedEducationIds, setLinkedEducationIds] = useState<number[]>([]);
  const [linkedPortfolioIds, setLinkedPortfolioIds] = useState<number[]>([]);

  type SkillLinks = {
    experienceIds: number[];
    educationIds: number[];
    portfolioIds: number[];
  };

  function getLinks(skill: Skill): SkillLinks {
    if (!skill.notes) {
      return { experienceIds: [], educationIds: [], portfolioIds: [] };
    }
    try {
      const parsed = JSON.parse(skill.notes);
      if (parsed && parsed.links) {
        return {
          experienceIds: parsed.links.experienceIds || [],
          educationIds: parsed.links.educationIds || [],
          portfolioIds: parsed.links.portfolioIds || [],
        };
      }
    } catch {
      // ignore, treat as plain text notes
    }
    return { experienceIds: [], educationIds: [], portfolioIds: [] };
  }

  function quickAdd() {
    if (!quickAddName.trim()) return;
    const newSkill: Skill = {
      name: quickAddName.trim(),
    };
    const updated = [...skills, newSkill];
    onUpdate(updated);
    setQuickAddName("");
  }

  function startEdit(skill?: Skill, index?: number) {
    if (skill != null && index != null) {
      setFormData(skill);
      const links = getLinks(skill);
      setLinkedExperienceIds(links.experienceIds);
      setLinkedEducationIds(links.educationIds);
      setLinkedPortfolioIds(links.portfolioIds);
      setEditingIndex(index);
    } else {
      setFormData({
        name: "",
      });
      setLinkedExperienceIds([]);
      setLinkedEducationIds([]);
      setLinkedPortfolioIds([]);
      setEditingIndex(null);
    }
  }

  function cancelEdit() {
    setEditingIndex(null);
    setLinkedExperienceIds([]);
    setLinkedEducationIds([]);
    setLinkedPortfolioIds([]);
  }

  const [localErrors, setLocalErrors] = useState<Record<string, string>>({});

  function saveSkill() {
    if (!formData.name.trim()) {
      setLocalErrors({ name: "Skill name is required" });
      return;
    }
    
    setLocalErrors({});

    const links: SkillLinks = {
      experienceIds: linkedExperienceIds,
      educationIds: linkedEducationIds,
      portfolioIds: linkedPortfolioIds,
    };
    const skillToSave: Skill = {
      ...formData,
      notes: JSON.stringify({ links }),
    };

    const updated = [...skills];
    if (editingIndex == null) {
      // New skill
      updated.push(skillToSave);
    } else if (editingIndex >= 0 && editingIndex < updated.length) {
      // Update existing at index
      const existing = updated[editingIndex];
      updated[editingIndex] = { ...existing, ...skillToSave };
    }
    onUpdate(updated);
    cancelEdit();
  }

  function deleteSkill(index: number) {
    // Create a completely new array to ensure React detects the change
    const updated = skills.filter((_, i) => i !== index);
    
    // Reset editing state if we're deleting the skill being edited
    if (editingIndex === index) {
      setEditingIndex(null);
      setFormData({ name: "" });
      setLinkedExperienceIds([]);
      setLinkedEducationIds([]);
      setLinkedPortfolioIds([]);
    } else if (editingIndex !== null && editingIndex > index) {
      // Adjust editing index if we're deleting before the edited item
      setEditingIndex(editingIndex - 1);
    }
    
    // Update parent state - this will trigger a re-render with new props
    onUpdate([...updated]);
  }

  return (
    <div className="profile-section">
      <div className="section-header">
        <h2>Skills</h2>
        <button
          type="button"
          onClick={async () => {
            try {
                  const extractedSkills = await invoke<string[]>("extract_skills_from_experience");
              // Add extracted skills that don't already exist
              const existingSkillNames = new Set(skills.map(s => s.name.toLowerCase()));
              const newSkills = extractedSkills
                .filter(skill => !existingSkillNames.has(skill.toLowerCase()))
                .map(name => ({ name }));
              if (newSkills.length > 0) {
                onUpdate([...skills, ...newSkills]);
              } else {
                showToast("No new skills found to add.", "info");
              }
            } catch (err: any) {
              showToast(err?.message || "Failed to extract skills", "error");
            }
          }}
          style={{
            padding: "0.375rem 0.75rem",
            backgroundColor: "#6366f1",
            color: "white",
            border: "none",
            borderRadius: "0.375rem",
            cursor: "pointer",
            fontSize: "0.875rem"
          }}
        >
          ✨ Extract from Experience
        </button>
      </div>

      <div className="quick-add">
        <input
          type="text"
          value={quickAddName}
          onChange={(e) => setQuickAddName(e.target.value)}
          onKeyPress={(e) => e.key === "Enter" && quickAdd()}
          placeholder="Quick add skill name..."
          className="quick-add-input"
        />
        <button onClick={quickAdd} className="quick-add-button">
          Add
        </button>
      </div>

      {editingIndex !== null && (
        <div className="edit-form">
          <h3>{editingIndex === null ? "Add Skill" : "Edit Skill"}</h3>
          <div className="form-grid">
            <div className="form-group">
              <label>
                Name <span className="required">*</span>
              </label>
              <input
                type="text"
                value={formData.name}
                onChange={(e) => {
                  setFormData({ ...formData, name: e.target.value });
                  if (localErrors.name) setLocalErrors({ ...localErrors, name: "" });
                }}
                required
                style={{
                  borderColor: localErrors.name ? "#ef4444" : undefined
                }}
              />
              {localErrors.name && (
                <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                  {localErrors.name}
                </span>
              )}
            </div>

            <div className="form-group full-width">
              <label>Linked Experience</label>
              <div className="link-list">
                {experience.length === 0 && (
                  <p className="link-empty">No experience entries yet.</p>
                )}
                {experience.map((exp) => (
                  <label key={exp.id || `${exp.company}-${exp.title}`}>
                    <input
                      type="checkbox"
                      checked={
                        !!exp.id && linkedExperienceIds.includes(exp.id)
                      }
                      onChange={(e) => {
                        if (!exp.id) return;
                        setLinkedExperienceIds((prev) =>
                          e.target.checked
                            ? [...prev, exp.id!]
                            : prev.filter((id) => id !== exp.id)
                        );
                      }}
                    />
                    {exp.title} — {exp.company}
                  </label>
                ))}
              </div>
            </div>

            <div className="form-group full-width">
              <label>Linked Education</label>
              <div className="link-list">
                {education.length === 0 && (
                  <p className="link-empty">No education entries yet.</p>
                )}
                {education.map((edu) => (
                  <label key={edu.id || edu.institution}>
                    <input
                      type="checkbox"
                      checked={
                        !!edu.id && linkedEducationIds.includes(edu.id)
                      }
                      onChange={(e) => {
                        if (!edu.id) return;
                        setLinkedEducationIds((prev) =>
                          e.target.checked
                            ? [...prev, edu.id!]
                            : prev.filter((id) => id !== edu.id)
                        );
                      }}
                    />
                    {edu.degree ? `${edu.degree} — ${edu.institution}` : edu.institution}
                  </label>
                ))}
              </div>
            </div>

            <div className="form-group full-width">
              <label>Linked Projects</label>
              <div className="link-list">
                {portfolio.length === 0 && (
                  <p className="link-empty">No portfolio items yet.</p>
                )}
                {portfolio.map((p) => (
                  <label key={p.id || p.title}>
                    <input
                      type="checkbox"
                      checked={
                        !!p.id && linkedPortfolioIds.includes(p.id)
                      }
                      onChange={(e) => {
                        if (!p.id) return;
                        setLinkedPortfolioIds((prev) =>
                          e.target.checked
                            ? [...prev, p.id!]
                            : prev.filter((id) => id !== p.id)
                        );
                      }}
                    />
                    {p.title}
                  </label>
                ))}
              </div>
            </div>
          </div>

          <div className="form-actions">
            <button onClick={cancelEdit} className="cancel-button">
              Cancel
            </button>
            <button onClick={saveSkill} className="save-button">
              Save
            </button>
          </div>
        </div>
      )}

      <div className="skills-grid" data-skills-count={skills.length}>
        {skills.length === 0 ? (
          <div className="empty-state">
            <p>No skills yet. Use quick add above or click "Add Skill" to get started.</p>
          </div>
        ) : (
          skills.map((skill, index) => {
            const links = getLinks(skill);
            const expCount = links.experienceIds.length;
            const eduCount = links.educationIds.length;
            const projCount = links.portfolioIds.length;

            // Use index as key since we're remounting the entire component on changes
            // This ensures React properly tracks items during deletion
            const itemKey = `skill-${index}-${skill.name}`;

            return (
              <div 
                key={itemKey} 
                className="skill-item" 
                data-skill-index={index} 
                data-skill-name={skill.name}
              >
                <div className="skill-info">
                  <InlineEditable
                    value={skill.name}
                    onSave={async (newName) => {
                      if (!newName.trim()) return;
                      const updated = [...skills];
                      updated[index] = { ...skill, name: newName.trim() };
                      onUpdate(updated);
                    }}
                    placeholder="Skill name"
                    className="skill-name-inline"
                  />
                  <div className="skill-links">
                    {expCount > 0 && (
                      <span>Experience ({expCount})</span>
                    )}
                    {eduCount > 0 && (
                      <span>Education ({eduCount})</span>
                    )}
                    {projCount > 0 && (
                      <span>Projects ({projCount})</span>
                    )}
                  </div>
                </div>
                <div className="skill-actions" onClick={(e) => e.stopPropagation()}>
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      startEdit(skill, index);
                    }}
                    className="edit-button"
                  >
                    Edit
                  </button>
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      e.stopPropagation();
                      deleteSkill(index);
                    }}
                    className="delete-button"
                    type="button"
                  >
                    Delete
                  </button>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}

// Education Section Component
function EducationSection({
  education,
  certifications,
  onUpdate,
  validationErrors = {},
}: {
  education: Education[];
  certifications: Certification[];
  onUpdate: (edu: Education[], certs: Certification[]) => void;
  validationErrors?: Record<string, string>;
}) {
  const [activeTab, setActiveTab] = useState<"education" | "certifications">("education");
  const [editingEduId, setEditingEduId] = useState<number | "new" | null>(null);
  const [editingCertId, setEditingCertId] = useState<number | "new" | null>(null);
  const [eduFormData, setEduFormData] = useState<Education>({
    institution: "",
    degree: "",
    field_of_study: "",
    start_date: "",
    end_date: "",
    grade: "",
    description: "",
  });
  const [certFormData, setCertFormData] = useState<Certification>({
    name: "",
    issuing_organization: "",
    issue_date: "",
    expiration_date: "",
    credential_id: "",
    credential_url: "",
  });
  const [localErrors, setLocalErrors] = useState<Record<string, string>>({});

  // Education handlers
  function startEditEdu(edu?: Education) {
    if (edu) {
      setEduFormData(edu);
      setEditingEduId(edu.id || "new");
    } else {
      setEduFormData({
        institution: "",
        degree: "",
        field_of_study: "",
        start_date: "",
        end_date: "",
        grade: "",
        description: "",
      });
      setEditingEduId("new");
    }
    setLocalErrors({});
  }

  function saveEducation() {
    const errors: Record<string, string> = {};
    if (!eduFormData.institution.trim()) {
      errors.institution = "Institution is required";
    }
    if (eduFormData.start_date && eduFormData.end_date) {
      const start = new Date(eduFormData.start_date);
      const end = new Date(eduFormData.end_date);
      if (start > end) {
        errors.dates = "End date must be after start date";
      }
    }
    
    if (Object.keys(errors).length > 0) {
      setLocalErrors(errors);
      return;
    }
    
    setLocalErrors({});
    const updated = [...education];
    if (editingEduId === "new") {
      updated.push({ ...eduFormData });
    } else {
      const index = updated.findIndex((e) => e.id === editingEduId);
      if (index >= 0) {
        updated[index] = { ...eduFormData, id: editingEduId as number };
      }
    }
    onUpdate(updated, certifications);
    setEditingEduId(null);
  }

  function deleteEducation(id: number) {
    if (confirm("Delete this education entry?")) {
      onUpdate(education.filter((e) => e.id !== id), certifications);
    }
  }

  // Certification handlers
  function startEditCert(cert?: Certification) {
    if (cert) {
      setCertFormData(cert);
      setEditingCertId(cert.id || "new");
    } else {
      setCertFormData({
        name: "",
        issuing_organization: "",
        issue_date: "",
        expiration_date: "",
        credential_id: "",
        credential_url: "",
      });
      setEditingCertId("new");
    }
    setLocalErrors({});
  }

  function saveCertification() {
    const errors: Record<string, string> = {};
    if (!certFormData.name.trim()) {
      errors.name = "Certification name is required";
    }
    if (certFormData.credential_url && certFormData.credential_url.trim() !== "") {
      try {
        new URL(certFormData.credential_url);
      } catch {
        errors.credential_url = "Please enter a valid URL";
      }
    }
    
    if (Object.keys(errors).length > 0) {
      setLocalErrors(errors);
      return;
    }
    
    setLocalErrors({});
    const updated = [...certifications];
    if (editingCertId === "new") {
      updated.push({ ...certFormData });
    } else {
      const index = updated.findIndex((c) => c.id === editingCertId);
      if (index >= 0) {
        updated[index] = { ...certFormData, id: editingCertId as number };
      }
    }
    onUpdate(education, updated);
    setEditingCertId(null);
  }

  function deleteCertification(id: number) {
    if (confirm("Delete this certification?")) {
      onUpdate(education, certifications.filter((c) => c.id !== id));
    }
  }

  return (
    <div className="profile-section">
      <div className="section-header">
        <h2>Education & Certifications</h2>
        <div className="tabs">
          <button
            className={activeTab === "education" ? "active" : ""}
            onClick={() => setActiveTab("education")}
          >
            Education
          </button>
          <button
            className={activeTab === "certifications" ? "active" : ""}
            onClick={() => setActiveTab("certifications")}
          >
            Certifications
          </button>
        </div>
      </div>

      {activeTab === "education" && (
        <div>
          <button 
            onClick={() => startEditEdu()} 
            className="add-button"
            aria-label="Add new education entry"
          >
            + Add Education
          </button>

          {editingEduId && (
            <div className="edit-form">
              <h3>{editingEduId === "new" ? "Add Education" : "Edit Education"}</h3>
              <div className="form-grid">
                <div className="form-group">
                  <label>
                    Institution <span className="required">*</span>
                  </label>
                  <input
                    type="text"
                    value={eduFormData.institution}
                    onChange={(e) => {
                      setEduFormData({ ...eduFormData, institution: e.target.value });
                      if (localErrors.institution) setLocalErrors({ ...localErrors, institution: "" });
                    }}
                    required
                    style={{
                      borderColor: localErrors.institution ? "#ef4444" : undefined
                    }}
                  />
                  {localErrors.institution && (
                    <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                      {localErrors.institution}
                    </span>
                  )}
                </div>
                <div className="form-group">
                  <label>Degree</label>
                  <input
                    type="text"
                    value={eduFormData.degree || ""}
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, degree: e.target.value })
                    }
                  />
                </div>
                <div className="form-group">
                  <label>Field of Study</label>
                  <input
                    type="text"
                    value={eduFormData.field_of_study || ""}
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, field_of_study: e.target.value })
                    }
                  />
                </div>
                <div className="form-group">
                  <label>Start Date</label>
                  <input
                    type="date"
                    value={eduFormData.start_date ? (eduFormData.start_date.includes('-') && eduFormData.start_date.length === 7 ? `${eduFormData.start_date}-01` : eduFormData.start_date) : ""}
                    onChange={(e) => {
                      const value = e.target.value;
                      // Store full date (YYYY-MM-DD)
                      setEduFormData({ ...eduFormData, start_date: value || "" });
                    }}
                  />
                </div>
                <div className="form-group">
                  <label>End Date</label>
                  <input
                    type="date"
                    value={eduFormData.end_date ? (eduFormData.end_date.includes('-') && eduFormData.end_date.length === 7 ? `${eduFormData.end_date}-01` : eduFormData.end_date) : ""}
                    onChange={(e) => {
                      const value = e.target.value;
                      // Store full date (YYYY-MM-DD)
                      setEduFormData({ ...eduFormData, end_date: value || "" });
                    }}
                  />
                </div>
                <div className="form-group">
                  <label>Grade</label>
                  <input
                    type="text"
                    value={eduFormData.grade || ""}
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, grade: e.target.value })
                    }
                  />
                </div>
                <div className="form-group full-width">
                  <label>Description</label>
                  <textarea
                    value={eduFormData.description || ""}
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, description: e.target.value })
                    }
                    rows={3}
                  />
                </div>
              </div>
              <div className="form-actions">
                <button onClick={() => setEditingEduId(null)} className="cancel-button">
                  Cancel
                </button>
                <button onClick={saveEducation} className="save-button">
                  Save
                </button>
              </div>
            </div>
          )}

          <div className="items-list">
            {education.length === 0 ? (
              <div className="empty-state">No education entries yet.</div>
            ) : (
              education.map((edu) => (
                <div key={edu.id || Math.random()} className="item-card">
                  <div className="item-header">
                    <div>
                      <h3>{edu.institution}</h3>
                      {edu.degree && <p className="item-subtitle">{edu.degree}</p>}
                      {edu.field_of_study && (
                        <p className="item-meta">{edu.field_of_study}</p>
                      )}
                    </div>
                    <div className="item-actions">
                      <button 
                        onClick={() => startEditEdu(edu)} 
                        className="edit-button"
                        aria-label={`Edit education at ${edu.institution}`}
                      >
                        Edit
                      </button>
                      <button
                        onClick={() => edu.id && deleteEducation(edu.id)}
                        className="delete-button"
                        aria-label={`Delete education at ${edu.institution}`}
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}

      {activeTab === "certifications" && (
        <div>
          <button onClick={() => startEditCert()} className="add-button">
            + Add Certification
          </button>

          {editingCertId && (
            <div className="edit-form">
              <h3>
                {editingCertId === "new" ? "Add Certification" : "Edit Certification"}
              </h3>
              <div className="form-grid">
                <div className="form-group">
                  <label>
                    Name <span className="required">*</span>
                  </label>
                  <input
                    type="text"
                    value={certFormData.name}
                    onChange={(e) => {
                      setCertFormData({ ...certFormData, name: e.target.value });
                      if (localErrors.name) setLocalErrors({ ...localErrors, name: "" });
                    }}
                    required
                    style={{
                      borderColor: localErrors.name ? "#ef4444" : undefined
                    }}
                  />
                  {localErrors.name && (
                    <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                      {localErrors.name}
                    </span>
                  )}
                </div>
                <div className="form-group">
                  <label>Issuing Organization</label>
                  <input
                    type="text"
                    value={certFormData.issuing_organization || ""}
                    onChange={(e) =>
                      setCertFormData({
                        ...certFormData,
                        issuing_organization: e.target.value,
                      })
                    }
                  />
                </div>
                <div className="form-group">
                  <label>Issue Date</label>
                  <input
                    type="date"
                    value={certFormData.issue_date ? (certFormData.issue_date.includes('-') && certFormData.issue_date.length === 7 ? `${certFormData.issue_date}-01` : certFormData.issue_date) : ""}
                    onChange={(e) => {
                      const value = e.target.value;
                      // Store full date (YYYY-MM-DD)
                      setCertFormData({ ...certFormData, issue_date: value || "" });
                    }}
                  />
                </div>
                <div className="form-group">
                  <label>Expiration Date</label>
                  <input
                    type="date"
                    value={certFormData.expiration_date ? (certFormData.expiration_date.includes('-') && certFormData.expiration_date.length === 7 ? `${certFormData.expiration_date}-01` : certFormData.expiration_date) : ""}
                    onChange={(e) => {
                      const value = e.target.value;
                      // Store full date (YYYY-MM-DD)
                      setCertFormData({
                        ...certFormData,
                        expiration_date: value || "",
                      });
                    }}
                  />
                </div>
                <div className="form-group">
                  <label>Credential ID</label>
                  <input
                    type="text"
                    value={certFormData.credential_id || ""}
                    onChange={(e) =>
                      setCertFormData({ ...certFormData, credential_id: e.target.value })
                    }
                  />
                </div>
                <div className="form-group">
                  <label>Credential URL</label>
                  <input
                    type="url"
                    value={certFormData.credential_url || ""}
                    onChange={(e) =>
                      setCertFormData({ ...certFormData, credential_url: e.target.value })
                    }
                  />
                </div>
              </div>
              <div className="form-actions">
                <button onClick={() => setEditingCertId(null)} className="cancel-button">
                  Cancel
                </button>
                <button onClick={saveCertification} className="save-button">
                  Save
                </button>
              </div>
            </div>
          )}

          <div className="items-list">
            {certifications.length === 0 ? (
              <div className="empty-state">No certifications yet.</div>
            ) : (
              certifications.map((cert) => (
                <div key={cert.id || Math.random()} className="item-card">
                  <div className="item-header">
                    <div>
                      <h3>{cert.name}</h3>
                      {cert.issuing_organization && (
                        <p className="item-subtitle">{cert.issuing_organization}</p>
                      )}
                    </div>
                    <div className="item-actions">
                      <button onClick={() => startEditCert(cert)} className="edit-button">
                        Edit
                      </button>
                      <button
                        onClick={() => cert.id && deleteCertification(cert.id)}
                        className="delete-button"
                      >
                        Delete
                      </button>
                    </div>
                  </div>
                </div>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  );
}

// Portfolio Section Component
function PortfolioSection({
  portfolio,
  onUpdate,
  validationErrors = {},
}: {
  portfolio: PortfolioItem[];
  onUpdate: (portfolio: PortfolioItem[]) => void;
  validationErrors?: Record<string, string>;
}) {
  const [editingId, setEditingId] = useState<number | "new" | null>(null);
  const [formData, setFormData] = useState<PortfolioItem>({
    title: "",
    url: "",
    description: "",
    role: "",
    tech_stack: "",
    highlighted: false,
  });

  function startEdit(item?: PortfolioItem) {
    if (item) {
      setFormData(item);
      setEditingId(item.id || "new");
    } else {
      setFormData({
        title: "",
        url: "",
        description: "",
        role: "",
        tech_stack: "",
        highlighted: false,
      });
      setEditingId("new");
    }
    setLocalErrors({});
  }

  function cancelEdit() {
    setEditingId(null);
    setLocalErrors({});
  }

  const [localErrors, setLocalErrors] = useState<Record<string, string>>({});

  function savePortfolioItem() {
    const errors: Record<string, string> = {};
    if (!formData.title.trim()) {
      errors.title = "Title is required";
    }
    if (formData.url && formData.url.trim() !== "") {
      try {
        new URL(formData.url);
      } catch {
        errors.url = "Please enter a valid URL";
      }
    }
    
    if (Object.keys(errors).length > 0) {
      setLocalErrors(errors);
      return;
    }
    
    setLocalErrors({});
    const updated = [...portfolio];
    if (editingId === "new") {
      updated.push({ ...formData });
    } else {
      const index = updated.findIndex((p) => p.id === editingId);
      if (index >= 0) {
        updated[index] = { ...formData, id: editingId as number };
      }
    }
    onUpdate(updated);
    cancelEdit();
  }

  function deletePortfolioItem(id: number) {
    if (confirm("Delete this portfolio item?")) {
      onUpdate(portfolio.filter((p) => p.id !== id));
    }
  }

  return (
    <div className="profile-section">
      <div className="section-header">
        <h2>Portfolio</h2>
        <div style={{ display: "flex", gap: "0.5rem" }}>
          <PortfolioExportButton portfolio={portfolio} />
          <button 
            onClick={() => startEdit()} 
            className="add-button"
            aria-label="Add new portfolio item"
          >
            + Add Portfolio Item
          </button>
        </div>
      </div>

      {editingId && (
        <div className="edit-form">
          <h3>{editingId === "new" ? "Add Portfolio Item" : "Edit Portfolio Item"}</h3>
          <div className="form-grid">
            <div className="form-group">
              <label>
                Title <span className="required">*</span>
              </label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) => {
                  setFormData({ ...formData, title: e.target.value });
                  if (localErrors.title) setLocalErrors({ ...localErrors, title: "" });
                }}
                required
                style={{
                  borderColor: localErrors.title ? "#ef4444" : undefined
                }}
              />
              {localErrors.title && (
                <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                  {localErrors.title}
                </span>
              )}
            </div>
            <div className="form-group">
              <label>URL</label>
              <input
                type="url"
                value={formData.url || ""}
                onChange={(e) => {
                  setFormData({ ...formData, url: e.target.value });
                  if (localErrors.url) setLocalErrors({ ...localErrors, url: "" });
                }}
                style={{
                  borderColor: localErrors.url ? "#ef4444" : undefined
                }}
              />
              {localErrors.url && (
                <span style={{ color: "#ef4444", fontSize: "0.875rem", marginTop: "0.25rem" }}>
                  {localErrors.url}
                </span>
              )}
            </div>
            <div className="form-group">
              <label>Role</label>
              <input
                type="text"
                value={formData.role || ""}
                onChange={(e) =>
                  setFormData({ ...formData, role: e.target.value })
                }
                placeholder="What you did in this project"
              />
            </div>
            <div className="form-group">
              <label>Tech Stack (comma-separated)</label>
              <input
                type="text"
                value={formData.tech_stack || ""}
                onChange={(e) =>
                  setFormData({ ...formData, tech_stack: e.target.value })
                }
              />
            </div>
            <div className="form-group">
              <label>
                <input
                  type="checkbox"
                  checked={formData.highlighted}
                  onChange={(e) =>
                    setFormData({ ...formData, highlighted: e.target.checked })
                  }
                />
                Highlight this project (for AI emphasis)
              </label>
            </div>
            <div className="form-group full-width">
              <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
                <label>Description</label>
                {formData.description && formData.id && (
                  <button
                    type="button"
                    onClick={async () => {
                      try {
                        const rewritten = await invoke<string>("rewrite_portfolio_description", {
                          portfolioId: formData.id,
                          description: formData.description
                        });
                        setFormData({ ...formData, description: rewritten });
                      } catch (err: any) {
                        // Show error in a non-blocking way
                        const message = document.createElement("div");
                        message.textContent = err?.message || "Failed to rewrite description";
                        message.style.cssText = "position: fixed; top: 20px; right: 20px; background: #ef4444; color: white; padding: 1rem; border-radius: 0.5rem; z-index: 10000; box-shadow: 0 4px 6px rgba(0,0,0,0.1);";
                        document.body.appendChild(message);
                        setTimeout(() => message.remove(), 5000);
                      }
                    }}
                    style={{
                      padding: "0.375rem 0.75rem",
                      backgroundColor: "#6366f1",
                      color: "white",
                      border: "none",
                      borderRadius: "0.375rem",
                      cursor: "pointer",
                      fontSize: "0.875rem"
                    }}
                  >
                    ✨ Rewrite with AI
                  </button>
                )}
              </div>
              <textarea
                value={formData.description || ""}
                onChange={(e) =>
                  setFormData({ ...formData, description: e.target.value })
                }
                rows={4}
              />
            </div>
          </div>
          <div className="form-actions">
            <button onClick={cancelEdit} className="cancel-button">
              Cancel
            </button>
            <button onClick={savePortfolioItem} className="save-button">
              Save
            </button>
          </div>
        </div>
      )}

      <div className="items-list">
        {portfolio.length === 0 ? (
          <div className="empty-state">No portfolio items yet.</div>
        ) : (
          portfolio.map((item) => (
            <div
              key={item.id || Math.random()}
              className={`item-card ${item.highlighted ? "highlighted" : ""}`}
            >
              <div className="item-header">
                <div>
                  <h3>
                    {item.title}
                    {item.highlighted && <span className="highlight-badge">★</span>}
                  </h3>
                  {item.url && (
                    <p className="item-subtitle">
                      <a href={item.url} target="_blank" rel="noopener noreferrer">
                        {item.url}
                      </a>
                    </p>
                  )}
                  {item.role && <p className="item-meta">{item.role}</p>}
                </div>
                <div className="item-actions">
                  <button onClick={() => startEdit(item)} className="edit-button">
                    Edit
                  </button>
                  <button
                    onClick={() => item.id && deletePortfolioItem(item.id)}
                    className="delete-button"
                  >
                    Delete
                  </button>
                </div>
              </div>
              {item.description && (
                <p className="item-description">{item.description}</p>
              )}
              {item.tech_stack && (
                <div className="item-tags">
                  {item.tech_stack.split(",").map((tech, i) => (
                    <span key={i} className="tag">
                      {tech.trim()}
                    </span>
                  ))}
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
}

// Portfolio Export Button Component
function PortfolioExportButton({ portfolio }: { portfolio: PortfolioItem[] }) {
  const [showExportMenu, setShowExportMenu] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  async function handleExport(format: "html" | "markdown" | "text", highlightedOnly: boolean) {
    setIsExporting(true);
    try {
      let content: string;
      let extension: string;

      if (format === "html") {
        content = await invoke<string>("export_portfolio_html", { includeHighlightedOnly: highlightedOnly });
        extension = "html";
      } else if (format === "markdown") {
        content = await invoke<string>("export_portfolio_markdown", { includeHighlightedOnly: highlightedOnly });
        extension = "md";
      } else {
        content = await invoke<string>("export_portfolio_text", { includeHighlightedOnly: highlightedOnly });
        extension = "txt";
      }

      const { save } = await import("@tauri-apps/plugin-dialog");
      const { writeTextFile } = await import("@tauri-apps/plugin-fs");

      const fileName = `portfolio${highlightedOnly ? "-highlighted" : ""}.${extension}`;
      const filePath = await save({
        defaultPath: fileName,
        filters: [{
          name: format.toUpperCase(),
          extensions: [extension]
        }]
      });

      if (filePath) {
        await writeTextFile(filePath, content);
        showToast(`Portfolio exported successfully as ${format.toUpperCase()}`, "success");
      }
    } catch (err: any) {
      showToast(err?.message || "Failed to export portfolio", "error");
    } finally {
      setIsExporting(false);
      setShowExportMenu(false);
    }
  }

  return (
    <div style={{ position: "relative" }}>
      <button
        onClick={() => setShowExportMenu(!showExportMenu)}
        className="add-button"
        disabled={isExporting || portfolio.length === 0}
        aria-label="Export portfolio"
      >
        {isExporting ? "Exporting..." : "📤 Export"}
      </button>
      {showExportMenu && (
        <div className="export-menu">
          <div className="export-menu-header">Export Portfolio</div>
          <div className="export-menu-section">
            <div className="export-menu-label">All Items</div>
            <button onClick={() => handleExport("html", false)}>Export as HTML</button>
            <button onClick={() => handleExport("markdown", false)}>Export as Markdown</button>
            <button onClick={() => handleExport("text", false)}>Export as Text</button>
          </div>
          <div className="export-menu-section">
            <div className="export-menu-label">Highlighted Only</div>
            <button onClick={() => handleExport("html", true)}>Export as HTML</button>
            <button onClick={() => handleExport("markdown", true)}>Export as Markdown</button>
            <button onClick={() => handleExport("text", true)}>Export as Text</button>
          </div>
        </div>
      )}
    </div>
  );
}
