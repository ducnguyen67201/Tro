import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { SignIn } from "./SignIn";

const { signInWithGoogle } = vi.hoisted(() => ({
  signInWithGoogle: vi.fn(),
}));

vi.mock("../../lib/tauri", () => ({
  desktop: { signInWithGoogle },
}));

beforeEach(() => {
  signInWithGoogle.mockReset();
});

afterEach(cleanup);

test("opens Google login before continuing", async () => {
  signInWithGoogle.mockResolvedValue({ authenticated: true });
  const authenticated = vi.fn();
  render(<SignIn onAuthenticated={authenticated} />);

  fireEvent.click(screen.getByRole("button", { name: "Tiếp tục với Google" }));

  await waitFor(() => {
    expect(signInWithGoogle).toHaveBeenCalledOnce();
    expect(authenticated).toHaveBeenCalledOnce();
  });
});

test("shows a localized inline Google login error", async () => {
  signInWithGoogle.mockRejectedValue({
    code: "provider_unavailable",
    message_vi: "Google đang tạm thời không phản hồi. Hãy thử lại sau.",
  });
  render(<SignIn onAuthenticated={vi.fn()} />);

  fireEvent.click(screen.getByRole("button", { name: "Tiếp tục với Google" }));

  expect(await screen.findByRole("alert")).toHaveTextContent(
    "Google đang tạm thời không phản hồi. Hãy thử lại sau.",
  );
});
