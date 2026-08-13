import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { AgentStatusPill } from "./features/agent/AgentStatusPill";
import { useAgentStore } from "./features/agent/agentStore";
import { AssistantBar } from "./features/assistant/AssistantBar";
import { TranscriptPanel } from "./features/assistant/TranscriptPanel";
import { useAssistantStore } from "./features/assistant/assistantStore";
import { Onboarding } from "./features/onboarding/Onboarding";
import { SettingsPage } from "./features/settings/SettingsPage";
import { desktop } from "./lib/tauri";

export default function App() {
  const { t } = useTranslation();
  const [onboarded, setOnboarded] = useState(
    () => localStorage.getItem("tro.onboarded") === "true",
  );
  const [settings, setSettings] = useState(false);
  const setSnapshot = useAssistantStore((state) => state.setSnapshot);
  const setConfirmation = useAgentStore((state) => state.setConfirmation);

  useEffect(() => {
    void desktop.snapshot().then(setSnapshot);
    void desktop.onSnapshot(setSnapshot);
    void desktop.onConfirmation(setConfirmation);
    void desktop.onGlobalShortcut((action) => {
      if (action === "ask") void desktop.startAssistant("command_option");
      if (action === "ask_release")
        void desktop.stopAssistant("command_option_released");
      if (action === "dictation") void desktop.startAssistant();
      if (action === "stop") void desktop.emergencyStop();
    });
    void desktop.onOpenSettings(() => {
      setSettings(true);
    });
  }, [setConfirmation, setSnapshot]);

  if (!onboarded)
    return (
      <Onboarding
        onComplete={() => {
          localStorage.setItem("tro.onboarded", "true");
          setOnboarded(true);
        }}
      />
    );
  if (settings)
    return (
      <SettingsPage
        onClose={() => {
          setSettings(false);
        }}
      />
    );

  return (
    <main className="app-shell">
      <header className="topbar">
        <div className="brand-lockup">
          <span className="brand-mark">T</span>
          <span>Tro</span>
          <small>beta</small>
        </div>
        <nav>
          <button
            onClick={() => {
              setSettings(true);
            }}
          >
            {t("settings")}
          </button>
          <span className="privacy-indicator">
            <i /> Riêng tư
          </span>
        </nav>
      </header>
      <section className="welcome-block">
        <span className="eyebrow">Trợ lý học tập trên máy tính</span>
        <h1>
          Chào bạn, <em>mình học gì hôm nay?</em>
        </h1>
        <p>
          Giữ <kbd>⌘</kbd>
          <kbd>⌥</kbd> rồi nói. Thả phím để dừng. Tro sẽ nhìn đúng màn hình hiện
          tại và hướng dẫn thay vì làm hộ.
        </p>
      </section>
      <AssistantBar />
      <TranscriptPanel />
      <section className="mode-grid">
        <article>
          <span className="mode-icon lilac">?</span>
          <h2>Hỏi về màn hình</h2>
          <p>Giải thích bài toán, đoạn văn hoặc giao diện đang mở.</p>
          <small>Giữ ⌘ ⌥</small>
        </article>
        <article>
          <span className="mode-icon mint">Aa</span>
          <h2>Đọc chính tả</h2>
          <p>Nói tiếng Việt, xem lại, rồi chèn văn bản đúng dấu.</p>
          <small>⌘ ⇧ D</small>
        </article>
        <article>
          <span className="mode-icon peach">↗</span>
          <h2>Hướng dẫn trực quan</h2>
          <p>Mũi tên và từng bước xuất hiện trên đúng vị trí cần làm.</p>
          <small>Tự động khi cần</small>
        </article>
      </section>
      <AgentStatusPill />
      <footer>
        <span>
          Tro không thay thế giảng viên và không hỗ trợ gian lận trong bài thi.
        </span>
        <button
          onClick={() => {
            void desktop.emergencyStop();
          }}
        >
          Dừng khẩn cấp · Esc
        </button>
      </footer>
    </main>
  );
}
