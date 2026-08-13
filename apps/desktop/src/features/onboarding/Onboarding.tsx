import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { PermissionSnapshot } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";
import { PermissionCard } from "./PermissionCard";

export function Onboarding({ onComplete }: { onComplete: () => void }) {
  const { t } = useTranslation();
  const [requesting, setRequesting] = useState<keyof PermissionSnapshot | null>(
    null,
  );
  const [permissions, setPermissions] = useState<PermissionSnapshot>({
    microphone: "not_determined",
    screen_capture: "not_determined",
    input_control: "not_determined",
  });

  const refreshPermissions = useCallback(() => {
    void desktop
      .permissions()
      .then(setPermissions)
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    refreshPermissions();
    window.addEventListener("focus", refreshPermissions);
    document.addEventListener("visibilitychange", refreshPermissions);
    return () => {
      window.removeEventListener("focus", refreshPermissions);
      document.removeEventListener("visibilitychange", refreshPermissions);
    };
  }, [refreshPermissions]);

  const request = (permission: keyof PermissionSnapshot) => {
    if (permissions[permission] === "restart_required") {
      void desktop.restart();
      return;
    }
    setRequesting(permission);
    void desktop
      .requestPermission(permission)
      .then(setPermissions)
      .catch(refreshPermissions)
      .finally(() => {
        setRequesting(null);
      });
  };

  return (
    <main className="permission-shell">
      <div className="brand-lockup">
        <span className="brand-mark">T</span>
        <span>Tro</span>
      </div>
      <span className="eyebrow">Thiết lập một lần</span>
      <h1>{t("permissions")}</h1>
      <p>
        Tro hoạt động theo nguyên tắc hỏi trước, dùng sau. Điều khiển nhập liệu
        có thể bật sau khi bạn chủ động dùng agent. Sau bước này, Tro sẽ ở trên
        thanh menu; giữ ⌘ + ⌥ để gọi Tro cạnh con trỏ.
      </p>
      <div className="permission-grid">
        <PermissionCard
          icon="◉"
          title={t("microphone")}
          detail="Nghe câu hỏi khi bạn giữ phím tắt"
          status={permissions.microphone}
          requesting={requesting === "microphone"}
          onRequest={() => {
            request("microphone");
          }}
        />
        <PermissionCard
          icon="▣"
          title={t("screen")}
          detail="Chụp đúng màn hình sau khi bạn yêu cầu"
          status={permissions.screen_capture}
          requesting={requesting === "screen_capture"}
          onRequest={() => {
            request("screen_capture");
          }}
        />
        <PermissionCard
          icon="↖"
          title="Phím tắt & điều khiển"
          detail="Nhận ⌘ + ⌥ và cho agent thao tác sau khi bạn xác nhận"
          status={permissions.input_control}
          requesting={requesting === "input_control"}
          onRequest={() => {
            request("input_control");
          }}
        />
      </div>
      <button className="button primary" onClick={onComplete}>
        Hoàn tất <span>→</span>
      </button>
      <button className="button quiet" onClick={onComplete}>
        {t("later")}
      </button>
    </main>
  );
}
