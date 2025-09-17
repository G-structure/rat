import { For, Show } from "solid-js";
import { store, dequeuePermission, PermissionReq } from "../state";
import { sendPermissionCancelled, sendPermissionSelected } from "../lib/ws";

export function PermissionDialog() {
  const chooseAllow = (p: PermissionReq) => {
    // Prefer explicit allow_always/allow ids when present
    const opts = p.options || [];
    const allowAlways = opts.find((o) => o.id === "allow_always");
    const allowOnce = opts.find((o) => o.id === "allow");
    const fallback = opts.find((o) => /allow|yes/i.test(o.id) || /allow|yes/i.test(o.label || "")) || opts[0];
    const optionId = (allowOnce?.id) || (allowAlways?.id) || (fallback?.id) || "allow";
    sendPermissionSelected(p.rid, optionId);
    dequeuePermission(p.id);
  };
  const chooseDeny = (p: PermissionReq) => {
    // Prefer explicit reject ids if present
    const opts = p.options || [];
    const rejectAlways = opts.find((o) => o.id === "reject_always");
    const rejectOnce = opts.find((o) => o.id === "reject" || /deny/i.test(o.id));
    const optionId = (rejectOnce?.id) || (rejectAlways?.id);
    if (optionId) {
      sendPermissionSelected(p.rid, optionId);
    } else {
      sendPermissionCancelled(p.rid);
    }
    dequeuePermission(p.id);
  };
  return (
    <Show when={store.permissions().length > 0}>
      <div style="position:fixed; inset:0; background:rgba(0,0,0,0.5); display:flex; align-items:center; justify-content:center;">
        <div style="background:#0f141b; border:1px solid #1a2130; border-radius:8px; min-width:420px; max-width:600px; padding:16px;">
          <h3 style="margin:0 0 8px 0; color:#b6c2d6; font-size:14px;">Permission requested</h3>
          <For each={store.permissions()}>
            {(p) => (
              <div style="padding:8px 0; border-top:1px solid #111826;">
                <div style="color:#c7d2e8;">Tool: {p.tool}</div>
                <div style="color:#6b7a90; font-size:12px;">{p.reason ?? "The agent requested permission to run a tool."}</div>
                <div style="margin-top:8px; display:flex; gap:8px; justify-content:flex-end;">
                  <button onClick={() => chooseDeny(p)} style="background:#2b1a1a; border-color:#3a2222;">Deny</button>
                  <button onClick={() => chooseAllow(p)} style="background:#142a1f; border-color:#1f3a2b;">Allow</button>
                </div>
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}
