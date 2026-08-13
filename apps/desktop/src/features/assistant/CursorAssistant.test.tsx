import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import type { AppSnapshot, CursorCompanionSnapshot } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";
import { CursorAssistant } from "./CursorAssistant";

const bridge = vi.hoisted(() => ({
  companion: { phase: "hidden" } as CursorCompanionSnapshot,
  snapshot: {
    assistant: "idle",
    agent: "idle",
    transcript: null,
    status_vi: "Sẵn sàng",
    capture_active: false,
  } as AppSnapshot,
  stopAssistant: vi.fn(),
  dismissCursorCompanion: vi.fn(),
  unlistenCompanion: vi.fn(),
  unlistenSnapshot: vi.fn(),
}));

vi.mock("../../lib/tauri", () => ({
  desktop: {
    cursorCompanionSnapshot: vi.fn(() => Promise.resolve(bridge.companion)),
    snapshot: vi.fn(() => Promise.resolve(bridge.snapshot)),
    onCursorCompanion: vi.fn(() => Promise.resolve(bridge.unlistenCompanion)),
    onSnapshot: vi.fn(() => Promise.resolve(bridge.unlistenSnapshot)),
    stopAssistant: bridge.stopAssistant,
    dismissCursorCompanion: bridge.dismissCursorCompanion,
  },
}));

describe("CursorAssistant", () => {
  beforeEach(() => {
    bridge.companion = { phase: "hidden" };
    bridge.snapshot = {
      assistant: "idle",
      agent: "idle",
      transcript: null,
      status_vi: "Sẵn sàng",
      capture_active: false,
    };
    vi.clearAllMocks();
  });

  test("renders only the orb while following", async () => {
    bridge.companion = { phase: "following" };
    const { container } = render(<CursorAssistant />);

    await waitFor(() => {
      expect(container.querySelector(".cursor-following")).toBeInTheDocument();
    });
    expect(container.querySelector(".cursor-orb img")).toHaveAttribute(
      "src",
      "/tro-companion-v1.png",
    );
    expect(screen.queryByText("T")).not.toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  test("renders the orb while the native phase snapshot is still loading", () => {
    vi.mocked(desktop.cursorCompanionSnapshot).mockReturnValueOnce(
      new Promise<CursorCompanionSnapshot>(() => undefined),
    );

    const { container } = render(<CursorAssistant />);

    expect(container.querySelector(".cursor-following")).toBeInTheDocument();
  });

  test("renders a detached acting orb without opening a card", async () => {
    bridge.companion = { phase: "acting" };
    const { container } = render(<CursorAssistant />);

    await waitFor(() => {
      expect(
        container.querySelector(".cursor-following.is-acting"),
      ).toBeInTheDocument();
    });
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  test("shows assistant state and stops an active anchored turn", async () => {
    bridge.companion = { phase: "anchored" };
    bridge.snapshot = {
      ...bridge.snapshot,
      assistant: "listening",
      status_vi: "Đang nghe…",
    };
    render(<CursorAssistant />);

    fireEvent.click(await screen.findByRole("button", { name: "Dừng Tro" }));
    expect(bridge.stopAssistant).toHaveBeenCalledWith("cursor_card");
    expect(bridge.dismissCursorCompanion).not.toHaveBeenCalled();
  });

  test("shows transcript and dismisses an idle tray card", async () => {
    bridge.companion = { phase: "anchored" };
    bridge.snapshot = {
      ...bridge.snapshot,
      transcript: "Giải thích phương trình này",
    };
    render(<CursorAssistant />);

    expect(
      await screen.findByText("Giải thích phương trình này"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Đóng Tro" }));
    expect(bridge.dismissCursorCompanion).toHaveBeenCalledOnce();
    expect(bridge.stopAssistant).not.toHaveBeenCalled();
  });

  test("unsubscribes from native events", async () => {
    const { unmount } = render(<CursorAssistant />);
    await waitFor(() => {
      expect(bridge.unlistenCompanion).not.toHaveBeenCalled();
    });
    unmount();
    await waitFor(() => {
      expect(bridge.unlistenCompanion).toHaveBeenCalledOnce();
      expect(bridge.unlistenSnapshot).toHaveBeenCalledOnce();
    });
  });
});
