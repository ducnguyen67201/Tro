import { useState } from "react";
import { desktop } from "../../lib/tauri";
import { useAssistantStore } from "../assistant/assistantStore";
import { useAgentStore } from "./agentStore";

const labels: Record<string, string> = {
  idle: "Agent đang tắt",
  resolving_app: "Đang tìm ứng dụng",
  awaiting_app_approval: "Đang chờ cho phép ứng dụng",
  activating_app: "Đang mở ứng dụng",
  planning: "Đang lập kế hoạch",
  validating: "Đang kiểm tra mục tiêu",
  awaiting_confirmation: "Đang chờ bạn xác nhận",
  executing: "Đang thao tác",
  stabilizing: "Đang chờ ứng dụng ổn định",
  observing: "Đang kiểm tra kết quả",
  stale_recovery: "Giao diện đổi — đang quan sát lại",
  needs_user: "Cần bạn hỗ trợ",
  paused_by_user: "Bạn đã tiếp quản",
  completed: "Đã hoàn thành",
  stopped: "Đã dừng",
  failed: "Không thể tiếp tục",
};

export function AgentStatusPill() {
  const agent = useAssistantStore((state) => state.agent);
  const goal = useAgentStore((state) => state.goal);
  const setGoal = useAgentStore((state) => state.setGoal);
  const scopedAppName = useAssistantStore((state) => state.scoped_app_name);
  const choices = useAssistantStore((state) => state.agent_choices ?? []);
  const [open, setOpen] = useState(false);

  if (agent !== "idle") {
    return (
      <div className="agent-status" role="status">
        <span className="status-dot" />
        <span>
          {labels[agent]}
          {scopedAppName ? ` · ${scopedAppName}` : ""}
        </span>
        {agent === "needs_user" && choices.length > 0 ? (
          <span className="agent-choice-summary" aria-label="Chọn ứng dụng">
            {choices.map((app) => (
              <button
                key={app.app_id}
                className="button subtle"
                onClick={() => {
                  if (goal.trim().length >= 3) {
                    void desktop.startAgentForApp(goal.trim(), app.app_id);
                  }
                }}
              >
                {app.display_name} · {app.identity_summary}
              </button>
            ))}
          </span>
        ) : null}
        {agent === "paused_by_user" && goal.trim().length >= 3 ? (
          <button
            className="button subtle"
            onClick={() => {
              void desktop.startAgent(goal.trim());
            }}
          >
            Tiếp tục từ quan sát mới
          </button>
        ) : null}
        <button
          onClick={() => {
            void desktop.emergencyStop();
          }}
        >
          Dừng
        </button>
      </div>
    );
  }

  if (!open) {
    return (
      <button
        className="button subtle"
        onClick={() => {
          setOpen(true);
        }}
      >
        Làm cùng mình
      </button>
    );
  }

  return (
    <form
      className="agent-goal"
      onSubmit={(event) => {
        event.preventDefault();
        if (goal.trim().length >= 3) void desktop.startAgent(goal.trim());
      }}
    >
      <label htmlFor="agent-goal">Bạn muốn Tro làm gì?</label>
      <textarea
        id="agent-goal"
        value={goal}
        maxLength={500}
        onChange={(event) => {
          setGoal(event.target.value);
        }}
        placeholder="Ví dụ: Mở tài liệu này và giúp mình định dạng tiêu đề"
      />
      <p>
        Tro sẽ báo trước các thao tác có hậu quả. Mật khẩu, thanh toán và bài
        thi luôn bị chặn.
      </p>
      <div className="row end">
        <button
          type="button"
          className="button quiet"
          onClick={() => {
            setOpen(false);
          }}
        >
          Hủy
        </button>
        <button type="submit" className="button primary">
          Bắt đầu có kiểm soát
        </button>
      </div>
    </form>
  );
}
