import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { desktop } from "../../lib/tauri";
import { useAgentStore } from "./agentStore";

export function ConfirmationDialog() {
  const { t } = useTranslation();
  const confirmation = useAgentStore((state) => state.confirmation);
  const setConfirmation = useAgentStore((state) => state.setConfirmation);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    void desktop.onConfirmation(setConfirmation).then((unlisten) => {
      dispose = unlisten;
    });
    return () => {
      dispose?.();
    };
  }, [setConfirmation]);

  if (!confirmation) return <div className="confirmation-empty" />;
  const resolve = (decision: "allow_once" | "stop") => {
    void desktop.resolveConfirmation(confirmation.confirmation_id, decision);
    setConfirmation(null);
  };

  return (
    <main
      className="confirmation-card"
      role="alertdialog"
      aria-modal="true"
      aria-labelledby="confirm-title"
    >
      <span className="warning-mark">!</span>
      <div>
        <span className="eyebrow">Cần bạn xác nhận</span>
        <h1 id="confirm-title">{confirmation.action_vi}</h1>
        <p>{confirmation.consequence_vi}</p>
        <p className="app-context">Trong ứng dụng: {confirmation.app_name}</p>
      </div>
      <div className="confirmation-actions">
        <button
          className="button quiet"
          onClick={() => {
            resolve("stop");
          }}
        >
          {t("cancel")}
        </button>
        <button
          className="button primary"
          onClick={() => {
            resolve("allow_once");
          }}
        >
          {t("allow")}
        </button>
      </div>
    </main>
  );
}
