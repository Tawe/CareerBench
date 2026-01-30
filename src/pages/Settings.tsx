import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AiSettings, CloudProvider } from "../ai/types";
import type { EmailAccount, CacheStats } from "../commands/types";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import "./Settings.css";

export default function Settings() {
  const [settings, setSettings] = useState<AiSettings | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [emailAccounts, setEmailAccounts] = useState<EmailAccount[]>([]);
  const [showEmailForm, setShowEmailForm] = useState(false);
  const [emailFormData, setEmailFormData] = useState<Partial<EmailAccount>>({
    emailAddress: "",
    provider: "gmail",
    useSsl: true,
    isActive: true,
  });
  const [cacheStats, setCacheStats] = useState<CacheStats | null>(null);
  const [isLoadingCacheStats, setIsLoadingCacheStats] = useState(false);
  const [isDownloadingModel, setIsDownloadingModel] = useState(false);
  const [downloadProgress, setDownloadProgress] = useState<string | null>(null);
  const [customModelUrl, setCustomModelUrl] = useState<string>("");
  const [availableModelFiles, setAvailableModelFiles] = useState<string[]>([]);

  useEffect(() => {
    loadSettings();
    loadEmailAccounts();
    loadCacheStats();
    findModelFiles();
  }, []);

  async function findModelFiles() {
    try {
      const files = await invoke<string[]>("find_model_files");
      setAvailableModelFiles(files);
      console.log("Found model files:", files);
    } catch (err: any) {
      console.error("Failed to find model files:", err);
    }
  }

  async function useDetectedModel(modelPath: string) {
    if (!settings) return;
    
    try {
      const updatedSettings = { ...settings, localModelPath: modelPath };
      await invoke("save_ai_settings", { settings: updatedSettings });
      setSettings(updatedSettings);
      showToast("Model path set successfully!", "success");
    } catch (err: any) {
      showToast(`Failed to set model path: ${err.message || err}`, "error");
    }
  }

  async function loadSettings() {
    setIsLoading(true);
    setError(null);
    try {
      const result = await invoke<AiSettings>("get_ai_settings");
      setSettings(result);
    } catch (err: any) {
      setError(err?.message || "Failed to load settings");
    } finally {
      setIsLoading(false);
    }
  }

  async function handleSave() {
    if (!settings) return;
    
    setIsSaving(true);
    setError(null);
    setTestResult(null);
    
    try {
      await invoke("save_ai_settings", { settings });
      // Reload to ensure we have the latest
      await loadSettings();
    } catch (err: any) {
      setError(err?.message || "Failed to save settings");
    } finally {
      setIsSaving(false);
    }
  }

  async function handleTestConnection() {
    if (!settings) return;
    
    setTestResult(null);
    setError(null);
    
    try {
      const result = await invoke<string>("test_ai_connection");
      setTestResult({ success: true, message: result });
    } catch (err: any) {
      setTestResult({ success: false, message: err?.message || "Connection test failed" });
    }
  }

  async function loadEmailAccounts() {
    try {
      const accounts = await invoke<EmailAccount[]>("get_email_accounts");
      setEmailAccounts(accounts);
    } catch (err: any) {
      console.error("Failed to load email accounts:", err);
    }
  }

  async function loadCacheStats() {
    setIsLoadingCacheStats(true);
    try {
      const stats = await invoke<CacheStats>("get_cache_stats");
      setCacheStats(stats);
    } catch (err: any) {
      console.error("Failed to load cache stats:", err);
    } finally {
      setIsLoadingCacheStats(false);
    }
  }

  async function handleClearCacheByPurpose(purpose: string) {
    if (!confirm(`Are you sure you want to clear all cache entries for "${purpose}"?`)) return;
    try {
      const count = await invoke<number>("clear_cache_by_purpose", { purpose });
      showToast(`Cleared ${count} cache entries for ${purpose}`, "success");
      loadCacheStats();
    } catch (err: any) {
      showToast(err?.message || "Failed to clear cache", "error");
    }
  }

  async function handleClearAllCache() {
    if (!confirm("Are you sure you want to clear ALL cache entries? This cannot be undone.")) return;
    try {
      const count = await invoke<number>("clear_all_cache");
      showToast(`Cleared ${count} cache entries`, "success");
      loadCacheStats();
    } catch (err: any) {
      showToast(err?.message || "Failed to clear cache", "error");
    }
  }

  async function handleCleanupExpiredCache() {
    try {
      const count = await invoke<number>("cleanup_expired_cache");
      showToast(`Cleaned up ${count} expired cache entries`, "success");
      loadCacheStats();
    } catch (err: any) {
      showToast(err?.message || "Failed to cleanup cache", "error");
    }
  }

  async function handleEvictBySize(maxSizeMb: number) {
    if (!confirm(`Evict cache entries to stay under ${maxSizeMb}MB?`)) return;
    try {
      const count = await invoke<number>("evict_cache_by_size", { maxSizeMb });
      showToast(`Evicted ${count} cache entries`, "success");
      loadCacheStats();
    } catch (err: any) {
      showToast(err?.message || "Failed to evict cache", "error");
    }
  }

  async function handleEvictByCount(maxEntries: number) {
    if (!confirm(`Evict cache entries to stay under ${maxEntries} entries?`)) return;
    try {
      const count = await invoke<number>("evict_cache_by_count", { maxEntries });
      showToast(`Evicted ${count} cache entries`, "success");
      loadCacheStats();
    } catch (err: any) {
      showToast(err?.message || "Failed to evict cache", "error");
    }
  }

  function formatBytes(bytes: number): string {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + " " + sizes[i];
  }

  async function handleSaveEmailAccount() {
    try {
      const [imapServer, imapPort, useSsl] = getProviderSettings(
        emailFormData.provider || "gmail",
        emailFormData.emailAddress || ""
      );

      const account: EmailAccount = {
        ...emailFormData,
        emailAddress: emailFormData.emailAddress || "",
        provider: emailFormData.provider || "gmail",
        imapServer: emailFormData.imapServer || imapServer,
        imapPort: emailFormData.imapPort || imapPort,
        useSsl: emailFormData.useSsl ?? useSsl,
        isActive: emailFormData.isActive ?? true,
        createdAt: emailFormData.createdAt || new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      } as EmailAccount;

      await invoke<number>("save_email_account", { account });
      showToast("Email account saved successfully", "success");
      setShowEmailForm(false);
      setEmailFormData({
        emailAddress: "",
        provider: "gmail",
        useSsl: true,
        isActive: true,
      });
      loadEmailAccounts();
    } catch (err: any) {
      showToast(err?.message || "Failed to save email account", "error");
    }
  }

  async function handleDeleteEmailAccount(id: number) {
    if (!confirm("Are you sure you want to delete this email account?")) return;
    try {
      await invoke("delete_email_account", { accountId: id });
      showToast("Email account deleted", "success");
      loadEmailAccounts();
    } catch (err: any) {
      showToast(err?.message || "Failed to delete email account", "error");
    }
  }

  async function handleTestEmailConnection() {
    if (!emailFormData.emailAddress) {
      showToast("Please enter an email address", "error");
      return;
    }
    try {
      const result = await invoke<string>("test_email_connection", {
        email: emailFormData.emailAddress,
        password: "", // In a real implementation, this would come from secure storage
        provider: emailFormData.provider || "gmail",
      });
      showToast(result, "info");
    } catch (err: any) {
      showToast(err?.message || "Connection test failed", "error");
    }
  }

  async function handleDownloadModel() {
    console.log("handleDownloadModel called");
    
    // Require custom URL now since default URLs don't work
    if (!customModelUrl.trim()) {
      showToast("Please enter a model URL from Hugging Face", "error");
      setError("Please enter a model URL. Visit the Hugging Face model page and copy the direct download link for a GGUF file.");
      return;
    }
    
    setIsDownloadingModel(true);
    setDownloadProgress("Starting download... This may take several minutes (~2.3GB)");
    setError(null);
    
    try {
      const modelUrl = customModelUrl.trim();
      console.log("Calling download_model with URL:", modelUrl);
      const result = await invoke<string>("download_model", { modelUrl: modelUrl });
      console.log("Download result:", result);
      
      // The result is the file path
      if (result) {
        setDownloadProgress("Download complete!");
        
        // Always save the path, even if settings is null (it will be loaded first)
        try {
          // Load current settings first to preserve other settings
          const currentSettings = await invoke<AiSettings>("get_ai_settings");
          const updatedSettings = { ...currentSettings, localModelPath: result };
          
          // Save the updated settings
          await invoke("save_ai_settings", { settings: updatedSettings });
          
          // Update local state
          if (settings) {
            setSettings({ ...settings, localModelPath: result });
          } else {
            // Reload settings if they weren't loaded
            const reloaded = await invoke<AiSettings>("get_ai_settings");
            setSettings(reloaded);
          }
          
          showToast("Model downloaded successfully! Path has been saved to settings.", "success");
          
          // Refresh available model files
          await findModelFiles();
        } catch (saveErr: any) {
          console.error("Failed to save model path to settings:", saveErr);
          showToast(`Model downloaded but failed to save path: ${saveErr.message || saveErr}. Please manually set the path: ${result}`, "warning");
          
          // Still refresh available model files in case the file exists
          await findModelFiles();
        }
      }
    } catch (err: any) {
      const errorMsg = err?.message || "Failed to download model";
      setError(errorMsg);
      showToast(errorMsg, "error");
      setDownloadProgress(null);
    } finally {
      setIsDownloadingModel(false);
      // Clear progress message after a delay
      setTimeout(() => setDownloadProgress(null), 5000);
    }
  }

  function getProviderSettings(provider: string, email: string): [string, number, boolean] {
    switch (provider.toLowerCase()) {
      case "gmail":
        return ["imap.gmail.com", 993, true];
      case "outlook":
      case "hotmail":
      case "live":
        return ["outlook.office365.com", 993, true];
      case "yahoo":
        return ["imap.mail.yahoo.com", 993, true];
      case "icloud":
        return ["imap.mail.me.com", 993, true];
      default:
        const domain = email.split("@")[1] || "";
        return [`imap.${domain}`, 993, true];
    }
  }

  if (isLoading) {
    return (
      <div className="settings-page">
        <div className="settings-header">
          <LoadingSkeleton width="200px" height="2rem" />
        </div>
        <div className="settings-content">
          <LoadingSkeleton variant="card" />
        </div>
      </div>
    );
  }

  if (!settings) {
    return (
      <div className="settings-page">
        <div className="error">Failed to load settings</div>
      </div>
    );
  }

  return (
    <div className="settings-page">
      <div className="settings-header">
        <h1>AI Provider Settings</h1>
        <p className="settings-subtitle">
          Configure how CareerBench uses AI for resume suggestions, cover letters, and skill analysis.
        </p>
      </div>

      {error && (
        <div className="error-banner">
          {error}
        </div>
      )}

      {testResult && (
        <div className={`test-result ${testResult.success ? "success" : "error"}`}>
          {testResult.message}
        </div>
      )}

      <div className="settings-content">
        <div className="settings-section">
          <h2>AI Mode</h2>
          <div className="form-group">
            <label>Select AI Provider Mode</label>
            <select
              value={settings.mode}
              onChange={(e) =>
                setSettings({ ...settings, mode: e.target.value as AiSettings["mode"] })
              }
            >
              <option value="local">Local (Privacy-friendly, Offline)</option>
              <option value="cloud">Cloud (Requires API Key)</option>
              <option value="hybrid">Hybrid (Local + Cloud)</option>
            </select>
            <p className="form-help">
              {settings.mode === "local" && "Uses a local GGUF model file. No data leaves your device. Requires downloading a model file."}
              {settings.mode === "cloud" && "Uses cloud-based AI services. Requires an API key."}
              {settings.mode === "hybrid" && "Uses cloud if API key is configured, otherwise falls back to local model if configured."}
            </p>
          </div>
        </div>

        {settings.mode === "local" && (
          <div className="settings-section">
            <h2>Local Model Configuration</h2>
            
            <div className="form-group">
              <label>Model Path</label>
              <input
                type="text"
                value={settings.localModelPath || ""}
                onChange={(e) =>
                  setSettings({ ...settings, localModelPath: e.target.value })
                }
                placeholder="/path/to/model.gguf"
              />
              <p className="form-help">
                Path to your GGUF model file. Recommended: Download Phi-3-mini GGUF from{" "}
                <a href="https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf" target="_blank" rel="noopener noreferrer">
                  Hugging Face
                </a>
              </p>
              {availableModelFiles.length > 0 && !settings.localModelPath && (
                <div style={{ 
                  marginTop: "0.75rem",
                  padding: "0.75rem",
                  backgroundColor: "#f0fdf4",
                  border: "1px solid #86efac",
                  borderRadius: "0.375rem"
                }}>
                  <p style={{ margin: "0 0 0.5rem 0", fontSize: "0.875rem", color: "#166534", fontWeight: "500" }}>
                    ✓ Found {availableModelFiles.length} model file(s):
                  </p>
                  {availableModelFiles.map((filePath, idx) => (
                    <div key={idx} style={{ 
                      display: "flex", 
                      alignItems: "center", 
                      gap: "0.5rem",
                      marginBottom: "0.5rem"
                    }}>
                      <span style={{ fontSize: "0.75rem", color: "#166534", flex: 1, wordBreak: "break-all" }}>
                        {filePath.split('/').pop()}
                      </span>
                      <button
                        onClick={() => useDetectedModel(filePath)}
                        className="btn-primary"
                        style={{ fontSize: "0.75rem", padding: "0.25rem 0.5rem" }}
                        type="button"
                      >
                        Use This
                      </button>
                    </div>
                  ))}
                </div>
              )}
              <div style={{ marginTop: "0.75rem" }}>
                <div style={{ 
                  padding: "1rem", 
                  backgroundColor: "#f0f9ff", 
                  border: "1px solid #bae6fd", 
                  borderRadius: "0.375rem",
                  marginBottom: "1rem"
                }}>
                  <p style={{ margin: "0 0 0.75rem 0", fontSize: "0.875rem", color: "#1e40af", fontWeight: "500" }}>
                    📥 How to Download a Model:
                  </p>
                  <ol style={{ margin: "0", paddingLeft: "1.25rem", fontSize: "0.875rem", color: "#1e40af" }}>
                    <li>Visit <a href="https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/tree/main" target="_blank" rel="noopener noreferrer" style={{ color: "#2563eb", textDecoration: "underline" }}>the Hugging Face model page</a></li>
                    <li>Find a GGUF file (any file ending in <code>.gguf</code>)</li>
                    <li>Right-click the file → "Copy link address" (should look like: <code>https://huggingface.co/.../resolve/main/...gguf</code>)</li>
                    <li>Paste the URL below and click Download</li>
                  </ol>
                </div>
                <div style={{ display: "flex", gap: "0.5rem", alignItems: "center", flexWrap: "wrap", marginBottom: "0.75rem" }}>
                  <input
                    type="text"
                    value={customModelUrl}
                    onChange={(e) => setCustomModelUrl(e.target.value)}
                    placeholder="https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/...gguf"
                    style={{ 
                      flex: "1",
                      minWidth: "300px",
                      padding: "0.5rem 0.75rem",
                      border: "1px solid #e5e7eb",
                      borderRadius: "0.375rem",
                      fontSize: "0.875rem"
                    }}
                  />
                  <button
                    onClick={(e) => {
                      e.preventDefault();
                      console.log("Download button clicked");
                      handleDownloadModel();
                    }}
                    className="btn-primary"
                    disabled={isDownloadingModel || !customModelUrl.trim()}
                    type="button"
                    style={{ fontSize: "0.875rem" }}
                  >
                    {isDownloadingModel ? "Downloading..." : "Download Model"}
                  </button>
                </div>
                {downloadProgress && (
                  <div style={{ fontSize: "0.875rem", color: "#6b7280", marginTop: "0.5rem" }}>
                    {downloadProgress}
                  </div>
                )}
                <p className="form-help" style={{ marginTop: "0.5rem", fontSize: "0.75rem" }}>
                  <strong>Tip:</strong> The URL must be a direct download link (containing <code>/resolve/main/</code>), not a page link. 
                  Recommended files: <code>q4_k_m.gguf</code> (~2.3GB) or <code>q4_0.gguf</code> (~2.2GB) for good balance of quality and size.
                </p>
              </div>
            </div>
          </div>
        )}

        {settings.mode === "cloud" && (
          <div className="settings-section">
            <h2>Cloud Provider Configuration</h2>
            
            <div className="form-group">
              <label>Cloud Provider</label>
              <select
                value={settings.cloudProvider || "openai"}
                onChange={(e) =>
                  setSettings({
                    ...settings,
                    cloudProvider: e.target.value as CloudProvider,
                  })
                }
              >
                <option value="openai">OpenAI</option>
                <option value="anthropic">Anthropic</option>
              </select>
            </div>

            <div className="form-group">
              <label>API Key</label>
              <input
                type="password"
                value={settings.apiKey || ""}
                onChange={(e) =>
                  setSettings({ ...settings, apiKey: e.target.value })
                }
                placeholder="Enter your API key"
              />
              <p className="form-help">
                Your API key is stored locally and never shared.{" "}
                {settings.cloudProvider === "openai" ? (
                  <>Get your OpenAI key at{" "}
                    <a href="https://platform.openai.com/api-keys" target="_blank" rel="noopener noreferrer">
                      platform.openai.com
                    </a>
                  </>
                ) : (
                  <>Get your Anthropic key at{" "}
                    <a href="https://console.anthropic.com/" target="_blank" rel="noopener noreferrer">
                      console.anthropic.com
                    </a>
                  </>
                )}
              </p>
            </div>

            <div className="form-group">
              <label>Model Name</label>
              <input
                type="text"
                value={settings.modelName || (settings.cloudProvider === "anthropic" ? "claude-3-5-sonnet-20241022" : "gpt-4o-mini")}
                onChange={(e) =>
                  setSettings({ ...settings, modelName: e.target.value })
                }
                placeholder={settings.cloudProvider === "anthropic" ? "e.g., claude-3-5-sonnet-20241022" : "e.g., gpt-4o-mini, gpt-4"}
              />
              <p className="form-help">
                {settings.cloudProvider === "openai" ? (
                  <>Recommended: gpt-4o-mini, gpt-4o, or gpt-4-turbo</>
                ) : (
                  <>Recommended: claude-3-5-sonnet-20241022, claude-3-opus-20240229, or claude-3-haiku-20240307</>
                )}
              </p>
            </div>

            <div className="form-actions">
              <button
                onClick={handleTestConnection}
                className="btn-secondary"
                type="button"
              >
                Test Connection
              </button>
            </div>
          </div>
        )}

        <div className="settings-actions">
          <button
            onClick={handleSave}
            className="btn-primary"
            disabled={isSaving}
          >
            {isSaving ? "Saving..." : "Save Settings"}
          </button>
          <button
            onClick={loadSettings}
            className="btn-secondary"
            disabled={isSaving}
            type="button"
          >
            Cancel
          </button>
        </div>
      </div>

      {/* Email Integration Section */}
      <div className="settings-content" style={{ marginTop: "3rem" }}>
        <div className="settings-header">
          <h1>Email Integration</h1>
          <p className="settings-subtitle">
            Connect your email account to automatically import application events and track email threads.
          </p>
        </div>

        <div className="settings-section">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1rem" }}>
            <h2>Email Accounts</h2>
            <button
              onClick={() => setShowEmailForm(!showEmailForm)}
              className="btn-secondary"
              type="button"
            >
              {showEmailForm ? "Cancel" : "+ Add Email Account"}
            </button>
          </div>

          {showEmailForm && (
            <div className="email-form" style={{ padding: "1.5rem", border: "1px solid #e5e7eb", borderRadius: "0.5rem", marginBottom: "1.5rem", backgroundColor: "#f9fafb" }}>
              <h3>Add Email Account</h3>
              <div className="form-group">
                <label>Email Address</label>
                <input
                  type="email"
                  value={emailFormData.emailAddress || ""}
                  onChange={(e) => setEmailFormData({ ...emailFormData, emailAddress: e.target.value })}
                  placeholder="your.email@example.com"
                />
              </div>
              <div className="form-group">
                <label>Provider</label>
                <select
                  value={emailFormData.provider || "gmail"}
                  onChange={(e) => {
                    const provider = e.target.value;
                    const [imapServer, imapPort, useSsl] = getProviderSettings(provider, emailFormData.emailAddress || "");
                    setEmailFormData({
                      ...emailFormData,
                      provider,
                      imapServer,
                      imapPort,
                      useSsl,
                    });
                  }}
                >
                  <option value="gmail">Gmail</option>
                  <option value="outlook">Outlook / Office 365</option>
                  <option value="yahoo">Yahoo</option>
                  <option value="icloud">iCloud</option>
                  <option value="other">Other</option>
                </select>
              </div>
              {emailFormData.provider === "other" && (
                <>
                  <div className="form-group">
                    <label>IMAP Server</label>
                    <input
                      type="text"
                      value={emailFormData.imapServer || ""}
                      onChange={(e) => setEmailFormData({ ...emailFormData, imapServer: e.target.value })}
                      placeholder="imap.example.com"
                    />
                  </div>
                  <div className="form-group">
                    <label>IMAP Port</label>
                    <input
                      type="number"
                      value={emailFormData.imapPort || 993}
                      onChange={(e) => setEmailFormData({ ...emailFormData, imapPort: parseInt(e.target.value) || 993 })}
                    />
                  </div>
                </>
              )}
              <div className="form-group">
                <label>
                  <input
                    type="checkbox"
                    checked={emailFormData.useSsl ?? true}
                    onChange={(e) => setEmailFormData({ ...emailFormData, useSsl: e.target.checked })}
                  />
                  Use SSL/TLS
                </label>
              </div>
              <div className="form-actions">
                <button
                  onClick={handleTestEmailConnection}
                  className="btn-secondary"
                  type="button"
                >
                  Test Connection
                </button>
                <button
                  onClick={handleSaveEmailAccount}
                  className="btn-primary"
                  type="button"
                >
                  Save Account
                </button>
              </div>
              <p className="form-help" style={{ marginTop: "1rem", fontSize: "0.875rem", color: "#6b7280" }}>
                <strong>Note:</strong> Full email sync requires IMAP access. For Gmail, you'll need to enable "Less secure app access" or use an App Password. 
                Email sync functionality is currently in development.
              </p>
            </div>
          )}

          {emailAccounts.length === 0 ? (
            <p style={{ color: "#6b7280", padding: "2rem", textAlign: "center" }}>
              No email accounts configured. Add an account to enable email integration.
            </p>
          ) : (
            <div className="email-accounts-list">
              {emailAccounts.map((account) => (
                <div key={account.id} className="email-account-card" style={{ padding: "1rem", border: "1px solid #e5e7eb", borderRadius: "0.5rem", marginBottom: "0.75rem", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                  <div>
                    <div style={{ fontWeight: "600", marginBottom: "0.25rem" }}>{account.emailAddress}</div>
                    <div style={{ fontSize: "0.875rem", color: "#6b7280" }}>
                      {account.provider} • {account.isActive ? "Active" : "Inactive"}
                      {account.lastSyncAt && ` • Last sync: ${new Date(account.lastSyncAt).toLocaleDateString()}`}
                    </div>
                  </div>
                  <div style={{ display: "flex", gap: "0.5rem" }}>
                    <button
                      onClick={() => invoke("sync_email_account", { accountId: account.id })}
                      className="btn-secondary"
                      type="button"
                      style={{ fontSize: "0.875rem", padding: "0.5rem 1rem" }}
                    >
                      Sync
                    </button>
                    <button
                      onClick={() => account.id && handleDeleteEmailAccount(account.id)}
                      className="btn-secondary"
                      type="button"
                      style={{ fontSize: "0.875rem", padding: "0.5rem 1rem", color: "#ef4444" }}
                    >
                      Delete
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Cache Management Section */}
        <div className="settings-section">
          <h2>AI Cache Management</h2>
          <p style={{ color: "#6b7280", fontSize: "0.875rem", marginBottom: "1.5rem" }}>
            Manage AI response cache to control storage usage and clear outdated entries.
          </p>

          {isLoadingCacheStats ? (
            <LoadingSkeleton variant="card" width="100%" height="200px" />
          ) : cacheStats ? (
            <div className="cache-stats">
              <div className="cache-stats-grid">
                <div className="cache-stat-card">
                  <div className="cache-stat-label">Total Entries</div>
                  <div className="cache-stat-value">{(cacheStats.totalEntries || 0).toLocaleString()}</div>
                </div>
                <div className="cache-stat-card">
                  <div className="cache-stat-label">Total Size</div>
                  <div className="cache-stat-value">{formatBytes(cacheStats.totalSizeBytes || 0)}</div>
                </div>
                <div className="cache-stat-card">
                  <div className="cache-stat-label">Expired Entries</div>
                  <div className="cache-stat-value">{(cacheStats.expiredEntries || 0).toLocaleString()}</div>
                </div>
              </div>

              {cacheStats.entriesByPurpose && Object.keys(cacheStats.entriesByPurpose).length > 0 && (
                <div className="cache-by-purpose">
                  <h3 style={{ fontSize: "0.875rem", fontWeight: "500", marginBottom: "0.75rem", color: "var(--cb-text)" }}>
                    Entries by Purpose
                  </h3>
                  <div className="cache-purpose-list">
                    {Object.entries(cacheStats.entriesByPurpose || {})
                      .sort((a, b) => b[1] - a[1])
                      .map(([purpose, count]) => (
                        <div key={purpose} className="cache-purpose-item">
                          <span className="cache-purpose-name">{purpose}</span>
                          <div style={{ display: "flex", alignItems: "center", gap: "0.5rem" }}>
                            <span className="cache-purpose-count">{count.toLocaleString()}</span>
                            <button
                              onClick={() => handleClearCacheByPurpose(purpose)}
                              className="btn-secondary"
                              style={{ fontSize: "0.75rem", padding: "0.25rem 0.5rem" }}
                            >
                              Clear
                            </button>
                          </div>
                        </div>
                      ))}
                  </div>
                </div>
              )}

              {cacheStats.oldestEntry && (
                <div style={{ fontSize: "0.875rem", color: "#6b7280", marginTop: "1rem" }}>
                  <strong>Oldest entry:</strong> {new Date(cacheStats.oldestEntry).toLocaleDateString()}
                  {cacheStats.newestEntry && (
                    <> • <strong>Newest entry:</strong> {new Date(cacheStats.newestEntry).toLocaleDateString()}</>
                  )}
                </div>
              )}

              <div className="cache-actions" style={{ marginTop: "1.5rem", display: "flex", flexWrap: "wrap", gap: "0.75rem" }}>
                <button
                  onClick={handleCleanupExpiredCache}
                  className="btn-secondary"
                  disabled={(cacheStats.expiredEntries || 0) === 0}
                >
                  Cleanup Expired ({cacheStats.expiredEntries || 0})
                </button>
                <button
                  onClick={() => handleEvictBySize(100)}
                  className="btn-secondary"
                >
                  Evict to 100MB
                </button>
                <button
                  onClick={() => handleEvictBySize(500)}
                  className="btn-secondary"
                >
                  Evict to 500MB
                </button>
                <button
                  onClick={() => handleEvictByCount(1000)}
                  className="btn-secondary"
                >
                  Evict to 1000 entries
                </button>
                <button
                  onClick={handleClearAllCache}
                  className="btn-secondary"
                  style={{ color: "#ef4444", borderColor: "#ef4444" }}
                >
                  Clear All Cache
                </button>
                <button
                  onClick={loadCacheStats}
                  className="btn-secondary"
                >
                  Refresh Stats
                </button>
              </div>
            </div>
          ) : (
            <p style={{ color: "#6b7280", padding: "2rem", textAlign: "center" }}>
              Failed to load cache statistics.
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

