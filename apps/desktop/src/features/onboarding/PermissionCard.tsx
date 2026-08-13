import type { PermissionStatus } from "../../lib/contracts";

export function PermissionCard({
  icon,
  title,
  detail,
  status,
  requesting,
  onRequest,
}: {
  icon: string;
  title: string;
  detail: string;
  status: PermissionStatus;
  requesting?: boolean;
  onRequest: () => void;
}) {
  return (
    <article className="permission-card">
      <span className="permission-icon" aria-hidden="true">
        {icon}
      </span>
      <div>
        <h3>{title}</h3>
        <p>{detail}</p>
      </div>
      {status === "granted" ? (
        <span className="permission-ok">Đã cho phép</span>
      ) : (
        <button
          className="button quiet"
          disabled={requesting}
          onClick={onRequest}
        >
          {requesting ? "Đang mở…" : "Cho phép"}
        </button>
      )}
    </article>
  );
}
