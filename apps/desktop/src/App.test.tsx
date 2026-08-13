import {
  act,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import App from "./App";

const bridge = vi.hoisted(() => ({
  setSnapshot: vi.fn(),
  setConfirmation: vi.fn(),
  showMainWindow: vi.fn(() => Promise.resolve()),
  hideMainWindow: vi.fn(() => Promise.resolve()),
  followCursorCompanion: vi.fn(() => Promise.resolve()),
  startAssistant: vi.fn(() => Promise.resolve()),
  finishAssistant: vi.fn(() => Promise.resolve()),
  emergencyStop: vi.fn(() => Promise.resolve()),
  permissions: vi.fn(() =>
    Promise.resolve({
      microphone: "granted",
      screen_capture: "granted",
      input_control: "granted",
    }),
  ),
  openSettings: undefined as (() => void) | undefined,
  shortcut: undefined as ((action: string) => void) | undefined,
  unlisteners: [vi.fn(), vi.fn(), vi.fn(), vi.fn()],
}));

vi.mock("./features/assistant/assistantStore", () => ({
  useAssistantStore: (
    selector: (state: { setSnapshot: typeof bridge.setSnapshot }) => unknown,
  ) => selector({ setSnapshot: bridge.setSnapshot }),
}));

vi.mock("./features/agent/agentStore", () => ({
  useAgentStore: (
    selector: (state: {
      setConfirmation: typeof bridge.setConfirmation;
    }) => unknown,
  ) => selector({ setConfirmation: bridge.setConfirmation }),
}));

vi.mock("./features/onboarding/Onboarding", () => ({
  Onboarding: ({ onComplete }: { onComplete: () => void }) => (
    <button type="button" onClick={onComplete}>
      Finish onboarding
    </button>
  ),
}));

vi.mock("./features/settings/SettingsPage", () => ({
  SettingsPage: ({ onClose }: { onClose: () => void }) => (
    <button type="button" onClick={onClose}>
      Close settings
    </button>
  ),
}));

vi.mock("./lib/tauri", () => ({
  desktop: {
    snapshot: vi.fn(() =>
      Promise.resolve({
        assistant: "idle",
        agent: "idle",
        transcript: null,
        status_vi: "Sẵn sàng",
        capture_active: false,
      }),
    ),
    onSnapshot: vi.fn(() => Promise.resolve(bridge.unlisteners[0])),
    onConfirmation: vi.fn(() => Promise.resolve(bridge.unlisteners[1])),
    onGlobalShortcut: vi.fn((handler: (action: string) => void) => {
      bridge.shortcut = handler;
      return Promise.resolve(bridge.unlisteners[2]);
    }),
    onOpenSettings: vi.fn((handler: () => void) => {
      bridge.openSettings = handler;
      return Promise.resolve(bridge.unlisteners[3]);
    }),
    showMainWindow: bridge.showMainWindow,
    hideMainWindow: bridge.hideMainWindow,
    followCursorCompanion: bridge.followCursorCompanion,
    startAssistant: bridge.startAssistant,
    finishAssistant: bridge.finishAssistant,
    emergencyStop: bridge.emergencyStop,
    permissions: bridge.permissions,
  },
}));

describe("App background lifecycle", () => {
  const storage = new Map<string, string>();

  beforeEach(() => {
    storage.clear();
    vi.stubGlobal("localStorage", {
      getItem: (key: string) => storage.get(key) ?? null,
      setItem: (key: string, value: string) => {
        storage.set(key, value);
      },
      removeItem: (key: string) => {
        storage.delete(key);
      },
      clear: () => {
        storage.clear();
      },
      key: (index: number) => [...storage.keys()][index] ?? null,
      get length() {
        return storage.size;
      },
    });
    bridge.openSettings = undefined;
    bridge.shortcut = undefined;
    vi.clearAllMocks();
  });

  test("shows onboarding once and hides after completion", async () => {
    render(<App />);

    expect(
      await screen.findByRole("button", { name: "Finish onboarding" }),
    ).toBeInTheDocument();
    await waitFor(() => {
      expect(bridge.showMainWindow).toHaveBeenCalled();
    });
    fireEvent.click(screen.getByRole("button", { name: "Finish onboarding" }));

    expect(localStorage.getItem("tro.onboarded")).toBe("true");
    await waitFor(() => {
      expect(bridge.hideMainWindow).toHaveBeenCalled();
      expect(bridge.followCursorCompanion).toHaveBeenCalled();
    });
  });

  test("keeps the main window hidden and starts the cursor follower", async () => {
    localStorage.setItem("tro.onboarded", "true");
    const { container } = render(<App />);

    await waitFor(() => {
      expect(bridge.hideMainWindow).toHaveBeenCalled();
      expect(bridge.followCursorCompanion).toHaveBeenCalled();
    });
    expect(container).toBeEmptyDOMElement();
    expect(bridge.showMainWindow).not.toHaveBeenCalled();
  });

  test("opens settings from the tray event and hides it on close", async () => {
    localStorage.setItem("tro.onboarded", "true");
    render(<App />);
    await waitFor(() => {
      expect(bridge.openSettings).toBeTypeOf("function");
    });

    act(() => {
      bridge.openSettings?.();
    });
    fireEvent.click(
      await screen.findByRole("button", { name: "Close settings" }),
    );
    expect(bridge.hideMainWindow).toHaveBeenCalled();
  });

  test("opens settings when the global shortcut permission is missing", async () => {
    localStorage.setItem("tro.onboarded", "true");
    bridge.permissions.mockResolvedValueOnce({
      microphone: "granted",
      screen_capture: "granted",
      input_control: "not_determined",
    });
    render(<App />);

    expect(
      await screen.findByRole("button", { name: "Close settings" }),
    ).toBeInTheDocument();
    expect(bridge.showMainWindow).toHaveBeenCalled();
  });

  test("finishes instead of cancelling when the shortcut is released", async () => {
    localStorage.setItem("tro.onboarded", "true");
    render(<App />);
    await waitFor(() => {
      expect(bridge.shortcut).toBeTypeOf("function");
    });

    act(() => {
      bridge.shortcut?.("ask_release");
    });
    await waitFor(() => {
      expect(bridge.finishAssistant).toHaveBeenCalledWith(
        "command_option_released",
      );
    });
  });
});
