import { createSignal } from "solid-js";
import {
  store,
  ensureSession,
  pushMessageFor,
  pushTerminalFor,
  upsertPlanFor,
  setCommandsFor,
  setModesFor,
  setCurrentModeFor,
  setDiffsFor,
  loadSessionList,
  persistSessionList,
} from "../state";

export type WsState = "idle" | "connecting" | "open" | "closed";

let wsGlobal: WebSocket | null = null;
const pendingPerms = new Map<string | number, { options?: { id: string; label?: string }[] }>();

export function sendPermissionSelected(rid: string | number, optionId: string) {
  if (!wsGlobal || wsGlobal.readyState !== WebSocket.OPEN) return;
  const resp = { jsonrpc: "2.0", id: rid, result: { outcome: { selected: { optionId } } } };
  wsGlobal.send(JSON.stringify(resp));
  // best-effort log for visibility
  try {
    const time = new Date().toISOString();
    const line = `→ ${time} permission response id=${rid} optionId=${optionId}`;
    // eslint-disable-next-line no-console
    console.log(line);
  } catch {}
}

export function sendPermissionCancelled(rid: string | number) {
  if (!wsGlobal || wsGlobal.readyState !== WebSocket.OPEN) return;
  const resp = { jsonrpc: "2.0", id: rid, result: { outcome: { cancelled: true } } };
  wsGlobal.send(JSON.stringify(resp));
}

