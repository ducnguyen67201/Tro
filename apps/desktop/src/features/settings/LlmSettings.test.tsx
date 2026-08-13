import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { LlmSettings } from "./LlmSettings";

const bridge = vi.hoisted(() => ({
  llmConfig: vi.fn(() =>
    Promise.resolve({
      backend_url: "https://api.tro.example",
      timeout_seconds: 25,
      device_authenticated: true,
    }),
  ),
}));

vi.mock("../../lib/tauri", () => ({ desktop: bridge }));

describe("LlmSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  test("shows only the Tro backend boundary and never renders a provider key field", async () => {
    render(<LlmSettings />);

    expect(await screen.findByText("https://api.tro.example")).toBeVisible();
    expect(screen.getByText("Thiết bị đã xác thực")).toBeVisible();
    expect(screen.queryByLabelText(/API key/i)).not.toBeInTheDocument();
    expect(screen.getByText(/chỉ tồn tại ở backend/i)).toBeVisible();
  });
});
