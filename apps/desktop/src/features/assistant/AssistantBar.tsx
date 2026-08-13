import { useTranslation } from "react-i18next";
import { desktop } from "../../lib/tauri";
import { useAssistantStore } from "./assistantStore";

const stateLabels: Record<string, string> = {
  idle: "Sẵn sàng",
  capturing: "Đang nhìn màn hình…",
  listening: "Đang nghe…",
  thinking: "Đang suy nghĩ…",
  speaking: "Đang trả lời…",
  guiding: "Đang hướng dẫn…",
  failed: "Có lỗi",
};

export function AssistantBar() {
  const { t } = useTranslation();
  const assistant = useAssistantStore((state) => state.assistant);
  const active = assistant !== "idle" && assistant !== "failed";

  return (
    <section
      className={`assistant-bar ${active ? "is-active" : ""}`}
      aria-label="Trợ lý Tro"
    >
      <div className="orb" aria-hidden="true">
        <span />
      </div>
      <div className="assistant-copy">
        <strong>{stateLabels[assistant]}</strong>
        <span>{active ? "Nhấn Esc bất cứ lúc nào để dừng" : t("ready")}</span>
      </div>
      <button
        className={active ? "button danger" : "button primary"}
        onClick={() => {
          void (active ? desktop.stopAssistant() : desktop.startAssistant());
        }}
      >
        {active ? t("stop") : t("ask")}
      </button>
    </section>
  );
}
