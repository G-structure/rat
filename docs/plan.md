# RAT (Rust Agent Terminal)
## High-Performance ACP Client for Claude Code & Gemini

### Executive Summary

RAT is a high-performance terminal-based ACP (Agent Client Protocol) client written in Rust, leveraging tachyonfx for stunning visual effects. The project creates a unified interface for interacting with multiple AI coding agents (Claude Code and Gemini CLI) through a standardized protocol, providing a superior alternative to traditional terminal interactions with rich visual feedback, structured edit reviews, and **multi-agent control capabilities**. 

RAT enables developers to manage and control multiple agents simultaneously - switching between different agents for different tasks, running parallel sessions, and maintaining multiple concurrent conversations with different AI models.

### Project Architecture

#### Core Components

1. **ACP Client Core** (`src/acp/`)
   - **Client Implementation**: Rust-based ACP client using `agent-client-protocol` crate
   - **Session Management**: Multi-session support with concurrent agent connections
   - **Message Routing**: JSON-RPC 2.0 bidirectional communication handling
   - **Permission System**: Interactive permission prompts for file operations and tool calls

2. **Agent Adapters** (`src/adapters/`)
   - **Claude Code Adapter**: Integration with `@anthropic-ai/claude-code` SDK via subprocess
   - **Gemini Adapter**: Direct integration with Gemini CLI as ACP agent
   - **Unified Interface**: Common adapter trait for seamless agent switching
   - **Multi-Agent Manager**: Control multiple agent instances simultaneously
   - **Health Monitoring**: Agent availability and capability detection

3. **TUI Framework** (`src/ui/`)
   - **Main Interface**: Tabbed layout supporting multiple concurrent agent sessions
   - **Agent Selector**: Quick switching between active agents with visual indicators
   - **Chat View**: Message threading with agent identification and syntax highlighting
   - **Edit Review**: Diff viewer with hunk-level accept/reject using tachyonfx transitions
   - **Terminal Integration**: Embedded terminal sessions with streaming output
   - **Multi-Agent Dashboard**: Overview of all active agents and their current tasks
   - **Status Bar**: Real-time multi-agent status, session info, and progress indicators

4. **Effects System** (`src/effects/`)
   - **Message Animations**: Typewriter effects for AI responses
   - **Code Highlighting**: Syntax-aware color transitions for code blocks
   - **Edit Transitions**: Smooth diff animations with fade/slide effects
   - **Status Indicators**: Pulsing connection status and activity indicators
   - **Theme System**: Dynamic color schemes with smooth transitions

5. **Configuration** (`src/config/`)
   - **Agent Settings**: API keys, model preferences, timeout configurations
   - **UI Preferences**: Themes, keybindings, layout preferences
   - **Project Context**: Per-project agent preferences and custom instructions

### Technical Specifications

#### Dependencies
```toml
[dependencies]
agent-client-protocol = "0.2.0-alpha.6"
tachyonfx = "0.18.0"
ratatui = "0.29.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
uuid = "1.11"
crossterm = "0.29"
```

#### Key Features

1. **Multi-Agent Control**
   - Simultaneous control of multiple Claude Code and Gemini agent instances
   - Tabbed interface for managing concurrent agent sessions
   - Quick agent switching with session preservation
   - Per-agent configuration and state management
   - Parallel task execution across different agents

2. **Rich Visual Experience**
   - Animated code diffs with tachyonfx effects
   - Smooth transitions between UI states
   - Syntax-highlighted code blocks with color animations
   - Real-time typing indicators and status updates

3. **Advanced Edit Management**
   - Structured edit review with diff visualization
   - Hunk-level accept/reject with animated feedback
   - Undo/redo support for edit operations
   - Batch edit operations with progress visualization

4. **Terminal Integration**
   - Embedded terminal sessions for agent tool execution
   - Streaming output with syntax highlighting
   - Interactive shell access with permission gating
   - Background process management

5. **Performance Optimizations**
   - Lazy loading of UI components
   - Efficient buffer management for large files
   - Streaming message processing
   - Minimal memory footprint

### Implementation Phases

