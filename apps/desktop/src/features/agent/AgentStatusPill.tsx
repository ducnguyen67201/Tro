import { useState } from "react";
import { desktop } from "../../lib/tauri";
import { useAssistantStore } from "../assistant/assistantStore";
import { useAgentStore } from "./agentStore";

const labels: Record<string, string> = {
  idle: "Agent đang tắt",
  planning: "Đang lập kế hoạch",
  awaiting_confirmation: "Đang chờ bạn xác nhận",
  executing: "Đang thao tác",
  observing: "Đang kiểm tra kết quả",
  completed: "Đã hoàn thành",
  stopped: "Đã dừng",
  failed: "Không thể tiếp tục",
};

export function AgentStatusPill() {
  const agent = useAssistantStore((state) => state.agent);
  const goal = useAgentStore((state) => state.goal);
  const setGoal = useAgentStore((state) => state.setGoal);
  const [open, setOpen] = useState(false);

  if (agent !== "idle") {
    return (
      <div className="agent-status" role="status">
        <span className="status-dot" />
        {labels[agent]}
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
