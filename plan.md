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
- [x] **Basic ACP client implementation using `agent-client-protocol`** - ✅ COMPLETED (claude-code-acp working)
- [x] **Message serialization/deserialization** - ✅ COMPLETED
- [x] **Session management and connection handling** - ✅ COMPLETED (ACP integration working)
- [x] **Basic TUI shell with ratatui** - ✅ COMPLETED (tabbed interface, keybindings, welcome screen)
- [x] **Configuration system with TOML support** - ✅ COMPLETED

#### Phase 2: Claude Code Integration (Weeks 3-4) - ✅ COMPLETED
- [x] **Claude Code subprocess adapter** - ✅ COMPLETED (full ACP integration with proper API usage)
- [x] **Permission system for file operations** - ✅ COMPLETED (integrated with ACP tool calls and permission management)
- [x] **Basic edit review interface** - ✅ COMPLETED (enhanced diff algorithm with proper UI rendering)
- [x] **Terminal session embedding** - ✅ COMPLETED (permission-aware command execution with ACP integration)
- [x] **Error handling and recovery** - ✅ COMPLETED (comprehensive error handling throughout)

#### Phase 3: Gemini Integration (Weeks 5-6) - ⚠️ IN PROGRESS  
- [ ] **Gemini CLI integration as ACP agent** - ⚠️ PARTIAL (structure exists, needs implementation)
- [x] **Unified agent interface abstraction** - ✅ COMPLETED (AgentAdapter trait implemented)
- [x] **Agent switching and session management** - ✅ MOSTLY COMPLETE (AgentManager handles multiple agents)
- [ ] **MCP server pass-through support** - ❌ NOT STARTED
- [ ] **Model selection and configuration** - ⚠️ PARTIAL (config structure exists)

#### Phase 4: Visual Enhancement (Weeks 7-8) - 🚧 IN PROGRESS
- [x] **Tachyonfx integration for UI animations** - ✅ INITIAL INTEGRATION
  - Added global EffectManager, ambient neon border pulse, subtle HSL drift
  - Post-processing pipeline runs each frame on terminal buffer
- [ ] **Code diff visualization with effects** - ❌ NOT STARTED
- [ ] **Syntax highlighting with color transitions** - ⚠️ PARTIAL (basic structure exists)
- [x] **Theme system implementation** - ✅ FOUNDATION ADDED
  - Cyberpunk palette + surface/background styles
  - Applied background fill, tab highlight, chat/input borders
- [x] **Status indicators and progress bars** - ✅ BASIC IMPLEMENTATION (status bar exists; restyled)

#### Phase 5: Advanced Features (Weeks 9-10) - ❌ EARLY STAGE
- [x] **Multi-session management** - ✅ COMPLETED (tabbed sessions, session switching implemented)
- [x] **Project-specific configurations** - ✅ COMPLETED (config system supports per-project settings)
- [x] **Keybinding customization** - ✅ COMPLETED (config system with keybinding support)
- [ ] **Plugin system for custom effects** - ❌ NOT STARTED
- [ ] **Performance profiling and optimization** - ❌ NOT STARTED

#### Phase 6: Polish & Documentation (Weeks 11-12) - ❌ NOT STARTED
- [ ] **Comprehensive testing suite** - ⚠️ PARTIAL (basic test structure exists)
- [ ] **User documentation and tutorials** - ❌ NOT STARTED  
- [ ] **Installation and packaging** - ❌ NOT STARTED
- [ ] **Performance benchmarks** - ❌ NOT STARTED
- [ ] **Release preparation** - ❌ NOT STARTED

---

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

The project has excellent architectural foundations but needs focused work on the core ACP functionality to become functional.