export function useAcpWs(defaultUrl = `ws://127.0.0.1:8081`) {
  const [state, setState] = createSignal<WsState>("idle");
  const [log, setLog] = createSignal<string[]>([]);
  const [sessionId, setSessionId] = createSignal<string | null>(null);
  let ws: WebSocket | null = null;
  let nextId = 1;

  function push(kind: "info" | "tx" | "rx" | "err", data: string) {
    const time = new Date().toISOString();
    const prefix = kind === "tx" ? "→" : kind === "rx" ? "←" : "•";
    setLog((prev) => [...prev, `${prefix} ${time} ${data}`]);
  }

  function connect(url = defaultUrl) {
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) ws.close();
    setState("connecting");
    push("info", `connecting ${url} …`);
    try {
      ws = new WebSocket(url, "acp.jsonrpc.v1");
      wsGlobal = ws;
    } catch (e: any) {
      push("err", String(e));
      setState("closed");
      return;
    }
    ws.onopen = () => {
      setState("open");
      push("info", "connected");
      const init = {
        jsonrpc: "2.0",
        id: nextId++,
        method: "initialize",
        params: {
          protocolVersion: 1,
          clientCapabilities: { fs: { readTextFile: true, writeTextFile: true }, terminal: false }
        }
      };
      ws!.send(JSON.stringify(init));
      push("tx", JSON.stringify(init));
      // resume known sessions
      const { ids, active } = loadSessionList();
      ids.forEach((sid) => {
        ensureSession(sid);
        const loadMsg = { jsonrpc: "2.0", id: nextId++, method: "session/load", params: { sessionId: sid } };
        ws!.send(JSON.stringify(loadMsg));
        push("tx", JSON.stringify(loadMsg));
      });
      if (active) {
        store.setActiveSessionId(active);
      }
    };
    ws.onmessage = (ev) => {
      const text = typeof ev.data === "string" ? ev.data : "[binary]";
      push("rx", text);
      try {
        const v = JSON.parse(text);
        const res = v?.result;
        const sid = res?.sessionId;
        if (typeof sid === "string" && sid.length > 0) {
          setSessionId(sid);
          ensureSession(sid);
          store.setActiveSessionId(sid);
          persistSessionList();
          // available modes on session/new result
          const modes: string[] | undefined = res.availableModes || res.modes;
          if (Array.isArray(modes)) setModesFor(sid, modes);
          const cur = res.currentMode;
          if (typeof cur === "string") setCurrentModeFor(sid, cur);
        }
        // route updates to proper session and derive clean chat/plan/terminal
        const method = v?.method as string | undefined;
        const params = v?.params ?? {};
        const sid2 = params.sessionId ?? v?.sessionId ?? sid ?? store.activeSessionId();
        const sessId = typeof sid2 === "string" ? sid2 : (sid ?? store.activeSessionId());
        if (sessId) ensureSession(sessId);

        const upd = params.update ?? v?.update;
        if (method === "session/update" || upd) {
          // Message chunks → clean chat entries
          const kind = upd?.sessionUpdate ?? params?.sessionUpdate;
          const content = upd?.content ?? params?.content;
          if (kind === "agent_message_chunk" && content?.type === "text" && content?.text) {
            pushMessageFor(sessId!, { from: "agent", text: String(content.text), ts: new Date().toISOString() });
          }
          if (kind === "user_message_chunk" && content?.type === "text" && content?.text) {
            pushMessageFor(sessId!, { from: "user", text: String(content.text), ts: new Date().toISOString() });
          }
          // Plan
          const plan = params?.plan ?? upd?.plan;
          if (plan && Array.isArray(plan?.items)) {
            const items = plan.items.map((it: any, i: number) => ({ id: String(it.id ?? i), title: String(it.title ?? it.name ?? "step"), status: (it.status ?? "pending") as any }));
            upsertPlanFor(sessId!, items);
          }
          // Terminal
          const term = params?.terminalOutput ?? upd?.terminalOutput;
          if (term) {
            const line = typeof term === "string" ? term : JSON.stringify(term);
            pushTerminalFor(sessId!, line);
          }
          // Commands
          const cmds = params?.availableCommands ?? upd?.availableCommands;
          if (Array.isArray(cmds)) setCommandsFor(sessId!, cmds.map((c: any) => ({ name: String(c.name ?? ""), description: c.description })));
          // Modes
          const mode = params?.currentMode ?? upd?.currentMode;
          if (typeof mode === "string") setCurrentModeFor(sessId!, mode);
          const modes2 = params?.availableModes ?? upd?.availableModes;
          if (Array.isArray(modes2)) setModesFor(sessId!, modes2.map((m: any) => String(m)));
          // Diffs
          const diff = params?.diff ?? upd?.diff;
          if (diff) {
            const path = diff.path || diff.file || "changes";
            setDiffsFor(sessId!, [ { path: String(path), diff: String(diff.text ?? diff.patch ?? JSON.stringify(diff)) } ]);
          }
        }
        // Heuristic match for permission request method naming
        const isPermReq = (!!method && /request[_-]?permission/i.test(method)) || (!!(v?.id) && (params?.tool || params?.toolCall) && Array.isArray(params?.options));
        if (isPermReq) {
          const rid = (v?.id ?? params?.id) as string | number;
          const idp = String(rid ?? Math.random());
          const tool = params?.tool ?? params?.toolCall?.tool ?? params?.name ?? "tool";
          const reason = params?.reason ?? params?.text;
          const options = Array.isArray(params?.options)
            ? (params.options as any[]).map((o) => ({ id: String(o.optionId ?? o.id ?? o.name ?? ""), label: o.name ?? o.label }))
            : undefined;
          pendingPerms.set(rid, { options });
          // lazy import to avoid circular
          // @ts-ignore
          import("../state").then(({ enqueuePermission }) => enqueuePermission({ id: idp, rid, tool: String(tool), reason, options }));
        }
        if (method && method.startsWith("terminal/")) {
          const t = JSON.stringify(params ?? {});
          if (sessId) pushTerminalFor(sessId, `${method}: ${t}`);
        }
      } catch (_) {
        // ignore parse errors for non-JSON frames
      }
    };
    ws.onclose = (ev) => {
      setState("closed");
      push("info", `closed (${ev.code})`);
    };
    ws.onerror = () => push("err", "ws error");
  }

  function disconnect() {
    ws?.close();
  }

  function sendRaw(json: string) {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      push("err", "not connected");
      return;
    }
    let minified = json;
    try {
      const obj = JSON.parse(json);
      minified = JSON.stringify(obj);
    } catch {
      push("err", "payload must be valid JSON");
      return;
    }
    ws.send(minified);
    push("tx", minified);
  }

  function sendJson(obj: any) {
    if (!ws || ws.readyState !== WebSocket.OPEN) {
      push("err", "not connected");
      return;
    }
    const s = JSON.stringify(obj);
    ws.send(s);
    push("tx", s);
  }

  function startSession(cwd = ".", mcpServers: any = []) {
    const id = nextId++;
    const msg = {
      jsonrpc: "2.0",
      id,
      method: "session/new",
      params: { cwd, mcpServers }
    };
    sendJson(msg);
  }

  function sendPrompt(text: string) {
    const sid = store.activeSessionId() || sessionId();
    if (!sid) {
      push("err", "no session; click Start Session first");
      return;
    }
    const id = nextId++;
    const msg = {
      jsonrpc: "2.0",
      id,
      method: "session/prompt",
      params: {
        sessionId: sid,
        prompt: [ { type: "text", text } ]
      }
    };
    sendJson(msg);
    // optimistic user echo in chat for the active session
    if (sid) {
      pushMessageFor(sid, { from: "user", text, ts: new Date().toISOString() });
    }
  }

  function closeSession(id?: string) {
    const sid = id || store.activeSessionId();
    if (!sid) return;
    // Best-effort notify agent if it supports a close or cancel; fallback to client-only removal
    try {
      const msg = { jsonrpc: "2.0", id: nextId++, method: "session/close", params: { sessionId: sid } };
      sendJson(msg);
    } catch {}
    // Remove locally regardless
    import("../state").then(({ removeSession }) => removeSession(sid));
  }

  return { state, log, sessionId, connect, disconnect, sendRaw, sendJson, startSession, sendPrompt, closeSession };
}
