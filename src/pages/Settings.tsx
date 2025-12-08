import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AiSettings, CloudProvider } from "../ai/types";
import { LoadingSkeleton } from "../components/LoadingSkeleton";
import { showToast } from "../components/Toast";
import "./Settings.css";

export default function Settings() {
  const [settings, setSettings] = useState<AiSettings | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadSettings();
  }, []);

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
    </div>
  );
}

