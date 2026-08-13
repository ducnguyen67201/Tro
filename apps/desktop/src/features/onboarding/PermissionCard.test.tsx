import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";
import { PermissionCard } from "./PermissionCard";

describe("PermissionCard", () => {
  test("requests permission from the allow button", () => {
    const onRequest = vi.fn();
    render(
      <PermissionCard
        icon="▣"
        title="Màn hình"
        detail="Chụp màn hình"
        status="not_determined"
        onRequest={onRequest}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "Cho phép" }));
    expect(onRequest).toHaveBeenCalledOnce();
  });

  test("prevents repeated requests while macOS opens its permission flow", () => {
    render(
      <PermissionCard
        icon="↖"
        title="Điều khiển nhập liệu"
        detail="Điều khiển có xác nhận"
        status="not_determined"
        requesting
        onRequest={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "Đang mở…" })).toBeDisabled();
  });

  test("offers to relaunch after macOS grants a restart-bound permission", () => {
    render(
      <PermissionCard
        icon="↖"
        title="Phím tắt & điều khiển"
        detail="Nhận phím tắt toàn cục"
        status="restart_required"
        onRequest={() => undefined}
      />,
    );

    expect(screen.getByRole("button", { name: "Mở lại Tro" })).toBeEnabled();
  });
});
