import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { OverlayCanvas } from "./features/overlay/OverlayCanvas";
import { ConfirmationDialog } from "./features/agent/ConfirmationDialog";
import { CursorAssistant } from "./features/assistant/CursorAssistant";
import "./lib/i18n";
import "./styles.css";

const label =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
    ? getCurrentWindow().label
    : "main";

const isOverlayWindow = label.startsWith("overlay-");
const isCursorAssistant = label === "assistant-cursor";

if (isOverlayWindow) {
  document.documentElement.dataset.windowKind = "overlay";
} else if (isCursorAssistant) {
  document.documentElement.dataset.windowKind = "cursor-assistant";
}

const Root = isOverlayWindow
  ? OverlayCanvas
  : isCursorAssistant
    ? CursorAssistant
    : label === "confirmation"
      ? ConfirmationDialog
      : App;

const root = document.getElementById("root");
if (root) {
  createRoot(root).render(
    <StrictMode>
      <Root />
    </StrictMode>,
  );
}
