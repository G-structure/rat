import { For } from "solid-js";
import { store } from "../state";

export function DiffView() {
  return (
    <div class="log" style="background:#0a0f10;">
      <For each={store.diffs()}>
        {(d) => (
          <div style="margin-bottom:12px;">
            <div style="color:#b6c2d6;">{d.path}</div>
            <pre style="white-space:pre-wrap; margin:6px 0;">
{d.diff}
            </pre>
          </div>
        )}
      </For>
    </div>
  );
}

