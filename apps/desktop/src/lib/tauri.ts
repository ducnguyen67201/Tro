import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppSettings,
  AppSnapshot,
  ConfirmationRequest,
  CursorCompanionSnapshot,
  LlmConfig,
  OverlayUpdate,
  PermissionSnapshot,
  ShortcutAction,
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

const isMacOS = () =>
  typeof navigator !== "undefined" && /Mac/.test(navigator.userAgent);

export const defaultAskShortcut = () =>
  isMacOS() ? "Command+Option" : "CommandOrControl+Shift+Space";

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
  startAssistant: async (source = "ui"): Promise<void> =>
    inTauri() ? invoke("start_assistant", { source }) : undefined,
  finishAssistant: async (reason = "user_released"): Promise<void> =>
    inTauri() ? invoke("finish_assistant", { reason }) : undefined,
  stopAssistant: async (reason = "user"): Promise<void> =>
    inTauri() ? invoke("stop_assistant", { reason }) : undefined,
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
          ask_shortcut: defaultAskShortcut(),
          dictation_shortcut: "CommandOrControl+Shift+D",
          stop_shortcut: "Escape",
          reduced_motion: false,
          dictation_preview: true,
          optional_telemetry: false,
          ...settings,
        },
  llmConfig: async (): Promise<LlmConfig> =>
    inTauri()
      ? invoke<LlmConfig>("get_llm_config")
      : {
          backend_url: "http://127.0.0.1:8080",
          timeout_seconds: 25,
          device_authenticated: false,
        },
  restart: async (): Promise<void> =>
    inTauri() ? invoke("restart_app") : undefined,
  cursorCompanionSnapshot: async (): Promise<CursorCompanionSnapshot> =>
    inTauri()
      ? invoke<CursorCompanionSnapshot>("get_cursor_companion_snapshot")
      : { phase: "hidden" },
  showMainWindow: async (): Promise<void> =>
    inTauri() ? invoke("show_main_window") : undefined,
  hideMainWindow: async (): Promise<void> =>
    inTauri() ? invoke("hide_main_window") : undefined,
  followCursorCompanion: async (): Promise<void> =>
    inTauri() ? invoke("follow_cursor_companion") : undefined,
  dismissCursorCompanion: async (): Promise<void> =>
    inTauri() ? invoke("dismiss_cursor_companion") : undefined,
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
    handler: (action: ShortcutAction) => void,
  ): Promise<UnlistenFn> =>
    inTauri()
      ? listen<ShortcutAction>("global_shortcut", (event) => {
          handler(event.payload);
        })
      : Promise.resolve(() => undefined),
  onCursorCompanion: (
    handler: (snapshot: CursorCompanionSnapshot) => void,
  ): Promise<UnlistenFn> =>
    inTauri()
      ? listen<CursorCompanionSnapshot>("cursor_companion_changed", (event) => {
          handler(event.payload);
        })
      : Promise.resolve(() => undefined),
  onOpenSettings: (handler: () => void): Promise<UnlistenFn> =>
    inTauri()
      ? listen("open_settings", handler)
      : Promise.resolve(() => undefined),
};
