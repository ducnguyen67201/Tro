import { useState } from "react";
import type { AppSettings } from "../../lib/contracts";
import { defaultAskShortcut, desktop } from "../../lib/tauri";
import { PrivacyPanel } from "./PrivacyPanel";
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
