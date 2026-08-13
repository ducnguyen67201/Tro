import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";
import { LlmSettings } from "./LlmSettings";

const bridge = vi.hoisted(() => ({
  llmConfig: vi.fn(() =>
    Promise.resolve({
      provider: "openrouter",
      base_url: "https://openrouter.ai/api/v1",
      model: "google/gemini-2.5-flash",
      timeout_seconds: 20,
      api_key_configured: false,
    }),
  ),
  updateLlmConfig: vi.fn(() =>
    Promise.resolve({
      provider: "openrouter",
      base_url: "https://openrouter.ai/api/v1",
      model: "google/gemini-2.5-flash",
      timeout_seconds: 20,
      api_key_configured: true,
    }),
  ),
}));

vi.mock("../../lib/tauri", () => ({ desktop: bridge }));

describe("LlmSettings", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  test("loads config and saves the API key without rendering it again", async () => {
    render(<LlmSettings />);

    expect(
      await screen.findByDisplayValue("google/gemini-2.5-flash"),
    ).toBeVisible();
    fireEvent.change(screen.getByLabelText("OpenRouter API key"), {
      target: { value: "test-key-not-a-secret-and-long-enough" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Lưu LLM" }));

    await waitFor(() => {
      expect(bridge.updateLlmConfig).toHaveBeenCalledWith(
        expect.objectContaining({
          provider: "openrouter",
          model: "google/gemini-2.5-flash",
          timeout_seconds: 20,
          api_key: "test-key-not-a-secret-and-long-enough",
        }),
      );
    });
    expect(await screen.findByText("Đã có API key")).toBeVisible();
    expect(screen.getByLabelText("OpenRouter API key")).toHaveValue("");
  });
});