### File Structure
```
rat/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── app.rs                    # Main application state
│   ├── config/
│   │   ├── mod.rs
│   │   ├── agent.rs             # Agent configurations
│   │   ├── ui.rs                # UI preferences
│   │   └── project.rs           # Project-specific settings
│   ├── acp/
│   │   ├── mod.rs
│   │   ├── client.rs            # ACP client implementation
│   │   ├── session.rs           # Session management
│   │   ├── message.rs           # Message handling
│   │   └── permissions.rs       # Permission system
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── claude_code.rs       # Claude Code adapter
│   │   ├── gemini.rs            # Gemini CLI adapter
│   │   ├── manager.rs           # Multi-agent instance manager
│   │   └── traits.rs            # Common adapter interfaces
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.rs               # Main UI coordinator
│   │   ├── chat.rs              # Chat interface
│   │   ├── diff.rs              # Edit review interface
│   │   ├── terminal.rs          # Terminal embedding
│   │   ├── statusbar.rs         # Status bar
│   │   └── components/          # Reusable UI components
│   ├── effects/
│   │   ├── mod.rs
│   │   ├── text.rs              # Text animation effects
│   │   ├── code.rs              # Code-specific effects
│   │   ├── transitions.rs       # UI transition effects
│   │   └── themes.rs            # Theme and color effects
│   └── utils/
│       ├── mod.rs
│       ├── diff.rs              # Diff utilities
│       ├── syntax.rs            # Syntax highlighting
│       └── terminal.rs          # Terminal utilities
├── examples/
│   ├── basic_client.rs          # Simple ACP client example
│   └── effects_demo.rs          # Tachyonfx effects showcase
├── tests/
│   ├── integration/
│   └── unit/
└── docs/
    ├── user_guide.md
    ├── configuration.md
    └── development.md
```

### Technical Challenges & Solutions

1. **ACP Protocol Complexity**
   - **Challenge**: Managing bidirectional JSON-RPC with multiple concurrent sessions
   - **Solution**: Use `agent-client-protocol` crate with careful async state management

2. **Cross-Platform Agent Integration**
   - **Challenge**: Different agents have varying installation and execution patterns
   - **Solution**: Abstract agent management with capability detection and auto-installation

3. **Performance with Visual Effects**
   - **Challenge**: Maintaining 60fps with complex tachyonfx animations
   - **Solution**: Selective effect application, frame rate limiting, and effect LOD system

4. **Terminal Integration**
   - **Challenge**: Embedding interactive terminals within TUI
   - **Solution**: Use `portable-pty` with custom rendering and input routing

5. **Edit Review UX**
   - **Challenge**: Making diff review intuitive and efficient
   - **Solution**: Hunk-level navigation with clear visual feedback and batch operations

### Success Metrics

1. **Performance**: Sub-100ms response times for UI interactions
2. **Reliability**: 99%+ uptime for agent connections
3. **Usability**: Intuitive interface requiring minimal learning curve
4. **Extensibility**: Plugin system supporting custom agents and effects
5. **Adoption**: Positive community feedback and contribution activity

### Risk Mitigation

1. **Agent API Changes**: Version pinning with update notifications
2. **Platform Compatibility**: Extensive testing on major platforms
3. **Performance Issues**: Profiling throughout development with optimization sprints
4. **User Experience**: Regular user testing and feedback incorporation

### Future Enhancements

- **Multi-Language Support**: Internationalization for global usage
- **Cloud Sync**: Configuration and session synchronization
- **Collaborative Features**: Shared sessions and pair programming
- **AI Training Integration**: Custom model fine-tuning support
- **Extension Marketplace**: Community-driven plugins and themes

---

This project positions RAT as the premier terminal-based interface for AI coding agents, combining the performance of Rust with the visual appeal of modern UIs, setting a new standard for developer-AI interaction paradigms.

---

## ACP‑Aligned TUI UI Plan (Deep Dive)

Task: Define and scope all UI elements in RAT’s TUI that are directly supported by the Agent Client Protocol (ACP), including agent plans, tool calls, permission prompts, diffs, and related flows. This plan is derived from ACP’s schema and examples in `agent-client-protocol`, the Claude Code ACP adapter, and our local ACP guide.

Context:
- RAT is an ACP client and must render the full set of ACP streaming updates and agent→client requests.
- References reviewed: `rat/ACP.md` (local spec), `agent-client-protocol` (schema + Rust client), `claude-code-acp` (real agent emitting plan/tool/diff/availableCommands), and existing RAT TUI scaffolding.
- Goal: add UI affordances that map 1:1 to ACP features with minimal client‑side invention.

Approach:
- MVP first: prioritize read‑only rendering for all ACP updates, plus interactive permission selection. Add navigation + streaming polish next. Defer UNSTABLE terminal features behind a feature flag.
- Keep RAT non‑blocking: stream updates into state; the draw loop renders snapshots.
- Tests-first for each element as we implement (snapshot frames via `insta`).

Scope of UI Elements (ACP‑backed):

