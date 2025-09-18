import { For } from "solid-js";
import { store, setPlanItemStatusFor } from "../state";

export function PlanPanel() {
  const badge = (s: string) =>
    s === "completed" ? "#6de8a0" : s === "in_progress" ? "#ffcf6e" : "#6b7a90";
  const items = () => {
    const id = store.activeSessionId();
    return id ? store.sessions()[id]?.plan ?? [] : [];
  };
  const cycle = (sid: string, itemId: string, cur: string) => {
    const next = cur === "pending" ? "in_progress" : cur === "in_progress" ? "completed" : "pending";
    setPlanItemStatusFor(sid, itemId, next as any);
  };
  return (
    <div style="padding:12px 16px; border-left:1px solid #111826;">
      <h3 style="margin:0 0 8px 0; color:#b6c2d6; font-size:12px;">Plan</h3>
      <For each={items()}>{(it) => {
        const sid = store.activeSessionId()!;
        return (
          <button
            onClick={() => cycle(sid, it.id, it.status)}
            title="Click to cycle status"
            style="display:flex;gap:8px;align-items:center;padding:6px 8px; width:100%; text-align:left; background:#0f141b; border:1px solid #1a2130; border-radius:6px; margin:4px 0;"
          >
            <span style={{ width: "8px", height: "8px", "border-radius": "50%", background: badge(it.status) }} />
            <span>{it.title}</span>
            <span style="margin-left:auto; color:#6b7a90; font-size:12px;">{it.status.replace('_',' ')}</span>
          </button>
        );
      }}</For>
    </div>
  );
}
