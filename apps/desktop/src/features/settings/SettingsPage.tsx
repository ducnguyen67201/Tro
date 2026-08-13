import { useCallback, useEffect, useState } from "react";
import type { AppSettings, PermissionStatus } from "../../lib/contracts";
import { PermissionCard } from "../onboarding/PermissionCard";
import { defaultAskShortcut, desktop } from "../../lib/tauri";
import { PrivacyPanel } from "./PrivacyPanel";
import { LlmSettings } from "./LlmSettings";
import { ShortcutEditor } from "./ShortcutEditor";

const initial: AppSettings = {
  locale: "vi",
  ask_shortcut: defaultAskShortcut(),
  dictation_shortcut: "CommandOrControl+Shift+D",
  stop_shortcut: "Escape",
  reduced_motion: false,
  dictation_preview: true,
  optional_telemetry: false,
};

export function SettingsPage({ onClose }: { onClose: () => void }) {
  const [settings, setSettings] = useState(initial);
  const [inputPermission, setInputPermission] =
    useState<PermissionStatus>("not_determined");
  const [requestingInput, setRequestingInput] = useState(false);
  const usesCommandOption = initial.ask_shortcut === "Command+Option";

  const refreshPermission = useCallback(() => {
    if (!usesCommandOption) return;
    void desktop
      .permissions()
      .then((snapshot) => {
        setInputPermission(snapshot.input_control);
      })
      .catch(() => undefined);
  }, [usesCommandOption]);

  useEffect(() => {
    refreshPermission();
    window.addEventListener("focus", refreshPermission);
    return () => {
      window.removeEventListener("focus", refreshPermission);
    };
  }, [refreshPermission]);

  const requestInputPermission = () => {
    if (inputPermission === "restart_required") {
      void desktop.restart();
      return;
    }
    setRequestingInput(true);
    void desktop
      .requestPermission("input_control")
      .then((snapshot) => {
        setInputPermission(snapshot.input_control);
      })
      .catch(refreshPermission)
      .finally(() => {
        setRequestingInput(false);
      });
  };

  const update = (patch: Partial<AppSettings>) => {
    const next = { ...settings, ...patch };
    setSettings(next);
    void desktop.updateSettings(patch);
  };
  return (
    <main className="settings-page">
      <div className="settings-header">
        <div>
          <span className="eyebrow">Tro</span>
          <h1>Cài đặt</h1>
        </div>
        <button className="button quiet" onClick={onClose}>
          Đóng
        </button>
      </div>
      <ShortcutEditor settings={settings} onChange={update} />
      {usesCommandOption ? (
        <section className="settings-section">
          <h2>Quyền phím tắt macOS</h2>
          <PermissionCard
            icon="⌘"
            title="Phím tắt Command + Option"
            detail="Cho phép Tro nhận hai phím này khi bạn đang dùng ứng dụng khác"
            status={inputPermission}
            requesting={requestingInput}
            onRequest={requestInputPermission}
          />
        </section>
      ) : null}
      <LlmSettings />
      <section className="settings-section">
        <h2>Trải nghiệm</h2>
        <label className="toggle-row">
          <span>Luôn xem trước dictation</span>
          <input
            type="checkbox"
            checked={settings.dictation_preview}
            onChange={(event) => {
              update({ dictation_preview: event.target.checked });
            }}
          />
        </label>
        <label className="toggle-row">
          <span>Giảm chuyển động</span>
          <input
            type="checkbox"
            checked={settings.reduced_motion}
            onChange={(event) => {
              update({ reduced_motion: event.target.checked });
            }}
          />
        </label>
        <label className="toggle-row">
          <span>Gửi telemetry không chứa nội dung</span>
          <input
            type="checkbox"
            checked={settings.optional_telemetry}
            onChange={(event) => {
              update({ optional_telemetry: event.target.checked });
            }}
          />
        </label>
      </section>
      <PrivacyPanel />
    </main>
  );
}