1) Chat Stream (session/update: agent_message_chunk, user_message_chunk, agent_thought_chunk)
- Render Markdown text chunks with syntax highlighting for code fences.
- Show images where present (inline thumbnail with open‑full action if supported; fallback: placeholder + metadata).
- “Thoughts” collapsed by default with a toggle to expand; visually distinct from user‑visible content.
- Stream-safe: accumulate chunks per turn; show typing indicator while receiving.

2) Agent Plan Panel (session/update: plan)
- Read‑only task list showing entry content, priority, and status (pending/in_progress/completed).
- Replace‑on‑update semantics: each incoming plan replaces the entire list (per spec).
- Visual cues: status icons, progress bar (% completed), priority color.
- Navigation: jump between plan and related tool calls/messages in the same turn.

3) Tool Calls Panel (session/update: tool_call, tool_call_update)
- Card per tool call with: title, kind (read/edit/delete/move/search/execute/think/fetch/other), status (pending/in_progress/completed/failed).
- Stream content items: text/resource/resource_link/diff/(terminal if enabled) with incremental updates.
- Locations list: file paths and optional line numbers; actions to preview, open, or follow.
- Collapsible details with compact timeline view; show rawInput/rawOutput in an “advanced” foldout.

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
- “Follow along” toggle: auto‑scroll tool/diff panels to the most recent location when enabled.

7) Available Commands & Slash UX (session/update: available_commands_update)
- Command palette with name/description and argument hint (from Claude Code adapter).
- Type `/` in chat input to filter + insert commands with argument placeholders; display MCP‑backed commands when advertised.

8) Authentication Flow (initialize.authMethods, auth_required errors)
- If `initialize` advertises `authMethods`, show a setup banner with selectable auth method(s) and guidance.
- When the agent raises auth required (e.g., Claude prompts to run `/login`), surface a prominent call‑to‑action.

9) Session Lifecycle UI (session/new, session/load replay)
- New: show connected banner with capabilities summary (promptCapabilities, loadSession).
- Load: show replay progress while the agent replays history via `session/update`; then mark “ready”.

10) Stream & Cancellation State (session/cancel and stopReason)
- “Cancel Turn” action; after sending cancel, mark the turn as cancelling and continue displaying late updates until the agent responds with `stopReason: cancelled`.
- Stop reason toast on completion: end_turn, max_tokens, max_turn_requests, refusal, cancelled.

11) Terminal (UNSTABLE, feature‑flagged)
- Optional panel for “terminal” ToolCallContent if emitted; background terminal progress + last output.
- Controls gated by ACP unstable client methods (create/release/kill/wait_for_exit/terminal_output).

12) Client FS Integration (fs/read_text_file, fs/write_text_file)
- No direct UI action beyond previews and write confirmations.
- Ensure all file paths are absolute; show a small “edited” badge in status bar when writes occur.

13) Status Bar & Notifications
- Connection state, active agent/session, streaming indicator, plan progress, pending permission count.
- Non‑blocking toasts for errors and important state changes.

14) Audio Content (ContentBlock::audio)
- Gated by `promptCapabilities.audio`. In chat and tool content streams, show an audio attachment chip with mime/duration when available.
- If playback support is implemented, add play/pause/mute controls; otherwise provide a “save/open externally” action and clearly indicate no inline playback.

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
- Offer a clear CTA to “Start new turn” and optionally adjust composer hinting.

18) Permission Policy Memory
- When user selects `allow_always`/`reject_always`, remember a client‑side policy scoped by agent and optionally session.
- Provide a UI to view and clear remembered policies (settings panel or command palette action).

19) Initialization & Version/Capability UI
- On successful `initialize`, show capability summary (promptCapabilities, loadSession). On version mismatch (client can’t support agent’s returned MAJOR), display a graceful error view with retry/help.
- Provide quick toggles to advertise FS capabilities and an indicator when FS is disabled.

20) Large Content Truncation & Performance Policy
- Define truncation for long text blocks, large diffs, images, and audio metadata (e.g., preview first N KB, with “Open full”/“Save” actions).
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

---

## UI Elements TODO Checklist (ACP‑backed)

- [ ] Chat stream rendering
  - [ ] agent_message_chunk (Markdown; code fences highlighted)
  - [ ] user_message_chunk
  - [ ] agent_thought_chunk (collapsed by default; toggle)
  - [ ] Image blocks (thumbnail + open‑full; fallback placeholder)
  - [ ] Typing/streaming indicator
