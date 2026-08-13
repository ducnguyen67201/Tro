import { useEffect, useState } from "react";
import type { LlmConfig } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";

const initial: LlmConfig = {
  backend_url: "http://127.0.0.1:8080",
  timeout_seconds: 25,
  device_authenticated: false,
};

export function LlmSettings() {
  const [config, setConfig] = useState(initial);
  const [message, setMessage] = useState("");

  useEffect(() => {
    let disposed = false;
    void desktop
      .llmConfig()
      .then((next) => {
        if (!disposed) setConfig(next);
      })
      .catch(() => {
        if (!disposed) setMessage("Chưa đọc được cấu hình máy chủ Tro.");
      });
    return () => {
      disposed = true;
    };
  }, []);

  return (
    <section className="settings-section llm-settings">
      <div className="settings-section-heading">
        <div>
          <h2>AI backend</h2>
          <p>
            Tro gửi yêu cầu đến máy chủ riêng. Provider và API key chỉ tồn tại ở
            backend.
          </p>
        </div>
        <span
          className={`llm-status ${config.device_authenticated ? "is-ready" : ""}`}
        >
          {config.device_authenticated
            ? "Thiết bị đã xác thực"
            : "Chưa có phiên thiết bị"}
        </span>
      </div>
      <div className="settings-row">
        <span>Máy chủ</span>
        <code>{config.backend_url}</code>
      </div>
      <div className="settings-row">
        <span>Giới hạn chờ</span>
        <span>{config.timeout_seconds} giây</span>
      </div>
      <div className="llm-settings-footer">
        <span role="status">{message}</span>
        <span>Model được quản lý an toàn ở backend.</span>
      </div>
    </section>
  );
}
