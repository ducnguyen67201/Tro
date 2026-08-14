import { useCallback, useEffect, useState } from "react";
import type { ApplicationRef } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";

export function ApprovedAppsSettings() {
  const [apps, setApps] = useState<ApplicationRef[]>([]);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(() => {
    let active = true;
    void desktop
      .approvedApps()
      .then((approved) => {
        if (active) {
          setApps(approved);
          setError(null);
        }
      })
      .catch(() => {
        if (active) setError("Chưa tải được danh sách ứng dụng.");
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => refresh(), [refresh]);

  const revoke = (app: ApplicationRef) => {
    void desktop
      .revokeApprovedApp(app.app_id)
      .then(() => {
        setApps((current) =>
          current.filter((candidate) => candidate.app_id !== app.app_id),
        );
      })
      .catch(() => {
        setError("Chưa thể thu hồi quyền ứng dụng.");
      });
  };

  return (
    <section className="settings-section" aria-labelledby="approved-apps-title">
      <div className="settings-section-heading">
        <div>
          <h2 id="approved-apps-title">Ứng dụng luôn được cho phép</h2>
          <p>
            Đây là quyền cục bộ của Tro, tách biệt với Screen
            Recording/Accessibility. Quyền này không duyệt thao tác gửi, xóa
            hoặc bảo mật.
          </p>
        </div>
      </div>
      {error ? <p role="alert">{error}</p> : null}
      {apps.length === 0 ? (
        <p className="settings-empty">Chưa có ứng dụng nào.</p>
      ) : (
        <ul className="approved-app-list">
          {apps.map((app) => (
            <li key={app.app_id}>
              <span>
                <strong>{app.display_name}</strong>
                <small>{app.identity_summary}</small>
              </span>
              <button
                className="button quiet"
                onClick={() => {
                  revoke(app);
                }}
              >
                Thu hồi
              </button>
            </li>
          ))}
        </ul>
      )}
      <small className="platform-limit">
        macOS: Tro có thể quan sát tốt nhất cửa sổ ứng dụng đã duyệt ở nền.
        Windows: ứng dụng mục tiêu phải hiển thị và ở phía trước.
      </small>
    </section>
  );
}
