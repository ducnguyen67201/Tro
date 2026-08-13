import { useEffect, useState } from "react";
import type { AppSnapshot, CursorCompanionPhase } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";

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
  guiding: "Đang hướng dẫn…",
  failed: "Tro gặp sự cố",
};

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
    return (
      <div
        className={`cursor-following${phase === "acting" ? " is-acting" : ""}`}
        aria-hidden="true"
      >
        <div className="cursor-orb">
          <span>T</span>
          <i />
        </div>
      </div>
    );
  }

  const active = snapshot.assistant !== "idle";
  const detail =
    snapshot.transcript ??
    (snapshot.assistant === "idle"
      ? "Giữ ⌘ + ⌥ và nói để hỏi Tro."
      : snapshot.status_vi);

  const dismiss = () => {
    if (active) void desktop.stopAssistant("cursor_card");
    else void desktop.dismissCursorCompanion();
  };

  return (
    <section className="cursor-card" role="status" aria-live="polite">
      <div
        className={`cursor-card-orb state-${snapshot.assistant}`}
        aria-hidden="true"
      >
        <span>T</span>
      </div>
      <div className="cursor-card-copy">
        <strong>{stateLabels[snapshot.assistant]}</strong>
        <p>{detail}</p>
        <small>
          {active ? "Bạn có thể dừng bất cứ lúc nào" : "Tro đang chạy nền"}
        </small>
      </div>
      <button
        type="button"
        onClick={dismiss}
        aria-label={active ? "Dừng Tro" : "Đóng Tro"}
      >
        {active ? "Dừng" : "Đóng"}
      </button>
    </section>
  );
}
