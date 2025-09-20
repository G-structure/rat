import { For } from "solid-js";
import { store } from "../state";

export function DiffView() {
  const diffs = () => {
    const id = store.activeSessionId();
    return id ? (store.sessions()[id]?.diffs ?? []) : [];
  };
  return (
    <div class="log" style="background:#0a0f10;">
      <For each={diffs()}>
        {(d) => (
          <div style="margin-bottom:12px;">
            <div style="color:#b6c2d6; font-weight:600;">{d.path}</div>
            <pre style="white-space:pre; overflow-x:auto; margin:6px 0;" class="scrollbar-thin">{d.diff}</pre>
          </div>
        )}
      </For>
      <For each={diffs().length === 0 ? [1] : []}>{() => (
        <div style="color:#6b7a90; padding:8px;">No diffs available yet.</div>
      )}</For>
    </div>
  );
}
