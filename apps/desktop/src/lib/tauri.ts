import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppSettings,
  AppSnapshot,
  ConfirmationRequest,
  OverlayUpdate,
  PermissionSnapshot,
} from "./contracts";

const inTauri = () =>
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const browserSnapshot: AppSnapshot = {
  assistant: "idle",
  agent: "idle",
  transcript: null,
  status_vi: "Sẵn sàng",
  capture_active: false,
};

export const desktop = {
  snapshot: async (): Promise<AppSnapshot> =>
    inTauri() ? invoke<AppSnapshot>("get_app_snapshot") : browserSnapshot,
  permissions: async (): Promise<PermissionSnapshot> =>
    inTauri()
      ? invoke<PermissionSnapshot>("get_permission_snapshot")
      : {
          microphone: "not_determined",
          screen_capture: "not_determined",
          input_control: "not_determined",
        },
  startAssistant: async (): Promise<void> =>
    inTauri() ? invoke("start_assistant", { source: "ui" }) : undefined,
  stopAssistant: async (): Promise<void> =>
    inTauri() ? invoke("stop_assistant", { reason: "user" }) : undefined,
  startAgent: async (goal: string): Promise<void> =>
    inTauri()
      ? invoke("start_agent", { goal, sourceFrameId: null })
      : undefined,
  emergencyStop: async (): Promise<void> =>
    inTauri() ? invoke("emergency_stop") : undefined,
  requestPermission: async (permission: string): Promise<PermissionSnapshot> =>
    inTauri()
      ? invoke<PermissionSnapshot>("request_permission", { permission })
      : desktop.permissions(),
  updateSettings: async (
    settings: Partial<AppSettings>,
  ): Promise<AppSettings> =>
    inTauri()
      ? invoke<AppSettings>("update_settings", { patch: settings })
      : {
          locale: "vi",
          ask_shortcut: "CommandOrControl+Shift+Space",
          dictation_shortcut: "CommandOrControl+Shift+D",
          stop_shortcut: "Escape",
          reduced_motion: false,
          dictation_preview: true,
          optional_telemetry: false,
          ...settings,
        },
  resolveConfirmation: async (
    confirmationId: string,
    decision: "allow_once" | "stop",
  ): Promise<void> =>
    inTauri()
      ? invoke("resolve_confirmation", { confirmationId, decision })
      : undefined,
  onSnapshot: (
    handler: (snapshot: AppSnapshot) => void,
  ): Promise<UnlistenFn> =>
    inTauri()
      ? listen<AppSnapshot>("assistant_state_changed", (event) => {
          handler(event.payload);
        })
      : Promise.resolve(() => undefined),
  onOverlay: (handler: (update: OverlayUpdate) => void): Promise<UnlistenFn> =>
    inTauri()
      ? listen<OverlayUpdate>("overlay_changed", (event) => {
          handler(event.payload);
        })
      : Promise.resolve(() => undefined),
  onConfirmation: (
    handler: (request: ConfirmationRequest) => void,
  ): Promise<UnlistenFn> =>
    inTauri()
      ? listen<ConfirmationRequest>("confirmation_requested", (event) => {
          handler(event.payload);
        })
      : Promise.resolve(() => undefined),
  onGlobalShortcut: (
    handler: (action: "ask" | "dictation" | "stop") => void,
  ): Promise<UnlistenFn> =>
    inTauri()
      ? listen<"ask" | "dictation" | "stop">("global_shortcut", (event) => {
          handler(event.payload);
        })
      : Promise.resolve(() => undefined),
  onOpenSettings: (handler: () => void): Promise<UnlistenFn> =>
    inTauri()
      ? listen("open_settings", handler)
      : Promise.resolve(() => undefined),
};