- [ ] Agent Plan panel
  - [ ] Replace‑on‑update semantics
  - [ ] Status icons (pending/in_progress/completed) and percent progress
  - [ ] Priority colors (high/medium/low)
  - [ ] Navigation link to related tool/messages
- [ ] Tool Calls panel
  - [ ] Create on `tool_call` with title/kind/status
  - [ ] Update on `tool_call_update` (status/content/locations/raw IO)
  - [ ] Collapsible details; compact timeline view
  - [ ] Locations list with preview + jump
  - [ ] RawInput/RawOutput foldouts
  - [ ] Filtering (by status/kind/text)
- [ ] Diff viewer
  - [ ] Unified view with syntax highlight
  - [ ] Hunk navigation (next/prev)
  - [ ] Large diff performance handling (pagination/chunking)
  - [ ] Side‑by‑side view (later)
- [ ] Permission requests modal
  - [ ] Render `options[]` with kind labels (allow_once/always, reject_once/always)
  - [ ] Context preview (diff/locations/summary)
  - [ ] Queueing and navigation between multiple prompts
  - [ ] Auto‑reply `cancelled` after turn cancel
- [ ] Available commands (slash palette)
  - [ ] Palette with name/description and argument hint
  - [ ] Insert command into input with placeholder
- [ ] Authentication UX
  - [ ] Initialize banner for `authMethods`
  - [ ] Auth‑required error surfacing with actionable CTA
- [ ] Session lifecycle
  - [ ] New session banner with capabilities summary
  - [ ] Load session replay progress indicator
- [ ] Cancellation + stop reason
  - [ ] Cancel action; pending state; continue accepting updates
  - [ ] Stop reason toast (end_turn|max_tokens|max_turn_requests|refusal|cancelled)
- [ ] Status bar
  - [ ] Connection/agent/session
  - [ ] Streaming indicator
  - [ ] Plan progress
  - [ ] Pending permission count
- [ ] Keybinding help overlay (Cheatsheet)
- [ ] Errors & diagnostics
  - [ ] Error toasts/banners (agent errors, protocol errors)
  - [ ] Lightweight logs/diagnostics viewer
- [ ] Themes & accessibility
  - [ ] Colorblind‑safe palette
  - [ ] Theme switch (light/dark/high‑contrast)
- [ ] Layout controls
  - [ ] Toggle panels (plan/tools/diff)
  - [ ] Focus mode; resize panes; follow mode toggle
- [ ] Images previewer (with metadata)
- [ ] MCP servers awareness (basic indicator if provided)
- [ ] Terminal (UNSTABLE, feature‑flagged)
  - [ ] Background terminal output pane (read‑only)
  - [ ] Terminal lifecycle badges on tool cards

---

## Simulator Support Checklist

- [ ] initialize
  - [ ] protocol v1; promptCapabilities: image + embeddedContext; loadSession toggle
  - [ ] authMethods advertised (for auth scenario)
- [ ] new_session (+ optional load_session replay scenario)
- [ ] prompt lifecycle
  - [ ] agent_message_chunk streaming
  - [ ] agent_thought_chunk streaming
  - [ ] image chunks (base64 or URL)
  - [ ] plan updates (replace list)
  - [ ] tool_call create with:
    - [ ] title/kind/status
    - [ ] locations (path + optional line)
    - [ ] content: text/resource/resource_link/diff
    - [ ] rawInput
  - [ ] tool_call_update with:
    - [ ] status transitions (pending→in_progress→completed/failed)
    - [ ] content/locations/rawOutput updates
  - [ ] request_permission with options (allow_once/always, reject_once/always)
  - [ ] available_commands_update
  - [ ] stopReason variants (end_turn, max_tokens, max_turn_requests, refusal, cancelled)
- [ ] cancel handling
  - [ ] Accept `session/cancel` during streaming
  - [ ] Continue sending final updates, then return `cancelled`
- [ ] fs methods
  - [ ] Agent calls `fs/read_text_file` for a preview location
  - [ ] Agent calls `fs/write_text_file` to finalize an edit (optional)
- [ ] Error injection
  - [ ] Unknown tool_call_update id (client should not crash)
  - [ ] Oversized diff (client truncation policy)
  - [ ] Malformed content ignored (forward‑compat safe)
