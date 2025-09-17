import { createSignal } from "solid-js";

export type ChatMessage = { from: "user" | "agent" | "system"; text: string; ts: string };
export type PlanItem = { id: string; title: string; status: "pending" | "in_progress" | "completed" };
export type PermissionOption = { id: string; label?: string };
export type PermissionReq = { id: string; rid: string | number; tool: string; reason?: string; options?: PermissionOption[] };
export type DiffItem = { path: string; diff: string };
export type Command = { name: string; description?: string };

export type SessionState = {
  messages: ChatMessage[];
  plan: PlanItem[];
  terminal: string[];
  commands: Command[];
  availableModes: string[];
  currentMode: string | null;
  diffs: DiffItem[];
};

const [sessions, setSessions] = createSignal<Record<string, SessionState>>({});
const [activeSessionId, setActiveSessionId] = createSignal<string | null>(null);
const [permissions, setPermissions] = createSignal<PermissionReq[]>([]);

const STORAGE_KEY = "rat.sessions";
const STORAGE_ACTIVE_KEY = "rat.activeSession";

export const store = {
  sessions,
  setSessions,
  activeSessionId,
  setActiveSessionId,
  permissions,
  setPermissions,
};

export function ensureSession(id: string) {
  setSessions((prev) => {
    if (prev[id]) return prev;
    return {
      ...prev,
      [id]: {
        messages: [],
        plan: [],
        terminal: [],
        commands: [],
        availableModes: [],
        currentMode: null,
        diffs: [],
      },
    };
  });
}

export function pushMessageFor(id: string, msg: ChatMessage) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), messages: [ ...(prev[id]?.messages ?? []), msg ] },
  }));
}
export function pushTerminalFor(id: string, line: string) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), terminal: [ ...(prev[id]?.terminal ?? []), line ] },
  }));
}
export function upsertPlanFor(id: string, items: PlanItem[]) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), plan: items },
  }));
}
export function setCommandsFor(id: string, cmds: Command[]) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), commands: cmds },
  }));
}
export function setModesFor(id: string, modes: string[]) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), availableModes: modes },
  }));
}
export function setCurrentModeFor(id: string, mode: string | null) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), currentMode: mode },
  }));
}
export function setDiffsFor(id: string, diffs: DiffItem[]) {
  setSessions((prev) => ({
    ...prev,
    [id]: { ...(prev[id] ?? { messages: [], plan: [], terminal: [], commands: [], availableModes: [], currentMode: null, diffs: [] }), diffs },
  }));
}

export function enqueuePermission(p: PermissionReq) {
  setPermissions((prev) => [...prev, p]);
}
export function dequeuePermission(id: string) {
  setPermissions((prev) => prev.filter((x) => x.id !== id));
}

export function persistSessionList() {
  try {
    const ids = Object.keys(sessions());
    localStorage.setItem(STORAGE_KEY, JSON.stringify(ids));
    const active = activeSessionId();
    if (active) localStorage.setItem(STORAGE_ACTIVE_KEY, active);
  } catch {}
}

export function loadSessionList(): { ids: string[]; active: string | null } {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    const ids: string[] = raw ? JSON.parse(raw) : [];
    const active = localStorage.getItem(STORAGE_ACTIVE_KEY);
    return { ids, active };
  } catch {
    return { ids: [], active: null };
  }
}

export function selectActiveSession(id: string) {
  setActiveSessionId(id);
  persistSessionList();
}

export function removeSession(id: string) {
  setSessions((prev) => {
    const next = { ...prev } as Record<string, SessionState>;
    delete next[id];
    return next;
  });
  // If removing the active session, pick another or clear
  if (activeSessionId() === id) {
    const ids = Object.keys(sessions());
    setActiveSessionId(ids[0] ?? null);
  }
  persistSessionList();
}
