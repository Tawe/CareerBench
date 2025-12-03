import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
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

  async function saveProfile() {
    setIsSaving(true);
    setError(null);
    try {
      const result = await invoke<UserProfileData>("save_user_profile_data", { data });
      setData(result);
      setIsDirty(false);
    } catch (err: any) {
      setError(err?.message || "Failed to save profile");
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

  if (isLoading) {
    return (
      <div className="profile">
        <div className="loading">Loading profile...</div>
      </div>
    );
  }

  return (
    <div className="profile">
      <div className="profile-header">
        <h1>Profile</h1>
        <div className="profile-actions">
          {isDirty && (
            <span className="unsaved-indicator">Unsaved changes</span>
          )}
          <button
            onClick={saveProfile}
            disabled={!isDirty || isSaving}
            className="save-button"
          >
            {isSaving ? "Saving..." : "Save"}
          </button>
        </div>
      </div>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}

      <div className="profile-content content-constrained">
        <BasicInfoSection profile={data.profile} onUpdate={updateProfile} />

        <ExperienceSection
          experience={data.experience}
          onUpdate={(exp) => {
            setData((prev) => ({ ...prev, experience: exp }));
            setIsDirty(true);
          }}
        />

        <SkillsSection
          skills={data.skills}
          experience={data.experience}
          education={data.education}
          portfolio={data.portfolio}
          onUpdate={(skills) => {
            setData((prev) => ({ ...prev, skills }));
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
        />

        <PortfolioSection
          portfolio={data.portfolio}
          onUpdate={(portfolio) => {
            setData((prev) => ({ ...prev, portfolio }));
            setIsDirty(true);
          }}
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
}: {
  profile?: UserProfile;
  onUpdate: (field: keyof UserProfile, value: string) => void;
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
          <input
            type="text"
            value={profile?.full_name || ""}
            onChange={(e) => onUpdate("full_name", e.target.value)}
            placeholder="John Doe"
            required
          />
        </div>

        <div className="form-group">
          <label>Location</label>
          <input
            type="text"
            value={profile?.location || ""}
            onChange={(e) => onUpdate("location", e.target.value)}
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
          <label>Professional Summary</label>
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
}: {
  experience: Experience[];
  onUpdate: (exp: Experience[]) => void;
}) {
  const [editingId, setEditingId] = useState<number | "new" | null>(null);
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

  function startEdit(exp?: Experience) {
    if (exp) {
      setFormData(exp);
      setEditingId(exp.id || "new");
    } else {
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
      setEditingId("new");
    }
  }

  function cancelEdit() {
    setEditingId(null);
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

  function saveExperience() {
    if (!formData.company || !formData.title) {
      alert("Company and Title are required");
      return;
    }

    let updated = [...experience];
    if (editingId === "new") {
      updated.push({ ...formData });
    } else {
      const index = updated.findIndex((e) => e.id === editingId);
      if (index >= 0) {
        updated[index] = { ...formData, id: editingId as number };
      }
    }
    onUpdate(updated);
    cancelEdit();
  }

  function deleteExperience(id: number) {
    if (confirm("Are you sure you want to delete this experience?")) {
      onUpdate(experience.filter((e) => e.id !== id));
    }
  }

  return (
    <div className="profile-section">
      <div className="section-header">
        <h2>Experience</h2>
        <button onClick={() => startEdit()} className="add-button">
          + Add Experience
        </button>
      </div>

      {editingId && (
        <div className="edit-form">
          <h3>{editingId === "new" ? "Add Experience" : "Edit Experience"}</h3>
          <div className="form-grid">
            <div className="form-group">
              <label>
                Company <span className="required">*</span>
              </label>
              <input
                type="text"
                value={formData.company}
                onChange={(e) =>
                  setFormData({ ...formData, company: e.target.value })
                }
                required
              />
            </div>

            <div className="form-group">
              <label>
                Job Title <span className="required">*</span>
              </label>
              <input
                type="text"
                value={formData.title}
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
                required
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
              <label>Start Date</label>
              <input
                type="month"
                value={formData.start_date || ""}
                onChange={(e) =>
                  setFormData({ ...formData, start_date: e.target.value })
                }
              />
            </div>

            <div className="form-group">
              <label>End Date</label>
              <input
                type="month"
                value={formData.end_date || ""}
                onChange={(e) =>
                  setFormData({ ...formData, end_date: e.target.value })
                }
                disabled={formData.is_current}
              />
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
          experience.map((exp) => (
            <div key={exp.id || Math.random()} className="item-card">
              <div className="item-header">
                <div>
                  <h3>{exp.title}</h3>
                  <p className="item-subtitle">{exp.company}</p>
                  <p className="item-meta">
                    {exp.start_date && (
                      <span>
                        {new Date(exp.start_date + "-01").toLocaleDateString("en-US", {
                          month: "short",
                          year: "numeric",
                        })}
                      </span>
                    )}
                    {exp.start_date && (exp.is_current || exp.end_date) && " - "}
                    {exp.is_current ? (
                      <span>Present</span>
                    ) : (
                      exp.end_date && (
                        <span>
                          {new Date(exp.end_date + "-01").toLocaleDateString("en-US", {
                            month: "short",
                            year: "numeric",
                          })}
                        </span>
                      )
                    )}
                    {exp.location && ` • ${exp.location}`}
                  </p>
                </div>
                <div className="item-actions">
                  <button
                    onClick={() => startEdit(exp)}
                    className="edit-button"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => exp.id && deleteExperience(exp.id)}
                    className="delete-button"
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
    onUpdate([...skills, newSkill]);
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

  function saveSkill() {
    if (!formData.name.trim()) {
      alert("Skill name is required");
      return;
    }

    const links: SkillLinks = {
      experienceIds: linkedExperienceIds,
      educationIds: linkedEducationIds,
      portfolioIds: linkedPortfolioIds,
    };
    const skillToSave: Skill = {
      ...formData,
      notes: JSON.stringify({ links }),
    };

    let updated = [...skills];
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
    if (confirm("Are you sure you want to delete this skill?")) {
      onUpdate(
        skills.filter((_, i) => i !== index)
      );
    }
  }

  return (
    <div className="profile-section">
      <div className="section-header">
        <h2>Skills</h2>
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
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                required
              />
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

      <div className="skills-grid">
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

            return (
              <div key={skill.id ?? index} className="skill-item">
                <div className="skill-info">
                  <span className="skill-name">{skill.name}</span>
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
                <div className="skill-actions">
                  <button
                    onClick={() => startEdit(skill, index)}
                    className="edit-button"
                  >
                    Edit
                  </button>
                  <button
                    onClick={() => deleteSkill(index)}
                    className="delete-button"
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
}: {
  education: Education[];
  certifications: Certification[];
  onUpdate: (edu: Education[], certs: Certification[]) => void;
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
  }

  function saveEducation() {
    if (!eduFormData.institution.trim()) {
      alert("Institution is required");
      return;
    }
    let updated = [...education];
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
  }

  function saveCertification() {
    if (!certFormData.name.trim()) {
      alert("Certification name is required");
      return;
    }
    let updated = [...certifications];
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
          <button onClick={() => startEditEdu()} className="add-button">
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
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, institution: e.target.value })
                    }
                    required
                  />
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
                    type="month"
                    value={eduFormData.start_date || ""}
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, start_date: e.target.value })
                    }
                  />
                </div>
                <div className="form-group">
                  <label>End Date</label>
                  <input
                    type="month"
                    value={eduFormData.end_date || ""}
                    onChange={(e) =>
                      setEduFormData({ ...eduFormData, end_date: e.target.value })
                    }
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
                      <button onClick={() => startEditEdu(edu)} className="edit-button">
                        Edit
                      </button>
                      <button
                        onClick={() => edu.id && deleteEducation(edu.id)}
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
                    onChange={(e) =>
                      setCertFormData({ ...certFormData, name: e.target.value })
                    }
                    required
                  />
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
                    type="month"
                    value={certFormData.issue_date || ""}
                    onChange={(e) =>
                      setCertFormData({ ...certFormData, issue_date: e.target.value })
                    }
                  />
                </div>
                <div className="form-group">
                  <label>Expiration Date</label>
                  <input
                    type="month"
                    value={certFormData.expiration_date || ""}
                    onChange={(e) =>
                      setCertFormData({
                        ...certFormData,
                        expiration_date: e.target.value,
                      })
                    }
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
}: {
  portfolio: PortfolioItem[];
  onUpdate: (portfolio: PortfolioItem[]) => void;
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
  }

  function cancelEdit() {
    setEditingId(null);
  }

  function savePortfolioItem() {
    if (!formData.title.trim()) {
      alert("Title is required");
      return;
    }
    let updated = [...portfolio];
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
        <button onClick={() => startEdit()} className="add-button">
          + Add Portfolio Item
        </button>
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
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
                required
              />
            </div>
            <div className="form-group">
              <label>URL</label>
              <input
                type="url"
                value={formData.url || ""}
                onChange={(e) =>
                  setFormData({ ...formData, url: e.target.value })
                }
              />
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
              <label>Description</label>
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