- [ ] Controls
  - [ ] --scenario (happy_path_edit|multi_tools|failure|cancellation|large_diff|images_and_thoughts|auth_required|commands_update)
  - [ ] --speed, --jitter-ms, --seed, --loop, pause/step
---

## ACP Simulator for UI Development

Task: Provide a no‑credits, deterministic ACP agent simulator to design, human‑test, and iterate RAT’s UI without relying on Claude Code or network access.

Why
- Enables rapid UI prototyping of all ACP features (plans, tool calls, diffs, permissions, cancellation, stop reasons, command palette, auth) with fully controlled streams.
- Deterministic, seedable runs for reproducible snapshots and mutation tests.
- Human‑testing friendly: speed controls, pause/step, scenario switching.

Approach
- Add an example binary `examples/sim_agent.rs` implementing `agent_client_protocol::Agent`, communicating over stdio.
- Drive scripted `session/update` sequences and `session/request_permission` calls regardless of prompt contents.
- Provide multiple scenarios via CLI flags: `--scenario`, `--speed`, `--seed`, `--loop`.
- Use the crate’s `example_agent.rs` pattern (LocalSet + mpsc) to stream notifications.

Scenarios (initial set)
- happy_path_edit: plan → tool_call(edit+diff) → permission → accepted → tool_call_update(completed) → message → end_turn.
- multi_tools: two concurrent tool calls (search then edit), interleaved updates, locations following.
- failure_path: tool_call fails → update(status=failed) → agent_msg explaining; stopReason=end_turn.
- cancellation: stream for a while; after client sends `session/cancel`, finish pending updates then respond with `stopReason=cancelled`.
- large_diff: big file edit with multiple hunks; test navigation and performance.
- images_and_thoughts: emit image chunks and thought chunks; verify collapsible rendering.
- auth_required: on first prompt, error path that implies auth required; include `initialize.authMethods`.
- commands_update: send `available_commands_update` with a sample list.

Emitted Content (per ACP)
- agent_message_chunk, user_message_chunk, agent_thought_chunk (Markdown/text and images)
- plan (replace‑on‑update semantics)
- tool_call (with kind/status/title/content/locations/rawInput)
- tool_call_update (status changes + content/location updates)
- request_permission (options: allow_once/always, reject_once/always)
- available_commands_update (for slash UX)

Controls
- Speed: `--speed=slomo|normal|fast|max` or numeric multiplier.
- Pause/step: simulator responds to SIGUSR1 or stdin keycodes to pause/resume/step.
- Jitter: `--jitter-ms` adds random delay noise around scripted timings for realism.
- Seed: `--seed` to reproduce interleavings and generated content.

Integration
- Add a RAT flag `--agent-cmd <path>` (or config override) to spawn a custom agent; use it to launch the simulator. If not present, add as a small CLI addition in a follow‑up.
- Default simulator command: `cargo run -q --example sim_agent -- --scenario happy_path_edit --speed fast`.

Testing
- Use `insta` snapshots of TUI frames for each scenario (freeze speed, seed=0, deterministic timestamps). Validate: plan rendering, tool card lifecycle, diff view, permission modal flows, stop reasons, command palette list.
- Add property tests for diff view invariants (e.g., toggling hunks never loses lines; accept/reject selection state independent of scroll).
- Mutation tests: parsing and state reducers for session updates.

Minimal Implementation Sketch (sim_agent.rs)
- implement Agent::initialize returning protocol v1 + promptCapabilities { embeddedContext: true, image: true }.
- Agent::new_session: incrementing session ids.
- Agent::prompt: spawn a task that emits the chosen scenario’s scripted notifications to `session_notification()`, then return `stop_reason` at the end (unless cancelled).
- Agent::cancel: set cancelled flag; scenario runner drains pending updates then sets final stop reason.
- ToolCallContent::diff: construct from small fixture strings embedded in the example.

Verification
- Manually: run RAT against the simulator to perform human UI testing; adjust speed and step to inspect states.
- Automated: scenario smoke tests that run RAT headless and snapshot text frames.

Next
- Add `examples/sim_agent.rs` and wire a `--agent-cmd` override in RAT.
- Land one scenario (happy_path_edit) with plan/tool/diff/permission; then iterate others.
