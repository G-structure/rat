import { For } from "solid-js";
import { store } from "../state";

export function ChatView() {
  const current = () => {
    const id = store.activeSessionId();
    return id ? store.sessions()[id]?.messages ?? [] : [];
  };
  return (
    <div class="log" style="flex:1;">
      <For each={current()}>
        {(m) => (
          <div>
            <span style="color:#6b7a90;">[{m.from}]</span> {m.text}
          </div>
        )}
      </For>
    </div>
  );
}
