import { For } from "solid-js";
import { store } from "../state";

export function TerminalView() {
  const lines = () => {
    const id = store.activeSessionId();
    return id ? store.sessions()[id]?.terminal ?? [] : [];
  };
  return (
    <div class="log" style="background:#0a0f15;">
      <For each={lines()}>{(line) => <div>{line}</div>}</For>
    </div>
  );
}
