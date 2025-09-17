import { For } from "solid-js";
import { store } from "../state";

export function CommandsPanel() {
  const cmds = () => {
    const id = store.activeSessionId();
    return id ? store.sessions()[id]?.commands ?? [] : [];
  };
  return (
    <div style="padding:12px 16px; border-right:1px solid #111826; min-width:220px;">
      <h3 style="margin:0 0 8px 0; color:#b6c2d6; font-size:12px;">Commands</h3>
      <For each={cmds()}>
        {(c) => (
          <div style="padding:4px 0;">
            <div style="color:#c7d2e8;">/{c.name}</div>
            <div style="color:#6b7a90; font-size:12px;">{c.description}</div>
          </div>
        )}
      </For>
      <div style="color:#6b7a90; font-size:12px; margin-top:8px;">(updates via session/update)</div>
    </div>
  );
}
