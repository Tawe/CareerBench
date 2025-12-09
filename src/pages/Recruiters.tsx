import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import type {
  RecruiterContact,
  RecruiterInteraction,
} from "../commands/types";
import "./Recruiters.css";

export default function Recruiters() {
  const [contacts, setContacts] = useState<RecruiterContact[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedContact, setSelectedContact] = useState<RecruiterContact | null>(null);
  const [interactions, setInteractions] = useState<RecruiterInteraction[]>([]);
  const [showContactModal, setShowContactModal] = useState(false);
  const [showInteractionModal, setShowInteractionModal] = useState(false);
  const [editingContact, setEditingContact] = useState<RecruiterContact | null>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [companyFilter, setCompanyFilter] = useState("");

  useEffect(() => {
    loadContacts();
  }, [companyFilter, searchQuery]);

  async function loadContacts() {
    setIsLoading(true);
    try {
      const result = await invoke<RecruiterContact[]>("get_recruiter_contacts", {
        companyFilter: companyFilter || null,
        searchQuery: searchQuery || null,
      });
      setContacts(result);
    } catch (err: any) {
      showToast(err?.message || "Failed to load contacts", "error");
    } finally {
      setIsLoading(false);
    }
  }

  async function loadContactDetails(contact: RecruiterContact) {
    setSelectedContact(contact);
    try {
      const result = await invoke<RecruiterInteraction[]>("get_interactions_for_contact", {
        contactId: contact.id,
      });
      setInteractions(result);
    } catch (err: any) {
      showToast(err?.message || "Failed to load interactions", "error");
    }
  }

  async function handleDeleteContact(contactId: number) {
    if (!confirm("Are you sure you want to delete this contact?")) return;
    try {
      await invoke("delete_recruiter_contact", { contactId });
      showToast("Contact deleted", "success");
      setSelectedContact(null);
      loadContacts();
    } catch (err: any) {
      showToast(err?.message || "Failed to delete contact", "error");
    }
  }

  async function handleDeleteInteraction(interactionId: number) {
    if (!confirm("Are you sure you want to delete this interaction?")) return;
    try {
      await invoke("delete_interaction", { interactionId });
      showToast("Interaction deleted", "success");
      if (selectedContact) {
        loadContactDetails(selectedContact);
      }
    } catch (err: any) {
      showToast(err?.message || "Failed to delete interaction", "error");
    }
  }

  function handleEditContact(contact: RecruiterContact) {
    setEditingContact(contact);
    setShowContactModal(true);
  }

  function handleNewContact() {
    setEditingContact(null);
    setShowContactModal(true);
  }

  function handleNewInteraction() {
    if (!selectedContact) return;
    setShowInteractionModal(true);
  }

  if (isLoading) {
    return (
      <div className="recruiters-page">
        <div className="recruiters-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="recruiters-content">
          <LoadingSkeleton variant="card" width="100%" height="400px" />
        </div>
      </div>
    );
  }

  return (
    <div className="recruiters-page">
      <div className="recruiters-header">
        <h1>Recruiter CRM</h1>
        <button onClick={handleNewContact} className="btn-primary">
          + New Contact
        </button>
      </div>

      <div className="recruiters-filters">
        <input
          type="text"
          placeholder="Search contacts..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />
        <input
          type="text"
          placeholder="Filter by company..."
          value={companyFilter}
          onChange={(e) => setCompanyFilter(e.target.value)}
          className="search-input"
        />
      </div>

      <div className="recruiters-content">
        <div className="contacts-list">
          <h2>Contacts ({contacts.length})</h2>
          {contacts.length === 0 ? (
            <div className="empty-state">
              <p>No contacts found. Create your first recruiter contact.</p>
            </div>
          ) : (
            contacts.map((contact) => (
              <div
                key={contact.id}
                className={`contact-card ${selectedContact?.id === contact.id ? "active" : ""}`}
                onClick={() => loadContactDetails(contact)}
              >
                <div className="contact-header">
                  <h3>{contact.name}</h3>
                  <div className="contact-actions">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEditContact(contact);
                      }}
                      className="icon-button"
                      aria-label="Edit contact"
                    >
                      ✏️
                    </button>
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        contact.id && handleDeleteContact(contact.id);
                      }}
                      className="icon-button delete"
                      aria-label="Delete contact"
                    >
                      ×
                    </button>
                  </div>
                </div>
                {contact.company && (
                  <div className="contact-company">{contact.company}</div>
                )}
                {contact.title && (
                  <div className="contact-title">{contact.title}</div>
                )}
                {contact.email && (
                  <div className="contact-email">{contact.email}</div>
                )}
                {contact.lastContactDate && (
                  <div className="contact-last-contact">
                    Last contact: {new Date(contact.lastContactDate).toLocaleDateString()}
                  </div>
                )}
                <div className={`relationship-badge relationship-${contact.relationshipStrength}`}>
                  {contact.relationshipStrength}
                </div>
              </div>
            ))
          )}
        </div>

        {selectedContact && (
          <div className="contact-detail">
            <div className="detail-header">
              <div>
                <h2>{selectedContact.name}</h2>
                {selectedContact.company && (
                  <div className="detail-company">{selectedContact.company}</div>
                )}
              </div>
              <button onClick={() => setSelectedContact(null)} aria-label="Close">×</button>
            </div>

            <div className="detail-info">
              {selectedContact.title && (
                <div className="info-item">
                  <strong>Title:</strong> {selectedContact.title}
                </div>
              )}
              {selectedContact.email && (
                <div className="info-item">
                  <strong>Email:</strong>{" "}
                  <a href={`mailto:${selectedContact.email}`}>{selectedContact.email}</a>
                </div>
              )}
              {selectedContact.phone && (
                <div className="info-item">
                  <strong>Phone:</strong> {selectedContact.phone}
                </div>
              )}
              {selectedContact.linkedinUrl && (
                <div className="info-item">
                  <strong>LinkedIn:</strong>{" "}
                  <a href={selectedContact.linkedinUrl} target="_blank" rel="noopener noreferrer">
                    View Profile
                  </a>
                </div>
              )}
              {selectedContact.notes && (
                <div className="info-item">
                  <strong>Notes:</strong>
                  <p>{selectedContact.notes}</p>
                </div>
              )}
              {selectedContact.tags && (
                <div className="info-item">
                  <strong>Tags:</strong> {selectedContact.tags}
                </div>
              )}
            </div>

            <div className="interactions-section">
              <div className="section-header">
                <h3>Interactions</h3>
                <button onClick={handleNewInteraction} className="btn-secondary">
                  + Add Interaction
                </button>
              </div>
              {interactions.length === 0 ? (
                <div className="empty-state">
                  <p>No interactions yet. Add your first interaction.</p>
                </div>
              ) : (
                <div className="interactions-list">
                  {interactions.map((interaction) => (
                    <div key={interaction.id} className="interaction-card">
                      <div className="interaction-header">
                        <div>
                          <strong>{interaction.interactionType}</strong>
                          {interaction.subject && <span> - {interaction.subject}</span>}
                        </div>
                        <button
                          onClick={() => interaction.id && handleDeleteInteraction(interaction.id)}
                          className="icon-button delete"
                          aria-label="Delete interaction"
                        >
                          ×
                        </button>
                      </div>
                      <div className="interaction-date">
                        {new Date(interaction.interactionDate).toLocaleDateString()}
                      </div>
                      {interaction.notes && (
                        <div className="interaction-notes">{interaction.notes}</div>
                      )}
                      {interaction.outcome && (
                        <div className="interaction-outcome">
                          <strong>Outcome:</strong> {interaction.outcome}
                        </div>
                      )}
                      {interaction.followUpDate && (
                        <div className="interaction-followup">
                          Follow up: {new Date(interaction.followUpDate).toLocaleDateString()}
                        </div>
                      )}
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {showContactModal && (
        <ContactModal
          contact={editingContact}
          onClose={() => {
            setShowContactModal(false);
            setEditingContact(null);
          }}
          onSave={() => {
            setShowContactModal(false);
            setEditingContact(null);
            loadContacts();
          }}
        />
      )}

      {showInteractionModal && selectedContact && (
        <InteractionModal
          contactId={selectedContact.id!}
          onClose={() => setShowInteractionModal(false)}
          onSave={() => {
            setShowInteractionModal(false);
            if (selectedContact) {
              loadContactDetails(selectedContact);
            }
          }}
        />
      )}
    </div>
  );
}

function ContactModal({
  contact,
  onClose,
  onSave,
}: {
  contact: RecruiterContact | null;
  onClose: () => void;
  onSave: () => void;
}) {
  const [name, setName] = useState(contact?.name || "");
  const [email, setEmail] = useState(contact?.email || "");
  const [phone, setPhone] = useState(contact?.phone || "");
  const [linkedinUrl, setLinkedinUrl] = useState(contact?.linkedinUrl || "");
  const [company, setCompany] = useState(contact?.company || "");
  const [title, setTitle] = useState(contact?.title || "");
  const [notes, setNotes] = useState(contact?.notes || "");
  const [relationshipStrength, setRelationshipStrength] = useState(
    contact?.relationshipStrength || "neutral"
  );
  const [tags, setTags] = useState(contact?.tags || "");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!name.trim()) {
      showToast("Please enter a name", "error");
      return;
    }

    try {
      if (contact?.id) {
        await invoke("update_recruiter_contact", {
          contactId: contact.id,
          name: name || null,
          email: email || null,
          phone: phone || null,
          linkedinUrl: linkedinUrl || null,
          company: company || null,
          title: title || null,
          notes: notes || null,
          relationshipStrength: relationshipStrength || null,
          tags: tags || null,
        });
        showToast("Contact updated", "success");
      } else {
        await invoke("create_recruiter_contact", {
          name,
          email: email || null,
          phone: phone || null,
          linkedinUrl: linkedinUrl || null,
          company: company || null,
          title: title || null,
          notes: notes || null,
          relationshipStrength: relationshipStrength || null,
          tags: tags || null,
        });
        showToast("Contact created", "success");
      }
      onSave();
    } catch (err: any) {
      showToast(err?.message || "Failed to save contact", "error");
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>{contact ? "Edit Contact" : "New Contact"}</h3>
          <button onClick={onClose} aria-label="Close">×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Name *</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
          </div>
          <div className="form-group">
            <label>Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Phone</label>
            <input
              type="tel"
              value={phone}
              onChange={(e) => setPhone(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>LinkedIn URL</label>
            <input
              type="url"
              value={linkedinUrl}
              onChange={(e) => setLinkedinUrl(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Company</label>
            <input
              type="text"
              value={company}
              onChange={(e) => setCompany(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Title</label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Relationship Strength</label>
            <select
              value={relationshipStrength}
              onChange={(e) => setRelationshipStrength(e.target.value)}
            >
              <option value="strong">Strong</option>
              <option value="good">Good</option>
              <option value="neutral">Neutral</option>
              <option value="weak">Weak</option>
            </select>
          </div>
          <div className="form-group">
            <label>Tags (comma-separated)</label>
            <input
              type="text"
              value={tags}
              onChange={(e) => setTags(e.target.value)}
              placeholder="e.g., tech, startup, remote"
            />
          </div>
          <div className="form-group">
            <label>Notes</label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              rows={4}
            />
          </div>
          <div className="modal-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  );
}

function InteractionModal({
  contactId,
  onClose,
  onSave,
}: {
  contactId: number;
  onClose: () => void;
  onSave: () => void;
}) {
  const [interactionType, setInteractionType] = useState("email");
  const [interactionDate, setInteractionDate] = useState(
    new Date().toISOString().split("T")[0]
  );
  const [subject, setSubject] = useState("");
  const [notes, setNotes] = useState("");
  const [outcome, setOutcome] = useState("");
  const [followUpDate, setFollowUpDate] = useState("");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      await invoke("create_interaction", {
        contactId,
        interactionType,
        interactionDate,
        subject: subject || null,
        notes: notes || null,
        linkedApplicationId: null,
        linkedJobId: null,
        outcome: outcome || null,
        followUpDate: followUpDate || null,
      });
      showToast("Interaction created", "success");
      onSave();
    } catch (err: any) {
      showToast(err?.message || "Failed to create interaction", "error");
    }
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h3>New Interaction</h3>
          <button onClick={onClose} aria-label="Close">×</button>
        </div>
        <form onSubmit={handleSubmit}>
          <div className="form-group">
            <label>Type *</label>
            <select
              value={interactionType}
              onChange={(e) => setInteractionType(e.target.value)}
              required
            >
              <option value="email">Email</option>
              <option value="phone">Phone Call</option>
              <option value="meeting">Meeting</option>
              <option value="linkedin">LinkedIn</option>
              <option value="other">Other</option>
            </select>
          </div>
          <div className="form-group">
            <label>Date *</label>
            <input
              type="date"
              value={interactionDate}
              onChange={(e) => setInteractionDate(e.target.value)}
              required
            />
          </div>
          <div className="form-group">
            <label>Subject</label>
            <input
              type="text"
              value={subject}
              onChange={(e) => setSubject(e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Notes</label>
            <textarea
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              rows={4}
            />
          </div>
          <div className="form-group">
            <label>Outcome</label>
            <input
              type="text"
              value={outcome}
              onChange={(e) => setOutcome(e.target.value)}
              placeholder="e.g., Scheduled interview, Sent resume"
            />
          </div>
          <div className="form-group">
            <label>Follow-up Date</label>
            <input
              type="date"
              value={followUpDate}
              onChange={(e) => setFollowUpDate(e.target.value)}
            />
          </div>
          <div className="modal-actions">
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit" className="btn-primary">Save</button>
          </div>
        </form>
      </div>
    </div>
  );
}
