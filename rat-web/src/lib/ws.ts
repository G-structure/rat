import { createSignal } from "solid-js";

export type WsState = "idle" | "connecting" | "open" | "closed";

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
    };
    ws.onmessage = (ev) => {
      const text = typeof ev.data === "string" ? ev.data : "[binary]";
      push("rx", text);
      try {
        const v = JSON.parse(text);
        const sid = v?.result?.sessionId;
        if (typeof sid === "string" && sid.length > 0) {
          setSessionId(sid);
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
    const sid = sessionId();
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
  }

  return { state, log, sessionId, connect, disconnect, sendRaw, sendJson, startSession, sendPrompt };
}
