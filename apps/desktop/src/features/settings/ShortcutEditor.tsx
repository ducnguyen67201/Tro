import type { AppSettings } from "../../lib/contracts";

export function ShortcutEditor({
  settings,
  onChange,
}: {
  settings: AppSettings;
  onChange: (patch: Partial<AppSettings>) => void;
}) {
  return (
    <section className="settings-section">
      <h2>Phím tắt</h2>
      <div className="settings-row">
        <label htmlFor="ask-shortcut">Hỏi Tro</label>
        <input
          id="ask-shortcut"
          value={settings.ask_shortcut}
          readOnly={settings.ask_shortcut === "Command+Control"}
          aria-describedby={
            settings.ask_shortcut === "Command+Control"
              ? "ask-shortcut-help"
              : undefined
          }
          onChange={(event) => {
            onChange({ ask_shortcut: event.target.value });
          }}
        />
        {settings.ask_shortcut === "Command+Control" ? (
          <small id="ask-shortcut-help">
            Giữ ⌘ + ⌃ để nói, thả phím để dừng.
          </small>
        ) : null}
      </div>
      <div className="settings-row">
        <label htmlFor="dictation-shortcut">Đọc chính tả</label>
        <input
          id="dictation-shortcut"
          value={settings.dictation_shortcut}
          onChange={(event) => {
            onChange({ dictation_shortcut: event.target.value });
          }}
        />
      </div>
      <div className="settings-row">
        <label htmlFor="stop-shortcut">Dừng khẩn cấp</label>
        <input
          id="stop-shortcut"
          value={settings.stop_shortcut}
          readOnly
          aria-describedby="stop-shortcut-help"
        />
        <small id="stop-shortcut-help">
          Nhấn Esc bất kỳ lúc nào để dừng nghe, xử lý hoặc computer use.
        </small>
      </div>
    </section>
  );
}
