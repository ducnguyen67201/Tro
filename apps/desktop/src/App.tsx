import { useEffect, useRef, useState } from "react";
import { useAgentStore } from "./features/agent/agentStore";
import { useAssistantStore } from "./features/assistant/assistantStore";
import { SignIn } from "./features/auth/SignIn";
import { Onboarding } from "./features/onboarding/Onboarding";
import { SettingsPage } from "./features/settings/SettingsPage";
import { desktop } from "./lib/tauri";

const MAX_HOLD_TO_TALK_MS = 30_000;
type AuthState = "checking" | "signed_out" | "signed_in";

export default function App() {
  const [onboarded, setOnboarded] = useState(
    () => localStorage.getItem("tro.onboarded") === "true",
  );
  const [settings, setSettings] = useState(false);
  const [authState, setAuthState] = useState<AuthState>("checking");
  const authenticatedRef = useRef(false);
  const setSnapshot = useAssistantStore((state) => state.setSnapshot);
  const setConfirmation = useAgentStore((state) => state.setConfirmation);

  useEffect(() => {
    let disposed = false;
    let pendingAsk:
      | {
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
      // Release closes the microphone immediately. Native code waits for any
      // in-flight screenshot after the audio stream has already been dropped.
      void desktop.finishAssistant(reason).catch(() => undefined);
    };

    void desktop
      .authSnapshot()
      .then((snapshot) => {
        authenticatedRef.current = snapshot.authenticated;
        setAuthState(snapshot.authenticated ? "signed_in" : "signed_out");
      })
      .catch(() => {
        authenticatedRef.current = false;
        setAuthState("signed_out");
      });
    void desktop
      .snapshot()
      .then(setSnapshot)
      .catch(() => undefined);
    own(desktop.onSnapshot(setSnapshot));
    own(desktop.onConfirmation(setConfirmation));
    own(
      desktop.onAuthenticationChanged((snapshot) => {
        authenticatedRef.current = snapshot.authenticated;
        setAuthState(snapshot.authenticated ? "signed_in" : "signed_out");
      }),
    );
    own(
      desktop.onGlobalShortcut((action) => {
        if (action === "ask") {
          if (!authenticatedRef.current) {
            void desktop.dismissCursorCompanion();
            void desktop.showMainWindow();
            setAuthState("signed_out");
            return;
          }
          if (pendingAsk) return;

          const current = {
            timeout: 0 as ReturnType<typeof window.setTimeout>,
            finished: false,
          };
          void desktop.startAssistant("command_control").catch(() => undefined);
          current.timeout = window.setTimeout(() => {
            if (pendingAsk === current) {
              finishPendingAsk("command_control_timeout");
            }
          }, MAX_HOLD_TO_TALK_MS);
          pendingAsk = current;
        }
        if (action === "ask_release") {
          finishPendingAsk("command_control_released");
        }
        if (action === "dictation") {
          if (!authenticatedRef.current) {
            void desktop.dismissCursorCompanion();
            void desktop.showMainWindow();
            setAuthState("signed_out");
          } else {
            void desktop.startAssistant();
          }
        }
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
    authenticatedRef.current = authState === "signed_in";
  }, [authState]);

  useEffect(() => {
    let disposed = false;
    if (authState === "checking") {
      return () => {
        disposed = true;
      };
    }
    if (authState === "signed_out") {
      void desktop.dismissCursorCompanion();
      void desktop.showMainWindow();
      return () => {
        disposed = true;
      };
    }
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
  }, [authState, onboarded]);

  if (authState !== "signed_in")
    return (
      <SignIn
        checking={authState === "checking"}
        onAuthenticated={() => {
          authenticatedRef.current = true;
          setAuthState("signed_in");
        }}
      />
    );

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
