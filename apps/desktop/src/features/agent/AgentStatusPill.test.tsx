import { cleanup, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentState } from "../../lib/contracts";
import { useAssistantStore } from "../assistant/assistantStore";
import { desktop } from "../../lib/tauri";
import { useAgentStore } from "./agentStore";
import { AgentStatusPill } from "./AgentStatusPill";

vi.mock("../../lib/tauri", () => ({
  desktop: {
    emergencyStop: vi.fn(() => Promise.resolve()),
    startAgent: vi.fn(() => Promise.resolve()),
    startAgentForApp: vi.fn(() => Promise.resolve()),
  },
}));

const states: Array<[AgentState, string]> = [
  ["resolving_app", "Đang tìm ứng dụng"],
  ["awaiting_app_approval", "Đang chờ cho phép ứng dụng"],
  ["activating_app", "Đang mở ứng dụng"],
  ["planning", "Đang lập kế hoạch"],
  ["validating", "Đang kiểm tra mục tiêu"],
  ["awaiting_confirmation", "Đang chờ bạn xác nhận"],
  ["executing", "Đang thao tác"],
  ["stabilizing", "Đang chờ ứng dụng ổn định"],
  ["observing", "Đang kiểm tra kết quả"],
  ["stale_recovery", "Giao diện đổi — đang quan sát lại"],
  ["needs_user", "Cần bạn hỗ trợ"],
  ["paused_by_user", "Bạn đã tiếp quản"],
];

describe("AgentStatusPill", () => {
  afterEach(cleanup);

  beforeEach(() => {
    vi.clearAllMocks();
    useAgentStore.setState({ goal: "Mở khóa học số năm" });
    useAssistantStore.setState({
      assistant: "idle",
      agent: "idle",
      transcript: null,
      status_vi: "Sẵn sàng",
      capture_active: false,
      scoped_app_name: null,
      agent_choices: [],
    });
  });

  it("starts a fresh app-scoped run from an ambiguous candidate", async () => {
    useAssistantStore.setState({
      agent: "needs_user",
      agent_choices: [
        {
          app_id: "abc-browser",
          display_name: "ABC Browser",
          identity_summary: "Ứng dụng đã cài đặt",
        },
      ],
    });
    render(<AgentStatusPill />);
    await userEvent.click(screen.getByRole("button", { name: /ABC Browser/ }));
    expect(desktop.startAgentForApp).toHaveBeenCalledWith(
      "Mở khóa học số năm",
      "abc-browser",
    );
  });

  it("resumes takeover only through a fresh run", async () => {
    useAssistantStore.setState({ agent: "paused_by_user" });
    render(<AgentStatusPill />);
    await userEvent.click(
      screen.getByRole("button", { name: "Tiếp tục từ quan sát mới" }),
    );
    expect(desktop.startAgent).toHaveBeenCalledWith("Mở khóa học số năm");
  });

  it.each(states)("renders %s", (state, label) => {
    useAssistantStore.setState({
      agent: state,
      scoped_app_name: "ABC Browser",
    });
    render(<AgentStatusPill />);
    expect(screen.getByText(new RegExp(label))).toBeVisible();
    expect(screen.getByText(/ABC Browser/)).toBeVisible();
    expect(screen.getByText("Dừng")).toBeVisible();
  });
});
