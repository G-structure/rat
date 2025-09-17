import { For } from "solid-js";
import { store } from "../state";

export function PlanPanel() {
  const badge = (s: string) =>
    s === "completed" ? "#6de8a0" : s === "in_progress" ? "#ffcf6e" : "#6b7a90";
  const items = () => {
    const id = store.activeSessionId();
    return id ? store.sessions()[id]?.plan ?? [] : [];
  };
  return (
    <div style="padding:12px 16px; border-left:1px solid #111826;">
      <h3 style="margin:0 0 8px 0; color:#b6c2d6; font-size:12px;">Plan</h3>
      <For each={items()}>
        {(it) => (
          <div style="display:flex;gap:8px;align-items:center;padding:4px 0;">
            <span style={{ width: "8px", height: "8px", "border-radius": "50%", background: badge(it.status) }} />
            <span>{it.title}</span>
          </div>
        )}
      </For>
    </div>
  );
}
