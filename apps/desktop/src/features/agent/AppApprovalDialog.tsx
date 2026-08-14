import { useEffect, useRef } from "react";
import type {
  ConfirmationDecision,
  ConfirmationRequest,
} from "../../lib/contracts";

export function AppApprovalDialog({
  request,
  onResolve,
}: {
  request: ConfirmationRequest;
  onResolve: (decision: ConfirmationDecision) => void;
}) {
  const stopRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    stopRef.current?.focus();
  }, [request.confirmation_id]);

  const appAccess = request.kind === "app_access";
  return (
    <main
      className="confirmation-card"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
      aria-describedby="confirm-consequence"
    >
      <span className="warning-mark" aria-hidden="true">
        {appAccess ? "↗" : "!"}
      </span>
      <div>
        <span className="eyebrow">
          {appAccess ? "Quyền ứng dụng của Tro" : "Cần bạn xác nhận"}
        </span>
        <h1 id="confirm-title">{request.action_vi}</h1>
        <p id="confirm-consequence">{request.consequence_vi}</p>
        <p className="app-context">
          <strong>{request.app_name}</strong>
          <span>{request.identity_summary}</span>
        </p>
        {appAccess ? (
          <p className="approval-boundary">
            Đây là quyền cục bộ, có thể thu hồi trong Cài đặt. Quyền này không
            thay thế Screen Recording/Accessibility và không cho phép gửi, xóa
            hay thay đổi bảo mật.
          </p>
        ) : null}
      </div>
      <div className="confirmation-actions">
        <button
          ref={stopRef}
          className="button quiet"
          onClick={() => {
            onResolve("stop");
          }}
        >
          Dừng
        </button>
        {appAccess && request.choices.includes("always_allow") ? (
          <button
            className="button subtle"
            onClick={() => {
              onResolve("always_allow");
            }}
          >
            Luôn cho phép ứng dụng này
          </button>
        ) : null}
        <button
          className="button primary"
          onClick={() => {
            onResolve("allow_once");
          }}
        >
          Chỉ lần này
        </button>
      </div>
    </main>
  );
}
