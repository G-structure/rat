import { For, Show, createSignal } from "solid-js";
import { useAcpWs } from "./lib/ws";

export default function App() {
  const { state, log, sessionId, connect, disconnect, startSession, sendPrompt } = useAcpWs();
  const [prompt, setPrompt] = createSignal("");

  return (
    <div>
      <header class="hdr">
        <Dot state={state()} />
        <h1>RAT Web UI — Local</h1>
        <div class="spacer" />
        <button onClick={() => connect()}>Connect</button>
        <button onClick={() => disconnect()}>Disconnect</button>
      </header>
      <main class="main">
        <div style="display:flex;gap:8px;padding:8px 16px;border-bottom:1px solid #111826;align-items:center;">
          <button onClick={() => startSession()} disabled={!!sessionId()}>
            {sessionId() ? `Session: ${sessionId()}` : "Start Session"}
          </button>
          <span style="color:#6b7a90;">cwd “.”, mcpServers []</span>
        </div>
        <div id="log" class="log">
          <For each={log()}>{(line) => <div>{line}</div>}</For>
        </div>
        <Show when={sessionId()}>
          <div class="input">
            <input
              value={prompt()}
              onInput={(e) => setPrompt(e.currentTarget.value)}
              placeholder="Type your prompt"
              style="flex:1;min-height:0;background:#0f141b;color:#d7e1ee;border:1px solid #1a2130;border-radius:6px;padding:8px;"
            />
            <button onClick={() => { if (prompt().trim()) sendPrompt(prompt()); }}>Send Prompt</button>
          </div>
        </Show>
      </main>
    </div>
  );
}

function Dot(props: { state: "idle" | "connecting" | "open" | "closed" }) {
  const color = () =>
    props.state === "open" ? "var(--ok)" : props.state === "connecting" ? "var(--warn)" : "var(--muted)";
  return <span class="dot" style={{ background: color(), "box-shadow": `0 0 6px ${color()}` }} title={props.state} />;
}
