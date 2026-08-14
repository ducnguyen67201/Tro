import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ApprovedAppsSettings } from "./ApprovedAppsSettings";

const bridge = vi.hoisted(() => ({
  approvedApps: vi.fn(() =>
    Promise.resolve([
      {
        app_id: "browser-id",
        display_name: "ABC Browser",
        identity_summary: "com.example.browser",
      },
    ]),
  ),
  revokeApprovedApp: vi.fn(() => Promise.resolve(true)),
}));

vi.mock("../../lib/tauri", () => ({ desktop: bridge }));

describe("ApprovedAppsSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("lists and revokes a stable app identity", async () => {
    render(<ApprovedAppsSettings />);
    expect(await screen.findByText("ABC Browser")).toBeVisible();
    fireEvent.click(screen.getByText("Thu hồi"));
    expect(bridge.revokeApprovedApp).toHaveBeenCalledWith("browser-id");
    expect(await screen.findByText("Chưa có ứng dụng nào.")).toBeVisible();
  });
});