#### Phase 1: Core Infrastructure (Weeks 1-2) - ✅ COMPLETED
- [x] **Basic ACP client implementation using `agent-client-protocol`**
- [x] **Message serialization/deserialization**
- [x] **Session management and connection handling**
- [x] **Basic TUI shell with ratatui**
- [x] **Configuration system with TOML support**
- [x] **Agent manager with adapter pattern**
- [x] **Basic event loop and keybindings**

#### Phase 2: Claude Code Integration (Weeks 3-4) - ✅ COMPLETED
- [x] **Claude Code subprocess adapter**
- [x] **Permission system for file operations**
- [x] **Basic edit review interface**
- [x] **Terminal session embedding**
- [x] **Error handling and recovery**
- [x] **ACP initialization and capability negotiation**

#### Phase 3: Gemini Integration (Weeks 5-6) - ⚠️ IN PROGRESS  
- [ ] **Gemini CLI integration as ACP agent** - ⚠️ PARTIAL (structure exists, needs implementation)
- [x] **Unified agent interface abstraction** - ✅ COMPLETED (AgentAdapter trait implemented)
- [x] **Agent switching and session management** - ✅ MOSTLY COMPLETE (AgentManager handles multiple agents)
- [ ] **MCP server pass-through support** - ❌ NOT STARTED
- [ ] **Model selection and configuration** - ⚠️ PARTIAL (config structure exists)
- [ ] **Simulator Support Checklist**
  - [ ] Add real `request_permission` round-trip using the connection API. (blocks full tool call flow in simulator)
  - [ ] Gate and implement `AvailableCommandsUpdate` by enabling the crate's unstable feature for RAT. (blocks commands scenario in simulator)
  - [ ] Add extra scenarios per your Simulator Support Checklist, or tweak timings/jitter/seed. (enhances testing robustness)

#### Phase 4: Visual Enhancement (Weeks 7-8) - 🚧 IN PROGRESS
- [x] **Tachyonfx integration for UI animations** - ✅ INITIAL INTEGRATION
  - Added global EffectManager, ambient neon border pulse, subtle HSL drift
  - Post-processing pipeline runs each frame on terminal buffer
- [x] **Agent Selector** - ✅ BASIC IMPLEMENTATION (status bar exists; restyled)
- [ ] **Chat View** - ❌ NOT STARTED
- [ ] **Edit Review** - ❌ NOT STARTED
- [ ] **Agent Plan panel**
  - [ ] Replace‑on‑update semantics
  - [ ] Status icons (pending/in_progress/completed) and percent progress
  - [ ] Priority colors (high/medium/low)
  - [ ] Navigation link to related tool/messages
- [ ] **Syntax highlighting with color transitions** - ⚠️ PARTIAL (basic structure exists)
- [x] **Theme system implementation** - ✅ FOUNDATION ADDED
  - Cyberpunk palette + surface/background styles
  - Applied background fill, tab highlight, chat/input borders
- [x] **Status indicators and progress bars** - ✅ BASIC IMPLEMENTATION (status bar exists; restyled)

#### Phase 5: Advanced Features (Weeks 9-10) - ❌ EARLY STAGE
- [ ] **Multi-session management** - ✅ COMPLETED (tabbed sessions, session switching implemented)
- [ ] **Project-specific configurations** - ✅ COMPLETED (config system supports per-project settings)
- [ ] **Keybinding customization** - ✅ COMPLETED (config system with keybinding support)
- [ ] **Plugin system for custom effects** - ❌ NOT STARTED
- [ ] **Performance profiling and optimization** - ❌ NOT STARTED

#### Phase 6: Polish & Documentation (Weeks 11-12) - ❌ NOT STARTED
- [ ] **Comprehensive testing suite** - ⚠️ PARTIAL (basic test structure exists)
- [ ] **User documentation and tutorials** - ❌ NOT STARTED  
- [ ] **Installation and packaging** - ❌ NOT STARTED
- [ ] **Performance benchmarks** - ❌ NOT STARTED
- [ ] **Release preparation** - ❌ NOT STARTED

---

## 2025‑09‑17 — Local WS ACP Testing Support

Task: Enable ACP testing over local WebSocket without wscat and document usage for websocat, Node, and browser clients.

Context:
- RAT exposes a dev WS bridge (`--local-ws`) for direct ACP JSON‑RPC testing. Browsers and some clients require subprotocol negotiation (Sec‑WebSocket‑Protocol) for correctness.

