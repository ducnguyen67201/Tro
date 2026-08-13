import { useEffect, useState } from "react";
import { useAgentStore } from "./features/agent/agentStore";
import { useAssistantStore } from "./features/assistant/assistantStore";
import { Onboarding } from "./features/onboarding/Onboarding";
import { SettingsPage } from "./features/settings/SettingsPage";
import { desktop } from "./lib/tauri";

const MAX_HOLD_TO_TALK_MS = 30_000;

export default function App() {
  const [onboarded, setOnboarded] = useState(
    () => localStorage.getItem("tro.onboarded") === "true",
  );
  const [settings, setSettings] = useState(false);
  const setSnapshot = useAssistantStore((state) => state.setSnapshot);
  const setConfirmation = useAgentStore((state) => state.setConfirmation);

  useEffect(() => {
    let disposed = false;
    let pendingAsk:
      | {
          start: Promise<void>;
          timeout: ReturnType<typeof window.setTimeout>;
          finished: boolean;
        }
      | undefined;
    const unlisteners: Array<() => void> = [];
    const own = (registration: Promise<() => void>) => {
      void registration
        .then((unlisten) => {
          if (disposed) unlisten();
          else unlisteners.push(unlisten);
        })
        .catch(() => undefined);
    };

    const finishPendingAsk = (reason: string) => {
      const current = pendingAsk;
      if (!current || current.finished) return;

      current.finished = true;
      window.clearTimeout(current.timeout);
      pendingAsk = undefined;
      void current.start.then(() =>
        desktop.finishAssistant(reason).catch(() => undefined),
      );
    };

    void desktop
      .snapshot()
      .then(setSnapshot)
      .catch(() => undefined);
    own(desktop.onSnapshot(setSnapshot));
    own(desktop.onConfirmation(setConfirmation));
    own(
      desktop.onGlobalShortcut((action) => {
        if (action === "ask") {
          if (pendingAsk) return;

          const current = {
            start: desktop
              .startAssistant("command_option")
              .catch(() => undefined),
            timeout: 0 as ReturnType<typeof window.setTimeout>,
            finished: false,
          };
          current.timeout = window.setTimeout(() => {
            if (pendingAsk === current) {
              finishPendingAsk("command_option_timeout");
            }
          }, MAX_HOLD_TO_TALK_MS);
          pendingAsk = current;
        }
        if (action === "ask_release") {
          finishPendingAsk("command_option_released");
        }
        if (action === "dictation") void desktop.startAssistant();
        if (action === "stop") void desktop.emergencyStop();
      }),
    );
    own(
      desktop.onOpenSettings(() => {
        setSettings(true);
      }),
    );

    return () => {
      disposed = true;
      if (pendingAsk) window.clearTimeout(pendingAsk.timeout);
      for (const unlisten of unlisteners) unlisten();
    };
  }, [setConfirmation, setSnapshot]);

  useEffect(() => {
    let disposed = false;
    if (!onboarded) {
      void desktop.showMainWindow();
      return () => {
        disposed = true;
      };
    }

    void desktop.followCursorCompanion();
    void desktop
      .permissions()
      .then((permissions) => {
        if (disposed) return;
        if (permissions.input_control === "granted") {
          void desktop.hideMainWindow();
        } else {
          setSettings(true);
          void desktop.showMainWindow();
        }
      })
      .catch(() => {
        if (!disposed) void desktop.hideMainWindow();
      });
    return () => {
      disposed = true;
    };
  }, [onboarded]);

  if (!onboarded)
    return (
      <Onboarding
        onComplete={() => {
          localStorage.setItem("tro.onboarded", "true");
          setOnboarded(true);
          void desktop.followCursorCompanion();
          void desktop.hideMainWindow();
        }}
      />
    );
  if (settings)
    return (
      <SettingsPage
        onClose={() => {
          setSettings(false);
          void desktop.hideMainWindow();
        }}
      />
    );

  return null;
}
