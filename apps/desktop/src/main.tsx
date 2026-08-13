import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App";
import { OverlayCanvas } from "./features/overlay/OverlayCanvas";
import { ConfirmationDialog } from "./features/agent/ConfirmationDialog";
import "./lib/i18n";
import "./styles.css";

const label =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
    ? getCurrentWindow().label
    : "main";

const isOverlayWindow = label.startsWith("overlay-");

if (isOverlayWindow) {
  document.documentElement.dataset.windowKind = "overlay";
}

const Root = isOverlayWindow
  ? OverlayCanvas
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