Approach:
- Minimal code change to echo `acp.jsonrpc.v1` during WS handshake via `accept_hdr_async`.
- Add README instructions covering websocat and a one‑liner Node `ws` client, plus browser flow notes.

Changes:
- src/local_ws.rs: switch to `accept_hdr_async` and echo `Sec-WebSocket-Protocol: acp.jsonrpc.v1` when requested.
- README.md: new section “ACP over Local WebSocket (Dev Testing)” with usage for websocat and Node, pitfalls, and browser flow guidance.
- Tests: added `#[tokio::test]`s in `src/local_ws.rs` to validate
  - WS handshake echoes `acp.jsonrpc.v1`
  - Echo mode accepts ACP-shaped Text frames and returns echo wrapper containing the original JSON

Verification:
- Manual: Launch `RUST_LOG=trace cargo run -p rat -- --local-ws --local-port 8889`.
- Connect with Node `ws` using subprotocol list `["acp.jsonrpc.v1"]`; observe successful handshake and ACP round‑trip (initialize→newSession→prompt).
- Connect with websocat (`websocat -t ws://localhost:8889`) and paste JSON‑RPC lines; observe agent responses.
- Automated: `cargo test -q` or `cargo nextest run` locally. Note: CI sandbox here cannot link on macOS due to `cc` segfault, but tests compile in a normal toolchain.

Remaining:
- Optional: Add an integration test that exercises WS handshake with subprotocol assertion (requires a client harness).
- Optional: Ship a small web demo page in a separate repo (keeps this repo Rust‑only).

Next:
- If desired, add a minimal external web demo showing streaming `session/update` handling against the local bridge.
 
## 2025‑09‑17 — Add Local Solid (Vite) Web UI

Task: Provide a minimal browser UI (SolidJS via Vite) that connects to RAT's local WebSocket (`--local-ws`) using `Sec-WebSocket-Protocol: acp.jsonrpc.v1`, with basic send/receive JSON‑RPC.

Context:
- Developer requested a SolidJS/Vite app in-repo (folder `rat/rat-web`), overriding CLAUDE.md Rust‑only constraint for this subfolder.
- Only the local websockets mode is required; hosted relay + Noise is out of scope for this task.

Approach:
- Small, isolated scaffold under `rat-web/` with no coupling to Rust code.
- Simple log console, Connect/Disconnect, auto `initialize` on connect, text area to send arbitrary JSON‑RPC.

