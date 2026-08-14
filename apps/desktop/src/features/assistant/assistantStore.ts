import { create } from "zustand";
import type { AppSnapshot } from "../../lib/contracts";

interface AssistantStore extends AppSnapshot {
  setSnapshot: (snapshot: AppSnapshot) => void;
}

export const useAssistantStore = create<AssistantStore>((set) => ({
  assistant: "idle",
  agent: "idle",
  transcript: null,
  status_vi: "Sẵn sàng",
  capture_active: false,
  scoped_app_name: null,
  agent_choices: [],
  setSnapshot: (snapshot) => {
    set(snapshot);
  },
}));
