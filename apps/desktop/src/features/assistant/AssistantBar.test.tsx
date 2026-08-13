import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import "../../lib/i18n";
import { AssistantBar } from "./AssistantBar";

describe("AssistantBar", () => {
  it("is Vietnamese-first on a clean state", () => {
    render(<AssistantBar />);
    expect(screen.getByRole("button", { name: "Hỏi Tro" })).toBeVisible();
    expect(screen.getByText("Sẵn sàng")).toBeVisible();
  });
});
