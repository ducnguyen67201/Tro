import { create } from "zustand";
import type { ConfirmationRequest } from "../../lib/contracts";

interface AgentStore {
  goal: string;
  confirmation: ConfirmationRequest | null;
  setGoal: (goal: string) => void;
  setConfirmation: (confirmation: ConfirmationRequest | null) => void;
}

export const useAgentStore = create<AgentStore>((set) => ({
  goal: "",
  confirmation: null,
  setGoal: (goal) => {
    set({ goal });
  },
  setConfirmation: (confirmation) => {
    set({ confirmation });
  },
}));
