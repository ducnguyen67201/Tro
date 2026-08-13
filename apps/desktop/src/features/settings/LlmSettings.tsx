import { useEffect, useState } from "react";
import type { LlmConfig } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";

const initial: LlmConfig = {
  provider: "openrouter",
  base_url: "https://openrouter.ai/api/v1",
  model: "google/gemini-2.5-flash",
  timeout_seconds: 20,
  api_key_configured: false,
};

export function LlmSettings() {
  const [config, setConfig] = useState(initial);
  const [apiKey, setApiKey] = useState("");
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    let disposed = false;
    void desktop
      .llmConfig()
      .then((next) => {
        if (!disposed) setConfig(next);
      })
      .catch(() => {
        if (!disposed) setMessage("Chưa đọc được cấu hình LLM.");
      });
    return () => {
      disposed = true;
    };
  }, []);

  const save = () => {
    setSaving(true);
    setMessage("");
    void desktop
      .updateLlmConfig({
        provider: config.provider,
        base_url: config.base_url,
        model: config.model,
        timeout_seconds: config.timeout_seconds,
        ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
      })
      .then((next) => {
        setConfig(next);
        setApiKey("");
        setMessage("Đã lưu. Key nằm trong macOS Keychain.");
      })
      .catch(() => {
        setMessage("Không thể lưu cấu hình. Hãy kiểm tra key và model.");
      })
      .finally(() => {
        setSaving(false);
      });
  };

  return (
    <section className="settings-section llm-settings">
      <div className="settings-section-heading">
        <div>
          <h2>LLM</h2>
          <p>OpenRouter tạm thời; lớp provider có thể thay thế độc lập.</p>
        </div>
        <span
          className={`llm-status ${config.api_key_configured ? "is-ready" : ""}`}
        >
          {config.api_key_configured ? "Đã có API key" : "Chưa có API key"}
        </span>
      </div>
      <div className="settings-row">
        <label htmlFor="llm-provider">Provider</label>
        <select
          id="llm-provider"
          value={config.provider}
          onChange={(event) => {
            setConfig({ ...config, provider: event.target.value });
          }}
        >
          <option value="openrouter">OpenRouter</option>
        </select>
      </div>
      <div className="settings-row">
        <label htmlFor="llm-model">Model</label>
        <input
          id="llm-model"
          value={config.model}
          spellCheck={false}
          onChange={(event) => {
            setConfig({ ...config, model: event.target.value });
          }}
        />
      </div>
      <div className="settings-row">
        <label htmlFor="llm-base-url">Base URL</label>
        <input
          id="llm-base-url"
          value={config.base_url}
          inputMode="url"
          spellCheck={false}
          onChange={(event) => {
            setConfig({ ...config, base_url: event.target.value });
          }}
        />
      </div>
      <div className="settings-row">
        <label htmlFor="llm-timeout">Thinking timeout</label>
        <div className="timeout-input">
          <input
            id="llm-timeout"
            type="number"
            min={5}
            max={60}
            value={config.timeout_seconds}
            onChange={(event) => {
              setConfig({
                ...config,
                timeout_seconds: Number(event.target.value),
              });
            }}
          />
          <span>giây</span>
        </div>
      </div>
      <div className="settings-row">
        <label htmlFor="llm-api-key">OpenRouter API key</label>
        <input
          id="llm-api-key"
          type="password"
          value={apiKey}
          autoComplete="off"
          placeholder={
            config.api_key_configured ? "Để trống để giữ key cũ" : "sk-or-v1-…"
          }
          onChange={(event) => {
            setApiKey(event.target.value);
          }}
        />
      </div>
      <div className="llm-settings-footer">
        <span role="status">{message}</span>
        <button
          type="button"
          className="button primary"
          disabled={saving}
          onClick={save}
        >
          {saving ? "Đang lưu…" : "Lưu LLM"}
        </button>
      </div>
    </section>
  );
}
