import { createSignal, createRoot } from "solid-js";

export type ChatState = "idle" | "generating" | "completed";

interface ProjectChatState {
  projectId: string;
  state: ChatState;
  lastPrompt?: string;
  timestamp: number;
}

interface ChatHistory {
  projectId: string;
  prompts: Array<{
    prompt: string;
    response?: string;
    timestamp: number;
  }>;
}

function createChatStore() {
  const [projectStates, setProjectStates] = createSignal<Map<string, ProjectChatState>>(new Map());
  const [chatHistories, setChatHistories] = createSignal<Map<string, ChatHistory>>(new Map());

  const updateProjectState = (projectId: string, state: ChatState, lastPrompt?: string) => {
    setProjectStates(prev => {
      const newMap = new Map(prev);
      newMap.set(projectId, {
        projectId,
        state,
        lastPrompt,
        timestamp: Date.now()
      });
      return newMap;
    });
  };

  const getProjectState = (projectId: string): ChatState => {
    const state = projectStates().get(projectId);
    return state?.state || "idle";
  };

  const addPromptToHistory = (projectId: string, prompt: string, response?: string) => {
    setChatHistories(prev => {
      const newMap = new Map(prev);
      const existing = newMap.get(projectId) || { projectId, prompts: [] };
      existing.prompts.push({
        prompt,
        response,
        timestamp: Date.now()
      });
      newMap.set(projectId, existing);
      return newMap;
    });
  };

  const getProjectHistory = (projectId: string): ChatHistory | undefined => {
    return chatHistories().get(projectId);
  };

  const clearProjectState = (projectId: string) => {
    setProjectStates(prev => {
      const newMap = new Map(prev);
      newMap.delete(projectId);
      return newMap;
    });
  };

  return {
    projectStates,
    updateProjectState,
    getProjectState,
    addPromptToHistory,
    getProjectHistory,
    clearProjectState
  };
}

export const chatStore = createRoot(createChatStore);