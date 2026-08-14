import { useState } from "react";
import { desktop } from "../../lib/tauri";

interface SignInProps {
  checking?: boolean;
  onAuthenticated: () => void;
}

export function SignIn({ checking = false, onAuthenticated }: SignInProps) {
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  const signInWithGoogle = () => {
    if (checking || submitting) return;
    setSubmitting(true);
    setError("");
    void desktop
      .signInWithGoogle()
      .then((snapshot) => {
        if (snapshot.authenticated) onAuthenticated();
      })
      .catch((reason: unknown) => {
        setError(authErrorMessage(reason));
      })
      .finally(() => {
        setSubmitting(false);
      });
  };

  return (
    <main className="auth-shell">
      <section className="auth-card" aria-labelledby="auth-title">
        <div className="brand-lockup auth-brand">
          <span className="brand-mark">T</span>
          <span>Tro</span>
        </div>
        <div className="auth-orb" aria-hidden="true">
          <span />
          <i />
        </div>
        <span className="eyebrow">Trợ lý học tập tiếng Việt</span>
        <h1 id="auth-title">Chào mừng đến Tro</h1>
        <p className="auth-intro">
          Đăng nhập bằng Google một lần. Sau đó, bạn có thể gọi Tro ở bất kỳ ứng
          dụng nào bằng ⌘ + ⌥.
        </p>

        {checking ? (
          <div className="auth-checking" role="status">
            <span aria-hidden="true" /> Đang kiểm tra phiên đăng nhập…
          </div>
        ) : (
          <div className="auth-form">
            {error ? (
              <p id="auth-error" className="auth-error" role="alert">
                {error}
              </p>
            ) : null}
            <button
              className="auth-google"
              type="button"
              disabled={submitting}
              onClick={signInWithGoogle}
            >
              {submitting ? (
                <span className="auth-button-spinner" aria-hidden="true" />
              ) : (
                <GoogleMark />
              )}
              {submitting
                ? "Hoàn tất đăng nhập trong trình duyệt…"
                : "Tiếp tục với Google"}
            </button>
            <p className="auth-browser-note">
              Tro sẽ mở trình duyệt mặc định để bạn đăng nhập an toàn.
            </p>
          </div>
        )}

        <p className="auth-privacy">
          Tro chỉ lưu phiên thiết bị trong kho bảo mật của máy. Ảnh và âm thanh
          chỉ được gửi khi bạn chủ động gọi Tro.
        </p>
      </section>
      <p className="auth-footer">Vietnamese-first · Riêng tư theo mặc định</p>
    </main>
  );
}

function GoogleMark() {
  return (
    <svg className="google-mark" viewBox="0 0 18 18" aria-hidden="true">
      <path
        fill="#4285F4"
        d="M17.64 9.205c0-.639-.057-1.252-.164-1.841H9v3.481h4.844a4.14 4.14 0 0 1-1.797 2.715v2.258h2.909c1.702-1.567 2.684-3.875 2.684-6.613Z"
      />
      <path
        fill="#34A853"
        d="M9 18c2.43 0 4.468-.806 5.956-2.182l-2.909-2.258c-.806.54-1.835.859-3.047.859-2.344 0-4.328-1.584-5.037-3.711H.956v2.332A9 9 0 0 0 9 18Z"
      />
      <path
        fill="#FBBC05"
        d="M3.963 10.708A5.41 5.41 0 0 1 3.681 9c0-.593.102-1.17.282-1.708V4.96H.956A9 9 0 0 0 0 9c0 1.452.347 2.827.956 4.04l3.007-2.332Z"
      />
      <path
        fill="#EA4335"
        d="M9 3.58c1.321 0 2.507.455 3.441 1.346l2.581-2.581C13.464.892 11.426 0 9 0A9 9 0 0 0 .956 4.96l3.007 2.332C4.672 5.164 6.656 3.58 9 3.58Z"
      />
    </svg>
  );
}

function authErrorMessage(reason: unknown): string {
  if (
    typeof reason === "object" &&
    reason !== null &&
    "message_vi" in reason &&
    typeof reason.message_vi === "string"
  ) {
    return reason.message_vi;
  }
  return "Tro chưa thể đăng nhập bằng Google. Hãy thử lại.";
}