Changes:
- Added `rat-web/` with Vite+Solid scaffold:
  - `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
  - `src/main.tsx`, `src/App.tsx`, `src/lib/ws.ts`, `src/styles.css`
  - `README.md` with usage instructions

Verification:
- Manual: `cargo run -p rat -- --local-ws --local-port 8081`, then `cd rat-web && pnpm i && pnpm dev`, open http://localhost:5173 and verify:
  - WS opens with subprotocol `acp.jsonrpc.v1` (Chrome DevTools → Network → WS)
  - `initialize` is sent and responses are logged
  - Arbitrary JSON‑RPC payloads echo/bridge as expected

Remaining:
- Optional: add typed helpers for common ACP methods (session/new, prompt, session/update streaming render).
- Optional: reconnection with exponential backoff.
- Optional: environment/config for ws URL/port.

Next:
- If desired, style and evolve into a fuller UI (chat view, editor, permissions) per `spec_done.md` once hosted relay path is implemented.


## CURRENT STATUS SUMMARY (Updated: December 2024)

### ✅ **COMPLETED AREAS (~40% of project)**
- **Project Structure & Build System**: Full Rust project with proper dependencies
- **Configuration System**: TOML-based config with agent, UI, and project settings
- **Core Application Framework**: Event loop, async architecture, message passing
- **Basic TUI**: Tabbed interface, keybindings, status bar, welcome screen
- **Multi-Agent Architecture**: AgentManager, adapter pattern, session management
- **Multi-Session Support**: Concurrent sessions with tab switching

### ⚠️ **PARTIALLY COMPLETE AREAS (~30% of project)**
- **ACP Client Core**: Structure exists but needs real protocol implementation
- **Agent Adapters**: Framework in place, needs actual subprocess management
- **Permission System**: Basic structure, needs ACP integration
- **Effects System**: Dependencies added, modules exist but mostly empty
- **Message Routing**: Basic async messaging, needs ACP protocol integration

### ❌ **MISSING CRITICAL COMPONENTS (~30% of project)**
- **Real ACP Protocol Communication**: Currently using dummy implementations
- **Subprocess Management**: Agent processes not actually started/managed
- **Edit Review & Diff Visualization**: Core feature missing
- **Terminal Embedding**: No embedded terminal functionality
- **TachyonFX Visual Effects**: Minimal implementation
- **Testing & Documentation**: Comprehensive coverage missing

### 🚨 **IMMEDIATE PRIORITIES**
1. **Fix ACP Client Implementation** - Replace dummy implementations with real ACP protocol
2. **Implement Agent Subprocess Management** - Actually start and communicate with agents  
3. **Add Basic Edit Review** - Core diff viewing and approval workflow
4. **Test End-to-End Functionality** - Ensure agent communication works

---

## ACP‑Aligned TUI UI Plan (Deep Dive)

Task: Define and scope all UI elements in RAT's TUI that are directly supported by the Agent Client Protocol (ACP), including agent plans, tool calls, permission prompts, diffs, and related flows. This plan is derived from ACP's schema and examples in `agent-client-protocol`, the Claude Code ACP adapter, and our local ACP guide.

Context:
- RAT is an ACP client and must render the full set of ACP streaming updates and agent→client requests.
- References reviewed: `../docs/ACP.md` (local spec), `agent-client-protocol` (schema + Rust client), `claude-code-acp` (real agent emitting plan/tool/diff/availableCommands), and existing RAT TUI scaffolding.

---

## 2025‑09‑16 — RAT2E/Relay Integration Kickoff

Task: Start implementing RAT2E spec (rat/spec_done.md) across relay and clients.

Context:
- Align relay with WS upgrade gates: Origin allow‑list, single subprotocol echo, attach token parsing.
- Establish minimal RAM‑only pairing path: RAT connects with device_code; browser joins via session_id.
- Provide basic service probes: /health and /version.

Approach:
- Small, compile‑first diff in `relay` to fix mismatched types and implement spec‑aligned subprotocol parsing and pairing waits.
- Add unit tests for subprotocol parsing.
- Defer full Noise/ACP transport and presence metrics; stub hooks only.

Changes:
- relay/src/websocket.rs: replace placeholder SessionSockets, wire to PairingState::SessionEntry; add strict single‑token subprotocol parser; pairing wait; bidirectional blind relay.
- relay/src/main.rs: echo single subprotocol pre‑upgrade; add /health and /version endpoints.

Verification:
- Build relay; unit tests validate parser edge cases; manual WS connect with malformed subprotocol should close 1008 post‑upgrade.

Remaining:
- Configurable Origin allow‑list (ALLOWED_ORIGINS) and enforcement.
- Attach‑token generation/validation and TTL/jti cache.
- Presence snapshot (/v1/presence) with TTL sweeper and tenant scoping.
- Noise XX handshake and ciphertext‑only relay.
- Browser UI (claude‑code‑ui) pairing page and WSS connector.

Next:
- Implement ALLOWED_ORIGINS and close 1008 on mismatch.
- Add presence store and sweeper.
- Add Browser UI stub to complete pairing and open WSS with `acp.jsonrpc.v1.stksha256.<b64u>`.
- Goal: add UI affordances that map 1:1 to ACP features with minimal client‑side invention.

Approach:
- MVP first: prioritize read‑only rendering for all ACP updates, plus interactive permission selection. Add navigation + streaming polish next. Defer UNSTABLE terminal features behind a feature flag.
- Keep RAT non‑blocking: stream updates into state; the draw loop renders snapshots.
- Tests-first for each element as we implement (snapshot frames via `insta`).

Scope of UI Elements (ACP‑backed):

1) Chat Stream (session/update: agent_message_chunk, user_message_chunk, agent_thought_chunk)
- Render Markdown text chunks with syntax highlighting for code fences.
- Show images where present (inline thumbnail with open‑full action if supported; fallback: placeholder + metadata).
- "Thoughts" collapsed by default with a toggle to expand; visually distinct from user‑visible content.
- Stream-safe: accumulate chunks per turn; show typing indicator while receiving.

2) Agent Plan Messages (session/update: plan)
- Plans are now displayed as fancy-looking messages within the chat stream instead of a separate panel.
- Each plan update appears as a multi-line boxed message with ASCII borders, showing:
  - Header: "┌─ Agent Plan ──────────────────────────┐"
  - Each task: "│ ⏳ 🔴 High: Task description │" (with status icons and priority indicators)
  - Footer: "└───────────────────────────────────────┘"
- Status icons: ⏳ (pending), ⚡ (in_progress), ✅ (completed)
- Priority indicators: 🔴 (high), 🟡 (medium), 🟢 (low)
- Content truncation for long task descriptions
- Replace‑on‑update semantics: each incoming plan replaces the entire list (per spec).
- Navigation: plans appear inline with other messages in the conversation flow.

3) Tool Call Messages (session/update: tool_call)
- Tool calls now display as structured, multi-line boxed messages in the chat stream.
- Format includes:
  - Header: "┌─ Tool Call ──────────────────────────────┐"
  - Tool name: "│ 🔧 {tool_name} │"
  - Parameters preview: "│ 📋 {params} │" (truncated JSON)
  - Permission status: "│ 🔒 Requires permission │" or "│ ✅ Auto-approved │"
  - Footer: "└─────────────────────────────────────────┘"
- Shows tool execution context and permission requirements clearly.

4) Tool Result Messages (session/update: tool_call_update with result)
- Tool results display as structured boxes showing:
  - Header: "┌─ Tool Result ────────────────────────────┐"
  - Result preview: "│ 📄 {preview} │" (truncated output)
  - Statistics: "│ 📊 {lines} lines, {chars} chars │"
  - Footer: "└─────────────────────────────────────────┘"
- Provides quick overview of tool execution outcomes.

5) Code Edit Messages (EditProposed)
- Code edits appear as formatted diff previews:
  - Header: "┌─ Code Edit ──────────────────────────────┐"
  - File path: "│ 📁 {path} │" (truncated for long paths)
  - Description: "│ 💬 {description} │" (if available)
  - Diff preview: "│ 🔄 {diff_lines} │" (first few lines)
  - Statistics: "│ 📊 +{additions} -{deletions} │"
  - Footer: "└─────────────────────────────────────────┘"
- Shows file changes with visual diff indicators and change counts.

3) Tool Calls Panel (session/update: tool_call, tool_call_update)
- Card per tool call with: title, kind (read/edit/delete/move/search/execute/think/fetch/other), status (pending/in_progress/completed/failed).
- Stream content items: text/resource/resource_link/diff/(terminal if enabled) with incremental updates.
- Locations list: file paths and optional line numbers; actions to preview, open, or follow.
- Collapsible details with compact timeline view; show rawInput/rawOutput in an "advanced" foldout.

4) Diff Review (ToolCallContent: { type:"diff", path, oldText|null, newText })
- Unified view (MVP) with optional side‑by‑side; syntax highlighting; hunk navigation.
- If oldText is null, treat as create/overwrite preview; otherwise show additions/deletions.
- Accept/Reject affordances are contextual to permission requests (see #5). Outside a permission prompt, diffs are preview‑only.

5) Permission Requests Dialog (session/request_permission)
- Modal dialog with tool summary (title/kind/locations), a focused diff/file preview when available, and options from `options[]`.
- Option kinds: allow_once, allow_always, reject_once, reject_always (used for labels/shortcuts only; the agent defines policy).
- Required flows:
  - Submit selected option to agent, or
  - If turn was cancelled (`session/cancel`), auto‑respond with `cancelled`.
- Queue multiple concurrent permission prompts; show clear context for which tool call each corresponds to.

6) Locations & Following (ToolCall.locations)
- When locations contain paths/lines, show a contextual preview and allow jump‑to/peek.
- "Follow along" toggle: auto‑scroll tool/diff panels to the most recent location when enabled.

7) Available Commands & Slash UX (session/update: available_commands_update)
- Command palette with name/description and argument hint (from Claude Code adapter).
- Type `/` in chat input to filter + insert commands with argument placeholders; display MCP‑backed commands when advertised.

8) Authentication Flow (initialize.authMethods, auth_required errors)
- If `initialize` advertises `authMethods`, show a setup banner with selectable auth method(s) and guidance.
- When the agent raises auth required (e.g., Claude prompts to run `/login`), surface a prominent call‑to‑action.

9) Session Lifecycle UI (session/new, session/load replay)
- New: show connected banner with capabilities summary (promptCapabilities, loadSession).
- Load: show replay progress while the agent replays history via `session/update`; then mark "ready".

10) Stream & Cancellation State (session/cancel and stopReason)
- "Cancel Turn" action; after sending cancel, mark the turn as cancelling and continue displaying late updates until the agent responds with `stopReason: cancelled`.
- Stop reason toast on completion: end_turn, max_tokens, max_turn_requests, refusal, cancelled.

11) Terminal (UNSTABLE, feature‑flagged)
- Optional panel for "terminal" ToolCallContent if emitted; background terminal progress + last output.
- Controls gated by ACP unstable client methods (create/release/kill/wait_for_exit/terminal_output).

12) Client FS Integration (fs/read_text_file, fs/write_text_file)
- No direct UI action beyond previews and write confirmations.
- Ensure all file paths are absolute; show a small "edited" badge in status bar when writes occur.

13) Status Bar & Notifications
- Connection state, active agent/session, streaming indicator, plan progress, pending permission count.
- Non‑blocking toasts for errors and important state changes.

14) Audio Content (ContentBlock::audio)
- Gated by `promptCapabilities.audio`. In chat and tool content streams, show an audio attachment chip with mime/duration when available.
- If playback support is implemented, add play/pause/mute controls; otherwise provide a "save/open externally" action and clearly indicate no inline playback.

15) Prompt Composer Attachments (capability‑aware)
- Allow attaching files/resources when composing prompts:
  - Use embedded `resource` when `embeddedContext:true` and the file is readable; otherwise fall back to `resource_link`.
  - Support image attachments only if `image:true`; audio only if `audio:true`.
- Validate absolute paths before send; provide helper to convert relative paths to absolute based on the session `cwd`.

16) Resource and Resource Link Rendering in Chat/Tools
- For `resource_link`, render a compact chip with `name`, optional `title/size/mimeType`, and actions: preview (if text), open externally, copy URI.
- For embedded `resource` (text/blob), render a short preview with expand action; for binary show metadata + save option.

17) Refusal Stop Reason UX
- When `stopReason: refusal`, show a banner explaining the agent refused to continue and that the next turn should not auto‑append the prior user message.
- Offer a clear CTA to "Start new turn" and optionally adjust composer hinting.

18) Permission Policy Memory
- When user selects `allow_always`/`reject_always`, remember a client‑side policy scoped by agent and optionally session.
- Provide a UI to view and clear remembered policies (settings panel or command palette action).

19) Initialization & Version/Capability UI
- On successful `initialize`, show capability summary (promptCapabilities, loadSession). On version mismatch (client can't support agent's returned MAJOR), display a graceful error view with retry/help.
- Provide quick toggles to advertise FS capabilities and an indicator when FS is disabled.

20) Large Content Truncation & Performance Policy
- Define truncation for long text blocks, large diffs, images, and audio metadata (e.g., preview first N KB, with "Open full"/"Save" actions).
- Ensure streaming remains responsive; chunk rendering and backpressure in the UI loop.

21) Accessibility & Keybindings for Media/Attachments
- Add keys for attach file/image/audio in composer, and for media playback or opening attachments.
- Document keys alongside existing interaction model.

22) MCP Servers Summary & Config
- During session creation, show which MCP servers will be connected (name, command, args). Provide a settings view to edit per‑project MCP servers and environment variables.
- Display a compact summary in the connected banner for quick visibility.

23) Content Annotations (MCP‑compatible)
- Where present on content blocks, surface lightweight metadata: `lastModified`, `priority`, and `audience` hints (e.g., subtle badges or tooltips). Safe to omit when absent.

Interaction Model & Keybindings (additions)
- Space/Enter: expand/collapse focused tool card or plan item.
- p: toggle plan panel; t: toggle tool calls; d: focus diff; /: open commands palette; c: cancel turn.
- y / n: select allow/reject in permission modal; A: allow always; R: reject always.
- j/k or arrows: navigate lists; g/G: first/last; f: follow locations toggle.
  - a: attach file in composer; i: attach image; u: attach audio (if capability supported).
  - Media: space to play/pause (when focused), m to mute.

Data Model Hooks (state additions)
- plan: Vec<PlanEntry>
- tool_calls: ordered map keyed by toolCallId with status, kind, content[], locations[]
- permission_queue: FIFO of pending prompts with toolCallId + options[]
- stream_state: { receiving: bool, stop_reason: Option<StopReason> }
- commands: Vec<AvailableCommand>

Phased Delivery
- MVP (Phase 2/3):
  - Render chat chunks, plan panel (read‑only), tool calls list with diff preview, permission dialog with selection, cancellation flow, stop reason toasts, locations preview, commands palette (read‑only insert).
- Enhanced (Phase 4+):
  - Side‑by‑side diffs, hunk‑level navigation, follow mode polish, raw IO foldouts, thought‑chunk toggle animations, images previewer, terminal (unstable) under feature flag, tachyonfx transitions.

Verification (planned as we implement)
- Snapshot tests (`insta`) for: plan rendering state transitions; tool call lifecycle (create→update→complete/failed); diff preview; permission modal with options; cancellation state and stop reason banners; commands palette entries.
- Property tests for diff hunk navigation invariants.
- Mutation tests on parsing/merge of streaming updates.

Remaining / Risks
- Terminal is unstable; isolate behind a feature flag to avoid churn.
- Some agents omit oldText in diffs; ensure previews remain useful.
- Images and large content blocks require careful buffer management and truncation policies.
- Multiple concurrent permission requests must be queued and clearly disambiguated.

Next
- Wire ACP update handlers to TUI state for: plan, tool_call, tool_call_update, available_commands_update.
- Implement permission modal and response plumbing for `session/request_permission`.
- Add diff preview component reading ToolCallContent:diff.
- Add minimal commands palette using `availableCommands`.
- Snapshot tests for the above; land incrementally.

## 2025‑09‑17 — RAT TUI Implementation Spec

Task: Author a comprehensive `spec_rat.md` that describes the current state of the RAT TUI as implemented.

Context:
- The TUI comprises `TuiManager`, `ChatView`, `StatusBar`, and popups (Welcome/Help/Error). Additional components (AgentSelector, DiffView, TerminalView, PermissionPrompt) exist and are partially implemented but not fully wired into the main loop.
- Visual effects are provided by tachyonfx (startup rain morph, ambient neon border and hue drift), guarded for small terminal sizes.

Approach:
- Read and align with code in `src/ui/**`, `src/app.rs`, `src/main.rs`, `src/config/ui.rs`, `src/effects/**`, and `src/adapters/manager.rs`.
- Document state machines, rendering layout, keybindings, effects, configuration mapping, CLI flags that impact UI, message flow, and known limitations.

Changes:
- Added `spec_rat.md` at repo root with detailed sections: Architecture, State Model, Rendering, Input/Keys, Components, Effects, Message Flow, Pairing/TUI suspension, Config mapping, CLI, Logging, Safety, Limitations, Future hooks, and testing notes.

Verification:
- Manual verification against code strings and behavior (help/welcome copy, tab naming, effect names, min-size guards). No code behavior changed.

Remaining:
- Wire Agent Selector selection to `UiToApp::ConnectAgent`.
- Integrate DiffView and PermissionPrompt flows; expose TerminalView via a keybinding.
- Add snapshot/property tests for ChatView wrapping/scroll invariants and popups.

Next:
- Implement Agent Selector confirm + status updates.
- Add minimal DiffView integration for `EditProposed` messages.

## Progress Update (2025-01-XX)
- **Plan UI Integration**: Moved agent plan display from separate panel to inline messages within chat stream
- **Changes Made**:
  - Removed `PlanView` struct and separate plan rendering logic from `ChatView`
  - Modified `add_message` to treat plan messages as regular messages
  - Added `format_plan_content` method to render plans with status icons and priority colors
  - Updated message formatting to display plans as "Agent Plan:" messages with cyan styling
- **Verification**: Built successfully, all tests pass, plan messages now appear inline in conversation
- **Next**: Continue with other UI elements (tool calls, permission dialogs, diff review)

---
