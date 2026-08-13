import { useState, type FormEvent } from "react";
import { desktop } from "../../lib/tauri";

interface SignInProps {
  checking?: boolean;
  onAuthenticated: () => void;
}

export function SignIn({ checking = false, onAuthenticated }: SignInProps) {
  const [invite, setInvite] = useState("");
  const [ageAccepted, setAgeAccepted] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState("");

  const submit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (checking || submitting) return;
    setSubmitting(true);
    setError("");
    void desktop
      .signIn(invite, ageAccepted)
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
          Đăng nhập một lần để gọi Tro ở bất kỳ ứng dụng nào bằng ⌘ + ⌥.
        </p>

        {checking ? (
          <div className="auth-checking" role="status">
            <span aria-hidden="true" /> Đang kiểm tra phiên đăng nhập…
          </div>
        ) : (
          <form className="auth-form" onSubmit={submit}>
            <label htmlFor="invite">Mã truy cập</label>
            <input
              id="invite"
              value={invite}
              onChange={(event) => {
                setInvite(event.target.value.toUpperCase());
              }}
              placeholder="TRO-XXXX"
              autoCapitalize="characters"
              autoComplete="one-time-code"
              autoFocus
              aria-describedby={error ? "auth-error" : "auth-code-help"}
              aria-invalid={Boolean(error)}
            />
            <span id="auth-code-help" className="auth-field-help">
              Mã được cấp bởi nhóm Tro cho bản thử nghiệm.
            </span>
            <label className="auth-consent">
              <input
                type="checkbox"
                checked={ageAccepted}
                onChange={(event) => {
                  setAgeAccepted(event.target.checked);
                }}
              />
              <span>Tôi là sinh viên đại học và đã đủ 18 tuổi.</span>
            </label>
            {error ? (
              <p id="auth-error" className="auth-error" role="alert">
                {error}
              </p>
            ) : null}
            <button
              className="button primary wide auth-submit"
              type="submit"
              disabled={invite.trim().length < 4 || !ageAccepted || submitting}
            >
              {submitting ? "Đang đăng nhập…" : "Tiếp tục"}
              {!submitting ? <span aria-hidden="true">→</span> : null}
            </button>
          </form>
        )}

        <p className="auth-privacy">
          Phiên đăng nhập được lưu an toàn trên máy. Ảnh và âm thanh chỉ được
          gửi khi bạn chủ động gọi Tro.
        </p>
      </section>
      <p className="auth-footer">Vietnamese-first · Riêng tư theo mặc định</p>
    </main>
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
  return "Tro chưa thể đăng nhập. Hãy kiểm tra mã và thử lại.";
}
