import { For, Show, createSignal } from "solid-js";
import { useAcpWs } from "./lib/ws";
import { store, selectActiveSession } from "./state";
import { ChatView } from "./components/ChatView";
import { PlanPanel } from "./components/PlanPanel";
import { TerminalView } from "./components/TerminalView";
import { CommandsPanel } from "./components/CommandsPanel";
import { ModeSelector } from "./components/ModeSelector";
import { PermissionDialog } from "./components/PermissionDialog";
import { DiffView } from "./components/DiffView";

export default function App() {
  const { state, log, sessionId, connect, disconnect, startSession, sendPrompt, closeSession } = useAcpWs();
  const [prompt, setPrompt] = createSignal("");
  const sessions = () => Object.keys(store.sessions());
  const active = () => store.activeSessionId();
  const [showRaw, setShowRaw] = createSignal(false);
  const [view, setView] = createSignal<"chat" | "plan" | "diffs" | "terminal" | "commands" | "raw">("chat");

  return (
    <div class="shell">
      <header class="hdr">
        <Dot state={state()} />
        <h1>RAT Web UI — Local</h1>
        <div class="spacer" />
        <button onClick={() => connect()}>Connect</button>
        <button onClick={() => disconnect()}>Disconnect</button>
      </header>
      <div class="tabsbar">
        <button onClick={() => startSession()} style="background:#173021; border-color:#1f3a2b;">New Session</button>
        <For each={sessions()}>
          {(sid) => (
            <span style="display:inline-flex; align-items:center; gap:4px;">
              <button
                onClick={() => selectActiveSession(sid)}
                style={`padding:6px 10px;border-radius:6px;border:1px solid #1a2130; ${active()===sid?"background:#192847;":"background:#0f141b;"}`}
                title={`Select session ${sid}`}
              >{sid.slice(0,8)}…</button>
              <button
                onClick={() => closeSession(sid)}
                title="Remove session"
                style="padding:4px 6px;border-radius:6px;border:1px solid #1a2130;background:#1d0f10;color:#f3b0b0;"
              >×</button>
            </span>
          )}
        </For>
      </div>
      <div class="topnav">
        <button class={view()==='chat'? 'on' : ''} onClick={() => setView('chat')}>Chat</button>
        <button class={view()==='plan'? 'on' : ''} onClick={() => setView('plan')}>Plan</button>
        <button class={view()==='diffs'? 'on' : ''} onClick={() => setView('diffs')}>Diffs</button>
        <button class={view()==='terminal'? 'on' : ''} onClick={() => setView('terminal')}>Terminal</button>
        <button class={view()==='commands'? 'on' : ''} onClick={() => setView('commands')}>Commands</button>
        <button class={view()==='raw'? 'on' : ''} onClick={() => setView('raw')}>Raw</button>
      </div>
      <div class="threecol">
        <aside class="sidebar-left">
          <CommandsPanel />
          <ModeSelector />
        </aside>
        <main class="main">
          <div class="contentArea">
            <Show when={view()==='chat'}>
              <ChatView />
            </Show>
            <Show when={view()==='plan'}>
              <PlanPanel />
            </Show>
            <Show when={view()==='diffs'}>
              <DiffView />
            </Show>
            <Show when={view()==='terminal'}>
              <TerminalView />
            </Show>
            <Show when={view()==='commands'}>
              <CommandsPanel />
            </Show>
            <Show when={view()==='raw'}>
              <div class="log" style="border-top:1px solid #111826; max-height:30vh; overflow:auto;">
                <For each={log()}>{(line) => <div>{line}</div>}</For>
              </div>
            </Show>
            <Show when={showRaw()}>
              <div class="log" style="border-top:1px solid #111826; max-height:30vh; overflow:auto;">
                <For each={log()}>{(line) => <div>{line}</div>}</For>
              </div>
            </Show>
          </div>
          <Show when={active()}>
            <div class="input">
              <input
                value={prompt()}
                onInput={(e) => setPrompt(e.currentTarget.value)}
                placeholder="Type your prompt"
                style="flex:1;min-height:0;background:#0f141b;color:#d7e1ee;border:1px solid #1a2130;border-radius:6px;padding:8px;"
              />
              <button onClick={() => { if (prompt().trim()) sendPrompt(prompt()); }}>Send Prompt</button>
              <button onClick={() => setShowRaw(!showRaw())} title="Toggle raw ACP log">{showRaw() ? "Hide Raw" : "Show Raw"}</button>
              <button onClick={() => closeSession()} title="Close active session" style="background:#311515; border-color:#472222;">Close Session</button>
            </div>
          </Show>
        </main>
        <aside class="sidebar-right">
          <PlanPanel />
          <div style="border-top:1px solid #111826;">
            <TerminalView />
          </div>
        </aside>
      </div>
      <PermissionDialog />
    </div>
  );
}

function Dot(props: { state: "idle" | "connecting" | "open" | "closed" }) {
  const color = () =>
    props.state === "open" ? "var(--ok)" : props.state === "connecting" ? "var(--warn)" : "var(--muted)";
  return <span class="dot" style={{ background: color(), "box-shadow": `0 0 6px ${color()}` }} title={props.state} />;
}
