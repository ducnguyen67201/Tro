export type AssistantState =
  | "idle"
  | "capturing"
  | "listening"
  | "thinking"
  | "speaking"
  | "guiding"
  | "failed";

export type AgentState =
  | "idle"
  | "planning"
  | "awaiting_confirmation"
  | "executing"
  | "observing"
  | "completed"
  | "stopped"
  | "failed";

export interface AppSnapshot {
  assistant: AssistantState;
  agent: AgentState;
  transcript: string | null;
  status_vi: string;
  capture_active: boolean;
}

export type CursorCompanionPhase = "hidden" | "following" | "anchored";

export interface CursorCompanionSnapshot {
  phase: CursorCompanionPhase;
}

export type ShortcutAction = "ask" | "ask_release" | "dictation" | "stop";

export type PermissionStatus =
  | "not_determined"
  | "granted"
  | "denied"
  | "restricted"
  | "unavailable"
  | "restart_required";

export interface PermissionSnapshot {
  microphone: PermissionStatus;
  screen_capture: PermissionStatus;
  input_control: PermissionStatus;
}

export interface Point {
  x: number;
  y: number;
}

export type OverlayElement =
  | {
      kind: "rect";
      bounds: { x: number; y: number; width: number; height: number };
      label?: string;
    }
  | { kind: "arrow"; from: Point; to: Point; label?: string }
  | { kind: "point"; at: Point; label?: string }
  | { kind: "step"; at: Point; number: number; label: string };

export interface OverlayUpdate {
  session_id: string;
  generation: number;
  monitor_id: string;
  elements: OverlayElement[];
  expires_after_ms: number;
}

export interface ConfirmationRequest {
  confirmation_id: string;
  action_vi: string;
  consequence_vi: string;
  app_name: string;
  expires_at_unix_ms: number;
}

export interface AppSettings {
  locale: "vi" | "en";
  ask_shortcut: string;
  dictation_shortcut: string;
  stop_shortcut: string;
  reduced_motion: boolean;
  dictation_preview: boolean;
  optional_telemetry: boolean;
}
