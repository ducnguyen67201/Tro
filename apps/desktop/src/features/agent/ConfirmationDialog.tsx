import { useEffect } from "react";
import { desktop } from "../../lib/tauri";
import type { ConfirmationDecision } from "../../lib/contracts";
import { AppApprovalDialog } from "./AppApprovalDialog";
import { useAgentStore } from "./agentStore";

export function ConfirmationDialog() {
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
  const resolve = (decision: ConfirmationDecision) => {
    void desktop.resolveConfirmation(confirmation.confirmation_id, decision);
    setConfirmation(null);
  };

  return <AppApprovalDialog request={confirmation} onResolve={resolve} />;
}
