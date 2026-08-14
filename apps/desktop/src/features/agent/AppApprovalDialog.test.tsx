import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ConfirmationRequest } from "../../lib/contracts";
import { AppApprovalDialog } from "./AppApprovalDialog";

const appRequest: ConfirmationRequest = {
  confirmation_id: "one",
  kind: "app_access",
  action_vi: "Cho phép Tro dùng ứng dụng này",
  consequence_vi: "Quyền ứng dụng được giới hạn.",
  app_name: "ABC Browser",
  identity_summary: "com.example.browser",
  choices: ["allow_once", "always_allow", "stop"],
  expires_at_unix_ms: 1,
};

describe("AppApprovalDialog", () => {
  afterEach(cleanup);

  it("offers always allow only for app access", () => {
    const resolve = vi.fn();
    render(<AppApprovalDialog request={appRequest} onResolve={resolve} />);
    fireEvent.click(screen.getByText("Luôn cho phép ứng dụng này"));
    expect(resolve).toHaveBeenCalledWith("always_allow");
    expect(screen.getByRole("alertdialog")).toHaveAccessibleName(
      "Cho phép Tro dùng ứng dụng này",
    );
    expect(screen.getByRole("button", { name: "Dừng" })).toHaveFocus();
  });

  it("never offers persistent approval for a consequential action", () => {
    render(
      <AppApprovalDialog
        request={{ ...appRequest, kind: "consequential_action" }}
        onResolve={vi.fn()}
      />,
    );
    expect(screen.queryByText("Luôn cho phép ứng dụng này")).toBeNull();
  });
});
