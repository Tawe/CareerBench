import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import type { Company, CompanyWithStats } from "../commands/types";
import "./Companies.css";

export default function Companies() {
  const [companies, setCompanies] = useState<CompanyWithStats[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedCompany, setSelectedCompany] = useState<Company | null>(null);
  const [showCompanyModal, setShowCompanyModal] = useState(false);
  const [editingCompany, setEditingCompany] = useState<Company | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  useEffect(() => {
    loadCompanies();
  }, [searchQuery]);

  async function loadCompanies() {
    setIsLoading(true);
    try {
      const result = await invoke<CompanyWithStats[]>("get_companies_with_stats", {
        searchQuery: searchQuery || null,
      });
      setCompanies(result);
    } catch (err: any) {
      showToast(err?.message || "Failed to load companies", "error");
    } finally {
      setIsLoading(false);
    }
  }

  async function loadCompanyDetails(company: CompanyWithStats) {
    if (!company.id) return;
    try {
      const result = await invoke<Company>("get_company", { companyId: company.id });
      setSelectedCompany(result);
    } catch (err: any) {
      showToast(err?.message || "Failed to load company details", "error");
    }
  }

  async function handleDeleteCompany(companyId: number) {
    const forceDelete = confirm(
      "Are you sure you want to delete this company?\n\n" +
      "If this company is linked to jobs or applications, they will be unlinked automatically."
    );
    if (!forceDelete) return;
    
    try {
      await invoke("delete_company", { companyId, force: true });
      showToast("Company deleted", "success");
      setSelectedCompany(null);
      loadCompanies();
    } catch (err: any) {
      showToast(err?.message || "Failed to delete company", "error");
    }
  }

  function handleEditCompany(company: CompanyWithStats) {
    setEditingCompany(company as Company);
    setShowCompanyModal(true);
  }

  function handleNewCompany() {
    setEditingCompany(null);
    setShowCompanyModal(true);
  }

  if (isLoading) {
    return (
      <div className="companies-page">
        <div className="companies-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="companies-content">
          <LoadingSkeleton variant="card" width="100%" height="400px" />
        </div>
      </div>
    );
  }

  return (
    <div className="companies-page">
      <div className="companies-header">
        <h1>Companies</h1>
        <button onClick={handleNewCompany} className="btn-primary">
          + New Company
        </button>
      </div>

      <div className="companies-filters">
        <input
          type="text"
          placeholder="Search companies..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />
      </div>

      <div className="companies-content">
        <div className="companies-list">
          <h2>Companies ({companies.length})</h2>
          {companies.length === 0 ? (
            <div className="empty-state">
              <p>No companies found. Create your first company.</p>
            </div>
          ) : (
            companies.map((company) => (
              <div
                key={company.id}
                className={`company-card ${selectedCompany?.id === company.id ? "active" : ""}`}
                onClick={() => loadCompanyDetails(company)}
              >
                <div className="company-header">
                  <h3>{company.name}</h3>
                  <div className="company-actions">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEditCompany(company);
                      }}
                      className="icon-button"
                      aria-label="Edit company"
                    >
                      ✏️
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        company.id && handleDeleteCompany(company.id);
                      }}
                      className="icon-button delete"
                      aria-label="Delete company"
                    >
                      ×
                    </button>
                  </div>
                </div>
                {company.industry && (
                  <div className="company-industry">{company.industry}</div>
                )}
                {company.location && (
                  <div className="company-location">{company.location}</div>
                )}
                {company.companySize && (
                  <div className="company-size">{company.companySize}</div>
                )}
                <div className="company-stats">
                  <span>{company.jobCount} job{company.jobCount !== 1 ? "s" : ""}</span>
                  <span>{company.applicationCount} application{company.applicationCount !== 1 ? "s" : ""}</span>
                </div>
              </div>
            ))
          )}
        </div>

        {selectedCompany && (
          <div className="company-detail">
            <div className="detail-header">
              <div>
                <h2>{selectedCompany.name}</h2>
                {selectedCompany.industry && (
                  <div className="detail-industry">{selectedCompany.industry}</div>
                )}
              </div>
              <button onClick={() => setSelectedCompany(null)} aria-label="Close">×</button>
            </div>

            <div className="detail-info">
              {selectedCompany.website && (
                <div className="info-item">
                  <strong>Website:</strong>{" "}
                  <a href={selectedCompany.website} target="_blank" rel="noopener noreferrer">
                    {selectedCompany.website}
                  </a>
                </div>
              )}
              {selectedCompany.location && (
                <div className="info-item">
                  <strong>Location:</strong> {selectedCompany.location}
                </div>
              )}
              {selectedCompany.companySize && (
                <div className="info-item">
                  <strong>Company Size:</strong> {selectedCompany.companySize}
                </div>
              )}
              {selectedCompany.description && (
                <div className="info-item">
                  <strong>Description:</strong>
                  <p>{selectedCompany.description}</p>
                </div>
              )}
              {selectedCompany.mission && (
                <div className="info-item">
                  <strong>Mission:</strong>
                  <p>{selectedCompany.mission}</p>
                </div>
              )}
              {selectedCompany.vision && (
                <div className="info-item">
                  <strong>Vision:</strong>
                  <p>{selectedCompany.vision}</p>
                </div>
              )}
              {selectedCompany.values && (
                <div className="info-item">
                  <strong>Values:</strong>
                  <p>{selectedCompany.values}</p>
                </div>
              )}
              {selectedCompany.notes && (
                <div className="info-item">
                  <strong>Notes:</strong>
                  <p>{selectedCompany.notes}</p>
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {showCompanyModal && (
        <CompanyModal
          company={editingCompany}
          onClose={() => {
            setShowCompanyModal(false);
            setEditingCompany(null);
          }}
          onSave={() => {
            setShowCompanyModal(false);
            setEditingCompany(null);
            loadCompanies();
            if (selectedCompany && editingCompany?.id === selectedCompany.id) {
              loadCompanyDetails(editingCompany as CompanyWithStats);
            }
          }}
        />
      )}
    </div>
  );
}

interface CompanyModalProps {
  company: Company | null;
  onClose: () => void;
  onSave: () => void;
}

function CompanyModal({ company, onClose, onSave }: CompanyModalProps) {
  const [name, setName] = useState(company?.name || "");
  const [website, setWebsite] = useState(company?.website || "");
  const [industry, setIndustry] = useState(company?.industry || "");
  const [companySize, setCompanySize] = useState(company?.companySize || "");
  const [location, setLocation] = useState(company?.location || "");
  const [description, setDescription] = useState(company?.description || "");
  const [mission, setMission] = useState(company?.mission || "");
  const [vision, setVision] = useState(company?.vision || "");
  const [values, setValues] = useState(company?.values || "");
  const [notes, setNotes] = useState(company?.notes || "");
  const [isSaving, setIsSaving] = useState(false);
  const [fetchUrl, setFetchUrl] = useState("");
  const [isFetching, setIsFetching] = useState(false);

  async function handleFetchFromUrl() {
    if (!fetchUrl.trim()) {
      showToast("Please enter a website URL", "error");
      return;
    }

    // Validate URL format
    try {
      new URL(fetchUrl.trim());
    } catch {
      showToast("Please enter a valid URL (e.g., https://example.com)", "error");
      return;
    }

    setIsFetching(true);
    try {
      showToast("Fetching company information...", "info");
      // First clear any cached data for this URL to get fresh data
      await invoke("clear_company_fetch_cache", { url: fetchUrl.trim() });
      const fetched = await invoke<Company>("fetch_company_info_from_url", {
        url: fetchUrl.trim(),
        bypassCache: false, // We already cleared it above
      });

      // Populate form fields with fetched data
      if (fetched.name) setName(fetched.name);
      if (fetched.website) setWebsite(fetched.website);
      if (fetched.industry) setIndustry(fetched.industry);
      if (fetched.companySize) setCompanySize(fetched.companySize);
      if (fetched.location) setLocation(fetched.location);
      if (fetched.description) setDescription(fetched.description);
      if (fetched.mission) setMission(fetched.mission);
      if (fetched.vision) setVision(fetched.vision);
      if (fetched.values) setValues(fetched.values);
      if (fetched.mission) setMission(fetched.mission);
      if (fetched.vision) setVision(fetched.vision);
      if (fetched.values) setValues(fetched.values);

      showToast("Company information fetched successfully!", "success");
      setFetchUrl(""); // Clear the URL input
    } catch (err: any) {
      showToast(err?.message || "Failed to fetch company information", "error");
    } finally {
      setIsFetching(false);
    }
  }

  async function handleSave() {
    if (!name.trim()) {
      showToast("Company name is required", "error");
      return;
    }

    setIsSaving(true);
    try {
      if (company?.id) {
        await invoke("update_company", {
          companyId: company.id,
          name: name.trim() || null,
          website: website.trim() || null,
          industry: industry.trim() || null,
          companySize: companySize.trim() || null,
          location: location.trim() || null,
          description: description.trim() || null,
          mission: mission.trim() || null,
          vision: vision.trim() || null,
          values: values.trim() || null,
          notes: notes.trim() || null,
        });
        showToast("Company updated", "success");
      } else {
        await invoke("create_company", {
          name: name.trim(),
          website: website.trim() || null,
          industry: industry.trim() || null,
          companySize: companySize.trim() || null,
          location: location.trim() || null,
          description: description.trim() || null,
          mission: mission.trim() || null,
          vision: vision.trim() || null,
          values: values.trim() || null,
          notes: notes.trim() || null,
        });
        showToast("Company created", "success");
      }
      onSave();
    } catch (err: any) {
      showToast(err?.message || "Failed to save company", "error");
    } finally {
      setIsSaving(false);
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>{company ? "Edit Company" : "New Company"}</h2>
          <button onClick={onClose} aria-label="Close">×</button>
        </div>
        <div className="modal-body">
          <div className="form-group">
            <label htmlFor="fetchUrl">Fetch from Website URL</label>
            <div style={{ display: "flex", gap: "0.5rem" }}>
              <input
                id="fetchUrl"
                type="url"
                value={fetchUrl}
                onChange={(e) => setFetchUrl(e.target.value)}
                placeholder="https://example.com"
                disabled={isFetching}
                style={{ flex: 1 }}
              />
              <button
                onClick={handleFetchFromUrl}
                className="btn-secondary"
                disabled={isFetching || !fetchUrl.trim()}
              >
                {isFetching ? "Fetching..." : "Fetch Info"}
              </button>
            </div>
            <small style={{ color: "var(--text-secondary)", fontSize: "0.875rem" }}>
              Enter a company website URL to automatically fetch company information using AI
            </small>
          </div>

          <div style={{ margin: "1.5rem 0", borderTop: "1px solid var(--border-color)" }}></div>

          <div className="form-group">
            <label htmlFor="name">Company Name *</label>
            <input
              id="name"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="e.g., Acme Corp"
              required
            />
          </div>
          <div className="form-group">
            <label htmlFor="website">Website</label>
            <input
              id="website"
              type="url"
              value={website}
              onChange={(e) => setWebsite(e.target.value)}
              placeholder="https://example.com"
            />
          </div>
          <div className="form-row">
            <div className="form-group">
              <label htmlFor="industry">Industry</label>
              <input
                id="industry"
                type="text"
                value={industry}
                onChange={(e) => setIndustry(e.target.value)}
                placeholder="e.g., Technology"
              />
            </div>
            <div className="form-group">
              <label htmlFor="companySize">Company Size</label>
              <input
                id="companySize"
                type="text"
                value={companySize}
                onChange={(e) => setCompanySize(e.target.value)}
                placeholder="e.g., 50-200"
              />
            </div>
          </div>
          <div className="form-group">
            <label htmlFor="location">Location</label>
            <input
              id="location"
              type="text"
              value={location}
              onChange={(e) => setLocation(e.target.value)}
              placeholder="e.g., San Francisco, CA"
            />
          </div>
          <div className="form-group">
            <label htmlFor="description">Description</label>
            <textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Company description..."
              rows={4}
            />
          </div>
          <div className="form-group">
            <label htmlFor="mission">Mission</label>
            <textarea
              id="mission"
              value={mission}
              onChange={(e) => setMission(e.target.value)}
              placeholder="Company mission statement..."
              rows={3}
            />
          </div>
          <div className="form-group">
            <label htmlFor="vision">Vision</label>
            <textarea
              id="vision"
              value={vision}
              onChange={(e) => setVision(e.target.value)}
              placeholder="Company vision statement..."
              rows={3}
            />
          </div>
          <div className="form-group">
            <label htmlFor="values">Values</label>
            <textarea
              id="values"
              value={values}
              onChange={(e) => setValues(e.target.value)}
              placeholder="Company values (e.g., Innovation, Integrity, Customer Focus)..."
              rows={3}
            />
          </div>
          <div className="form-group">
            <label htmlFor="notes">Notes</label>
            <textarea
              id="notes"
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              placeholder="Personal notes about this company..."
              rows={4}
            />
          </div>
        </div>
        <div className="modal-footer">
          <button onClick={onClose} className="btn-secondary">
            Cancel
          </button>
          <button onClick={handleSave} className="btn-primary" disabled={isSaving}>
            {isSaving ? "Saving..." : company ? "Update" : "Create"}
          </button>
        </div>
      </div>
    </div>
  );
}

