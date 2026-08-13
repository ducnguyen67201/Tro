import { useEffect, useState } from "react";
import type { AppSnapshot, CursorCompanionPhase } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";

const companionImage = "/tro-companion-v1.png";

const initialSnapshot: AppSnapshot = {
  assistant: "idle",
  agent: "idle",
  transcript: null,
  status_vi: "Sẵn sàng",
  capture_active: false,
};

const stateLabels: Record<AppSnapshot["assistant"], string> = {
  idle: "Sẵn sàng",
  capturing: "Đang nhìn màn hình…",
  listening: "Đang nghe…",
  thinking: "Đang suy nghĩ…",
  speaking: "Đang trả lời…",
  guiding: "Tro trả lời",
  failed: "Tro gặp sự cố",
};

type CursorFeedback =
  | "idle"
  | "listening"
  | "processing"
  | "responding"
  | "failed";

const cursorFeedback: Record<AppSnapshot["assistant"], CursorFeedback> = {
  idle: "idle",
  capturing: "listening",
  listening: "listening",
  thinking: "processing",
  speaking: "responding",
  guiding: "responding",
  failed: "failed",
};

function feedbackFor(snapshot: AppSnapshot): CursorFeedback {
  if (snapshot.agent === "failed") return "failed";
  if (snapshot.agent === "completed") return "responding";
  if (
    snapshot.agent === "planning" ||
    snapshot.agent === "awaiting_confirmation" ||
    snapshot.agent === "executing" ||
    snapshot.agent === "observing"
  ) {
    return "processing";
  }
  return cursorFeedback[snapshot.assistant];
}

export function CursorAssistant() {
  const [phase, setPhase] = useState<CursorCompanionPhase>("following");
  const [snapshot, setSnapshot] = useState(initialSnapshot);

  useEffect(() => {
    let disposed = false;
    let phaseSeen = false;
    let snapshotSeen = false;
    const unlisteners: Array<() => void> = [];
    const own = (registration: Promise<() => void>) => {
      void registration
        .then((unlisten) => {
          if (disposed) unlisten();
          else unlisteners.push(unlisten);
        })
        .catch(() => undefined);
    };

    own(
      desktop.onCursorCompanion((next) => {
        phaseSeen = true;
        setPhase(next.phase);
      }),
    );
    own(
      desktop.onSnapshot((next) => {
        snapshotSeen = true;
        setSnapshot(next);
      }),
    );
    void desktop
      .cursorCompanionSnapshot()
      .then((next) => {
        if (!disposed && !phaseSeen) setPhase(next.phase);
      })
      .catch(() => undefined);
    void desktop
      .snapshot()
      .then((next) => {
        if (!disposed && !snapshotSeen) setSnapshot(next);
      })
      .catch(() => undefined);

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) unlisten();
    };
  }, []);

  if (phase === "hidden") return null;

  if (phase === "following" || phase === "acting") {
    const feedback = feedbackFor(snapshot);
    return (
      <div
        className={`cursor-following is-${feedback}${phase === "acting" ? " is-acting" : ""}`}
        data-assistant-state={snapshot.assistant}
        aria-hidden="true"
      >
        <div className="cursor-orb">
          <span className="cursor-orb-halo" />
          <img src={companionImage} alt="" draggable={false} />
          <span className="cursor-orb-listening">
            <b />
            <b />
            <b />
          </span>
          <span className="cursor-orb-processing" />
          <span className="cursor-orb-response">
            <svg viewBox="0 0 16 16" focusable="false">
              <path d="m4.2 8.1 2.3 2.3 5.2-5.2" />
            </svg>
          </span>
        </div>
      </div>
    );
  }

  const active = ["capturing", "listening", "thinking", "speaking"].includes(
    snapshot.assistant,
  );
  const needsReset = snapshot.assistant !== "idle";
  const detail =
    snapshot.transcript ??
    (snapshot.assistant === "idle"
      ? "Giữ ⌘ + ⌥ và nói để hỏi Tro."
      : snapshot.status_vi);

  const dismiss = () => {
    if (needsReset) void desktop.stopAssistant("cursor_card");
    else void desktop.dismissCursorCompanion();
  };

  return (
    <section className="cursor-card" role="status" aria-live="polite">
      <div
        className={`cursor-card-orb state-${snapshot.assistant}`}
        aria-hidden="true"
      >
        <img src={companionImage} alt="" draggable={false} />
      </div>
      <div className="cursor-card-copy">
        <strong>{stateLabels[snapshot.assistant]}</strong>
        <p>{detail}</p>
        <small>
          {active
            ? "Bạn có thể dừng bất cứ lúc nào"
            : snapshot.assistant === "guiding"
              ? "Câu trả lời chỉ nằm trong phiên hiện tại"
              : "Tro đang chạy nền"}
        </small>
      </div>
      <button
        type="button"
        onClick={dismiss}
        aria-label={active ? "Dừng Tro" : "Đóng Tro"}
      >
        {active ? "Dừng" : "Xong"}
      </button>
    </section>
  );
}
