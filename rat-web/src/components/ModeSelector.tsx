import { For } from "solid-js";
import { store, setCurrentModeFor } from "../state";
import { useAcpWs } from "../lib/ws";

export function ModeSelector() {
  const { sendJson } = useAcpWs();
  const change = (mode: string) => {
    const sid = store.activeSessionId();
    if (!sid) return;
    sendJson({ jsonrpc: "2.0", id: Math.floor(Math.random() * 1e9), method: "session/set_mode", params: { sessionId: sid, mode } });
    // optimistic
    setCurrentModeFor(sid, mode);
  };
  return (
    <div style="padding:12px 16px; border-right:1px solid #111826; min-width:220px;">
      <h3 style="margin:0 0 8px 0; color:#b6c2d6; font-size:12px;">Mode</h3>
      <div style="color:#c7d2e8; margin-bottom:6px;">Current: {(() => { const id = store.activeSessionId(); return id ? (store.sessions()[id]?.currentMode ?? "(unknown)") : "(unknown)"; })()}</div>
      <For each={( () => { const id = store.activeSessionId(); return id ? (store.sessions()[id]?.availableModes ?? []) : []; })()}>
        {(m) => (
          <button style="display:block; width:100%; margin:4px 0;" onClick={() => change(m)}>{m}</button>
        )}
      </For>
    </div>
  );
}
