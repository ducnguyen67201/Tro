import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import type { PermissionSnapshot } from "../../lib/contracts";
import { desktop } from "../../lib/tauri";
import { PermissionCard } from "./PermissionCard";

export function Onboarding({ onComplete }: { onComplete: () => void }) {
  const { t } = useTranslation();
  const [step, setStep] = useState<"welcome" | "permissions">("welcome");
  const [invite, setInvite] = useState("");
  const [age, setAge] = useState(false);
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

  if (step === "welcome") {
    return (
      <main className="onboarding-shell">
        <section className="onboarding-hero">
          <div className="brand-lockup">
            <span className="brand-mark">T</span>
            <span>Tro</span>
          </div>
          <span className="pill">Dành cho sinh viên Việt Nam</span>
          <h1>
            Đừng chỉ nhận đáp án.
            <br />
            <em>Hãy hiểu cách làm.</em>
          </h1>
          <p>
            Tro nhìn phần màn hình bạn chọn, nghe câu hỏi bằng tiếng Việt và
            hướng dẫn từng bước ngay trên ứng dụng đang dùng.
          </p>
          <ul className="trust-list">
            <li>
              <span>✓</span> Chỉ chụp màn hình khi bạn bấm hỏi
            </li>
            <li>
              <span>✓</span> Không lưu ảnh, âm thanh hay bài làm
            </li>
            <li>
              <span>✓</span> Bạn luôn có nút dừng khẩn cấp
            </li>
          </ul>
        </section>
        <section className="invite-card">
          <span className="eyebrow">Bản thử nghiệm riêng tư</span>
          <h2>Bắt đầu với Tro</h2>
          <label htmlFor="invite">{t("invite")}</label>
          <input
            id="invite"
            value={invite}
            onChange={(event) => {
              setInvite(event.target.value);
            }}
            placeholder="TRO-••••-••••"
            autoComplete="off"
          />
          <label className="check-row">
            <input
              type="checkbox"
              checked={age}
              onChange={(event) => {
                setAge(event.target.checked);
              }}
            />
            <span>{t("age")}</span>
          </label>
          <button
            className="button primary wide"
            disabled={invite.trim().length < 4 || !age}
            onClick={() => {
              setStep("permissions");
            }}
          >
            {t("continue")} <span>→</span>
          </button>
          <p className="fine-print">
            Bằng việc tiếp tục, bạn đồng ý với thông báo quyền riêng tư của
            chương trình thử nghiệm.
          </p>
        </section>
      </main>
    );
  }

  return (
    <main className="permission-shell">
      <div className="brand-lockup">
        <span className="brand-mark">T</span>
        <span>Tro</span>
      </div>
      <span className="eyebrow">Bước 2 / 2</span>
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
