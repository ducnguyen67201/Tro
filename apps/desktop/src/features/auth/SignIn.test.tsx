import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { SignIn } from "./SignIn";

const { signIn } = vi.hoisted(() => ({ signIn: vi.fn() }));

vi.mock("../../lib/tauri", () => ({
  desktop: { signIn },
}));

beforeEach(() => {
  signIn.mockReset();
});

afterEach(cleanup);

test("submits a real access-code login before continuing", async () => {
  signIn.mockResolvedValue({ authenticated: true });
  const authenticated = vi.fn();
  render(<SignIn onAuthenticated={authenticated} />);

  fireEvent.change(screen.getByLabelText("Mã truy cập"), {
    target: { value: "tro-test" },
  });
  fireEvent.click(
    screen.getByLabelText("Tôi là sinh viên đại học và đã đủ 18 tuổi."),
  );
  fireEvent.click(screen.getByRole("button", { name: /Tiếp tục/ }));

  await waitFor(() => {
    expect(signIn).toHaveBeenCalledWith("TRO-TEST", true);
    expect(authenticated).toHaveBeenCalledOnce();
  });
});

test("shows a localized inline login error", async () => {
  signIn.mockRejectedValue({
    code: "invite_invalid",
    message_vi: "Mã mời không hợp lệ hoặc đã hết hạn.",
  });
  render(<SignIn onAuthenticated={vi.fn()} />);

  fireEvent.change(screen.getByLabelText("Mã truy cập"), {
    target: { value: "TRO-WRONG" },
  });
  fireEvent.click(
    screen.getByLabelText("Tôi là sinh viên đại học và đã đủ 18 tuổi."),
  );
  fireEvent.click(screen.getByRole("button", { name: /Tiếp tục/ }));

  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Mã mời không hợp lệ hoặc đã hết hạn.",
  );
});
