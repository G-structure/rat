# Agent Client Protocol (ACP) — RAT Implementation Guide

This document is a complete, self‑contained reference for implementing the Agent Client Protocol (ACP) in RAT (Rust Agent Terminal). It extracts the normative behavior from the ACP docs and schema and adds pragmatic guidance for a TUI client implementation.

Audience: engineers and coding agents working on RAT’s ACP TUI. Scope: client‑side implementation that talks to ACP agents over JSON‑RPC 2.0 via stdio.

---

## 1) What ACP Is

ACP standardizes communication between code editors/clients and AI coding agents. It decouples agents from client UIs by defining:

- A transport: JSON‑RPC 2.0 over stdin/stdout (agent runs as a subprocess of the client)
- A session model with prompt turns and streaming updates
- A content model compatible with MCP (Model Context Protocol)
- Bidirectional RPC: clients call agent methods; agents call client methods

Default human‑readable text is Markdown. Agents stream progress via notifications so clients can render incremental UI.

---

## 2) Transport & Process Model

- Process: client launches agent as a subprocess; both communicate over stdio using JSON‑RPC 2.0
- Concurrency: a single process can host multiple concurrent sessions (each has its own context/history)
- Message types: methods (request/response) and notifications (one‑way)
- Streaming: agent streams UI updates with `session/update` notifications while a `session/prompt` request is active

---

## 3) Versioning & Capability Negotiation (initialize)

All connections start with `initialize` (client → agent). The client sends the highest major protocol version it supports and its capabilities; the agent replies with the negotiated version, its capabilities, and any auth methods.

Request (client → agent):
```
{"jsonrpc":"2.0","id":0,"method":"initialize","params":{
  "protocolVersion": 1,
  "clientCapabilities": {
    "fs": {"readTextFile": true, "writeTextFile": true}
  }
}}
```

Response (agent → client):
```
{"jsonrpc":"2.0","id":0,"result":{
  "protocolVersion": 1,
  "agentCapabilities": {
    "loadSession": true,
    "promptCapabilities": {"image": false, "audio": false, "embeddedContext": true}
  },
  "authMethods": []
}}
```

Rules:
- `protocolVersion` is a single MAJOR integer. If the agent doesn’t support the requested version, it returns its latest; if the client can’t support it, disconnect.
- Capabilities are optional and additive. Anything omitted MUST be treated as unsupported.

Client capabilities:
- `fs.readTextFile` → client implements `fs/read_text_file`
- `fs.writeTextFile` → client implements `fs/write_text_file`
- Note: a `terminal` capability exists in the schema as UNSTABLE; RAT should not rely on it for core flows.

Agent capabilities:
- `loadSession` → agent implements `session/load`
- `promptCapabilities` → which prompt content the agent accepts in `session/prompt` beyond the baseline:
  - Baseline: text and resource_link are always supported
  - Optional: `image`, `audio`, `embeddedContext` (embedded resource)

---

## 4) Authentication (authenticate)

If the agent requires auth before sessions can be created, its `initialize` response advertises `authMethods` (IDs + names/descriptions). The client then calls:

Method: `authenticate` (client → agent)
```
{"jsonrpc":"2.0","id":1,"method":"authenticate","params":{
  "methodId": "<one of initialize.authMethods[].id>"
}}
```

On success, the agent returns `null` (per schema) and subsequent `session/new` is allowed. If auth is required but missing, agent may reject `session/new` with an error (implementation‑specific); clients should surface a clear auth prompt flow.

---

## 5) Sessions (session/new, session/load)

Sessions isolate conversation state and context. One agent process may host many sessions.

Create a session (client → agent):
```
{"jsonrpc":"2.0","id":2,"method":"session/new","params":{
  "cwd":"/abs/project/root",
  "mcpServers":[{"name":"fs","command":"/path/to/mcp","args":["--stdio"],"env":[]}]
}}
```
→ Response: `{ "sessionId": "sess_abc123" }`

Load an existing session (optional, if `loadSession: true`):
```
{"jsonrpc":"2.0","id":3,"method":"session/load","params":{
  "sessionId":"sess_abc123","cwd":"/abs/project/root","mcpServers":[...]}}
```

When loading, the agent MUST replay the historical conversation to the client via `session/update` notifications (user/agent chunks, tool updates, etc.), then respond to `session/load` with `null` when replay completes.

Definitions:
- `sessionId`: unique string used on all subsequent calls
- `cwd`: absolute path that establishes file‑system scope for the session
- `mcpServers`: optional list of MCP servers (name, command, args, env) that the agent should connect to (agent may also connect through a client‑provided MCP proxy)

---

## 6) Prompt Turn Lifecycle (session/prompt)

A prompt turn begins with a user message and ends when the agent returns a `stopReason`. During the turn, the agent streams UI updates.

1) Client sends the prompt:
```
{"jsonrpc":"2.0","id":4,"method":"session/prompt","params":{
  "sessionId":"sess_abc123",
  "prompt":[
    {"type":"text","text":"Analyze this file."},
    {"type":"resource","resource":{"uri":"file:///.../main.rs","mimeType":"text/x-rust","text":"fn main(){}"}}
  ]
}}
```
Content blocks must conform to negotiated `promptCapabilities` (see §7). All file URIs/paths MUST be absolute.

2) Agent streams progress via `session/update` notifications (see §8):
- Execution plan: `sessionUpdate: "plan"`
- Agent/user/“thought” chunks: `agent_message_chunk`, `user_message_chunk`, `agent_thought_chunk`
- Tool calls: `tool_call` then `tool_call_update`

3) Completion: the agent responds to the original `session/prompt` request with a `stopReason`:
```
{"jsonrpc":"2.0","id":4,"result":{"stopReason":"end_turn"}}
```

Stop reasons:
- `end_turn` | `max_tokens` | `max_turn_requests` | `refusal` | `cancelled`

Cancellation:
- Client MAY send `session/cancel` at any time: `{"method":"session/cancel","params":{"sessionId":"..."}}`
- After cancelling, the agent MUST finish sending any pending updates and then reply to `session/prompt` with `stopReason: "cancelled"` (not an error). Clients MUST reply `cancelled` to any pending `session/request_permission`.

---

## 7) Content Model (MCP‑compatible)

Baseline prompt support (all agents):
- `text`: `{ "type":"text", "text":"..." }`
- `resource_link`: references a resource the agent can access: `{ "type":"resource_link", "uri":"file:///...", "name":"...", ... }`

Optional prompt support (gated by agent `promptCapabilities`):
- `image`: base64 image `{ type:"image", data, mimeType }` → requires `image:true`
- `audio`: base64 audio `{ type:"audio", data, mimeType }` → requires `audio:true`
- `resource` (embedded): include full contents `{ type:"resource", resource:{ text|blob, uri, mimeType? } }` → requires `embeddedContext:true`

Annotations: optional MCP “annotations” may accompany blocks and can hint at display/usage. Agents/clients may ignore them safely.

---

## 8) Streaming Updates (session/update)

Agent → Client notifications during a prompt turn:

- `user_message_chunk` → a piece of the user’s message
- `agent_message_chunk` → a piece of the agent’s user‑visible response (Markdown)
- `agent_thought_chunk` → a piece of the agent’s internal reasoning (if surfaced)
- `plan` → execution plan entries; on update, the agent sends the complete list and the client REPLACES the current plan
- `tool_call` → announces a tool call with id/title/kind/status/content/locations/raw{Input,Output}
- `tool_call_update` → updates an existing tool call (only changed fields required)

Execution plan entry fields:
- `content` (string), `priority` (high|medium|low), `status` (pending|in_progress|completed)

Tool call fields (on create):
- `toolCallId` (string, required), `title` (string, required)
- Optional: `kind` (read|edit|delete|move|search|execute|think|fetch|other), `status` (pending|in_progress|completed|failed; defaults to `pending`), `content` (array of ToolCallContent), `locations` (array of { path, line? }), `rawInput`, `rawOutput`

Tool call updates:
- Use `tool_call_update` with `toolCallId` and any subset of: `status`, `title`, `kind`, `content`, `locations`, `rawInput`, `rawOutput`

Tool call content variants:
- `content` → wraps a regular ContentBlock (text/image/audio/resource/resource_link)
- `diff` → `{ type:"diff", path:"/abs/file", oldText:null|"...", newText:"..." }`
- `terminal` (present in schema for terminal streaming; currently UNSTABLE in caps)

Following the agent:
- Tool calls can include `locations` to hint which files/lines they’re touching so the UI can follow along (preview and highlight).

---

## 9) Permission Requests (session/request_permission)

Agents may request user authorization before executing sensitive tool calls.

Agent → Client method:
```
{"jsonrpc":"2.0","id":5,"method":"session/request_permission","params":{
  "sessionId":"sess_abc123",
  "toolCall": {"toolCallId":"call_001"},
  "options":[
    {"optionId":"allow-once","name":"Allow once","kind":"allow_once"},
    {"optionId":"reject-once","name":"Reject","kind":"reject_once"}
  ]
}}
```

Client → Agent response:
```
{"jsonrpc":"2.0","id":5,"result":{ "outcome": {"outcome":"selected","optionId":"allow-once"} }}
```

Outcomes:
- `selected` with an `optionId` from `options`
- `cancelled` if the prompt was cancelled (client MUST return this after `session/cancel`)

Option kinds (UI hinting): `allow_once` | `allow_always` | `reject_once` | `reject_always`

Clients MAY auto‑decide per user policy.

---

## 10) Client File System Methods (optional)

Only exposed if the client advertised support in `initialize.clientCapabilities.fs`.

Read text files (agent → client):
```
method: fs/read_text_file
params: { sessionId, path:"/abs/file", line?: uint (1‑based), limit?: uint }
→ result: { content: string }
```

Write text files (agent → client):
```
method: fs/write_text_file
params: { sessionId, path:"/abs/file", content:string }
→ result: null (create file if missing)
```

Notes:
- All paths MUST be absolute
- Line numbers are 1‑based
- Reads should reflect unsaved editor buffers if applicable

---

## 11) MCP Servers (optional)

Clients can pass MCP server specs on session creation so agents can directly call MCP tools:
- `name` (string), `command` (abs path), `args` (string[]), `env` (name/value[])

Clients may also expose their own tools by running an MCP server or a proxy that tunnels MCP requests back to the client.

---

## 12) Error Handling

- JSON‑RPC 2.0 error objects for failures (standard codes/messages)
- Notifications never get responses
- Cancellation is not an error: after `session/cancel`, the agent MUST return `stopReason: "cancelled"` on the in‑flight `session/prompt`
- Clients should surface agent errors cleanly; unknown fields in updates should be ignored for forward compatibility

---

## 13) Complete RPC Surface (v1)

Agent methods (client → agent):
- `initialize`
- `authenticate`
- `session/new`
- `session/prompt`
- `session/load` (if `loadSession:true`)

Agent notifications (client → agent):
- `session/cancel`

Client methods (agent → client):
- `session/request_permission`
- `fs/read_text_file` (if `fs.readTextFile:true`)
- `fs/write_text_file` (if `fs.writeTextFile:true`)
- Terminal methods exist in the schema (`terminal/create`, `terminal/output`, `terminal/wait_for_exit`, `terminal/kill`, `terminal/release`) but are UNSTABLE and not part of the core spec; avoid relying on them until stabilized.

Client notifications (agent → client):
- `session/update` (see §8)

Argument requirements:
- All file paths MUST be absolute
- Line numbers are 1‑based

---

## 14) End‑to‑End Flow Example

1) initialize (negotiate version/caps)
2) authenticate (if required)
3) session/new (pass `cwd` and optional `mcpServers`)
4) session/prompt (send text + embedded files)
   - Receive `session/update` plan and message chunks
   - Receive `session/update` tool_call with `toolCallId"
   - Agent calls `session/request_permission` → client allows
   - Receive `tool_call_update` with `in_progress` → then `completed` with `content` (diffs or text)
   - Repeat model/tool loop until done
5) session/prompt result with `stopReason: "end_turn"`

Cancel path: client sends `session/cancel`; agent returns `stopReason: "cancelled"` on the in‑flight prompt, after final updates.

---

## 15) RAT TUI Implementation Guidance

Event loop & transport:
- Spawn agent subprocess; wire stdin/stdout to a JSON‑RPC codec (tokio + framed I/O)
- Maintain a router that correlates request `id` → response; notifications go to a stream handler
- Keep reading stdout continuously in a non‑blocking task; never block the TUI

Session management:
- Support multiple concurrent sessions; represent each as its own state (history, plan, active tool calls)
- Tab UI maps well to sessions; show per‑session status and agent identity

Rendering updates:
- agent_message_chunk/user_message_chunk → append to chat transcript as streaming Markdown
- agent_thought_chunk → render in a collapsible/obfuscated panel if exposed
- plan → replace the entire plan model each time; visualize priority and status
- tool_call → create a card/tile with title/kind/status; show “follow‑along” path/line (soft navigation)
- tool_call_update → update status, append outputs; render diffs with a dedicated diff viewer

Permissions UX:
- On `session/request_permission`, show a modal with the option list and safe defaults; persist “always” decisions per agent/session if desired
- When a turn is cancelled, auto‑respond to pending permission requests with `cancelled`

Filesystem methods:
- Enforce absolute paths
- For reads, prefer integrating the editor’s unsaved buffer model if present; otherwise read from disk
- For writes, create file if missing; ensure non‑blocking I/O and queue changes to avoid jank

Error/cancel semantics:
- Treat cancellation as a first‑class state; never bubble cancel as a generic error in the UI
- Unknown `session/update` variants should be ignored with a log; prefer forward compatibility

Telemetry & logs:
- Structured logs for RPC (method, latency, size) with redaction; toggle with RAT verbosity flags

---

## 16) Implementation Checklist (RAT)

- Transport: JSON‑RPC 2.0 codec over stdio; request/response correlation; notification handler
- Initialize: send latest supported `protocolVersion`, advertise fs capabilities
- Auth: if `authMethods` present, implement selection → `authenticate`
- Sessions: `session/new` and `session/load` (if agent supports)
- Prompting: build content blocks; respect agent `promptCapabilities`
- Streaming: handle all `session/update` variants and update UI state live
- Tools: render tool call cards; execute permission flow; show status/content, diffs, and follow‑along locations
- FS: implement `fs/read_text_file` and `fs/write_text_file` if advertised
- Cancellation: UI action → `session/cancel`; resolve turn with `stopReason: "cancelled"`
- Robustness: handle agent crash/disconnect; clear UI; allow reconnect/restart

---

## 17) Gotchas & Best Practices

- Absolute paths everywhere; no relative file paths
- Line numbers are 1‑based
- Plan updates replace the entire plan; don’t diff them client‑side
- Tool call statuses are a finite set; keep UI in sync: pending → in_progress → completed|failed
- Cancellation must conclude with `stopReason: "cancelled"` on the prompt; do not treat as error
- Prefer embedded `resource` for context you can provide directly; use `resource_link` when the agent can fetch itself
- Keep UI responsive; all long work in background tasks; no blocking reads/writes on the TUI thread

---

## 18) Minimal Reference (method index)

Client → Agent methods:
- `initialize`, `authenticate`, `session/new`, `session/prompt`, `session/load?`

Client → Agent notifications:
- `session/cancel`

Agent → Client methods:
- `session/request_permission`, `fs/read_text_file?`, `fs/write_text_file?`

Agent → Client notifications:
- `session/update` with variants: `user_message_chunk`, `agent_message_chunk`, `agent_thought_chunk`, `plan`, `tool_call`, `tool_call_update`

---

This guide mirrors ACP v1 as documented in the upstream docs and schema, tailored for RAT’s TUI. When in doubt, follow JSON‑RPC 2.0 semantics and the negotiation rules defined in `initialize`.

---

## 19) TUI UI Elements and Source References

This section expands each UI element RAT renders, with pointers to the ACP upstream docs and schema so implementers can trace fields to their definitions. Filepaths are relative to the `agent-client-protocol` repo that is vendored alongside RAT.

- Chat Stream (agent/user/thought chunks)
  - Events: `session/update` with `agent_message_chunk`, `user_message_chunk`, `agent_thought_chunk`
  - Docs: `docs/protocol/prompt-turn.mdx` (Agent Reports Output)
  - Schema: `schema/schema.json` → `$defs.SessionUpdate` variants
  - Rendering: stream markdown; keep chunks ordered within a turn

- Agent Plan Panel
  - Event: `session/update` with `plan`
  - Docs: `docs/protocol/agent-plan.mdx`
  - Schema: `$defs.Plan`, `$defs.PlanEntry`, and SessionUpdate `plan` variant
  - Semantics: replace-on-update; show `priority` and `status`

- Tool Calls Panel
  - Events: `session/update` `tool_call` and `tool_call_update`
  - Docs: `docs/protocol/tool-calls.mdx`
  - Schema: `$defs.ToolCall`, `$defs.ToolCallUpdate`, `$defs.ToolCallStatus`, `$defs.ToolKind`, SessionUpdate variants
  - Fields: `toolCallId` (key), `title`, `kind`, `status`, `content`, `locations`, `rawInput`, `rawOutput`

- Diff Review
  - Content: `ToolCallContent::diff` with `path`, `oldText`, `newText`
  - Docs: `docs/protocol/tool-calls.mdx#diffs`
  - Schema: `$defs.ToolCallContent` → `diff` arm
  - Notes: treat `oldText==null` as create/overwrite preview

- Permission Requests Dialog
  - Method: `session/request_permission` (agent → client)
  - Docs: `docs/protocol/tool-calls.mdx#requesting-permission`
  - Schema: `$defs.RequestPermissionRequest`, `$defs.RequestPermissionOutcome`, `$defs.PermissionOption{,Kind,Id}`
  - Rules: auto‑respond `cancelled` if turn cancelled; option kinds hint UI

- Locations & Follow‑Along
  - Field: `ToolCall.locations[]` with `{ path, line? }`
  - Docs: `docs/protocol/tool-calls.mdx#following-the-agent`
  - Schema: `$defs.ToolCallLocation`

- Commands Palette (available commands)
  - Event: `session/update` with `available_commands_update` (used by some agents)
  - Schema: `$defs.SessionUpdate` → `available_commands_update` (x-docs-ignore but defined)
  - Model: `$defs.AvailableCommand{,Input}`

- Authentication Flow
  - Methods: `initialize` (advertises `authMethods`), `authenticate`
  - Docs: `docs/protocol/initialization.mdx`, `docs/protocol/schema.mdx#authenticate`
  - Schema: `$defs.InitializeResponse.authMethods`, `$defs.AuthMethod{,Id}`, `$defs.AuthenticateRequest`

- Session Lifecycle UI
  - Methods: `session/new`, `session/load`
  - Docs: `docs/protocol/session-setup.mdx`
  - Schema: `$defs.NewSessionRequest/Response`, `$defs.LoadSessionRequest`
  - Replay: load requires streaming full history via `session/update` then responding `null`

- Stream & Cancellation State
  - Notification: `session/cancel` (client → agent)
  - Result: the in‑flight `session/prompt` must respond with `stopReason:"cancelled"`
  - Docs: `docs/protocol/prompt-turn.mdx#cancellation`
  - Schema: `$defs.CancelNotification`, `$defs.StopReason`

- Terminal (UNSTABLE)
  - Methods/Content: `terminal/*` methods and ToolCallContent `terminal`
  - Schema: `$defs.CreateTerminal*`, `$defs.TerminalOutput*`, `$defs.WaitForTerminalExit*`, ToolCallContent `terminal`
  - Note: marked unstable in caps; gate behind a feature flag in RAT

- Client FS Integration
  - Methods: `fs/read_text_file`, `fs/write_text_file`
  - Docs: `docs/protocol/file-system.mdx`
  - Schema: `$defs.ReadTextFile*`, `$defs.WriteTextFile*`, `$defs.FileSystemCapability`
  - Constraint: absolute paths; 1‑based lines

- Status Bar & Notifications
  - Surfaces: connection state from `initialize` and session methods, stop reasons from `session/prompt` result, permission queue length
  - Docs: `docs/protocol/overview.mdx` and `prompt-turn.mdx` (stop reasons)
  - Schema: `$defs.StopReason`

- Audio Content (prompt and outputs)
  - Content: `ContentBlock::audio` gated by `promptCapabilities.audio`
  - Docs: `docs/protocol/content.mdx#audio-content`
  - Schema: `$defs.ContentBlock` → `audio`, `$defs.AudioContent`

- Image Content (prompt and outputs)
  - Content: `ContentBlock::image` gated by `promptCapabilities.image`
  - Docs: `docs/protocol/content.mdx#image-content`
  - Schema: `$defs.ContentBlock` → `image`, `$defs.ImageContent`

- Prompt Composer Attachments (capability‑aware)
  - Content: `resource` (embedded) vs `resource_link` (reference), plus `image`/`audio` blocks
  - Docs: `docs/protocol/content.mdx`
  - Schema: `$defs.ContentBlock` → `resource`, `resource_link`, `image`, `audio`; `$defs.EmbeddedResource{,Resource}`
  - Constraints: absolute URIs/paths; capability checks via `promptCapabilities`

- Resource/Resource Link Rendering
  - Same sources as above; add actions for preview/open

- Refusal Stop Reason UX
  - Result: `stopReason:"refusal"`
  - Docs: `docs/protocol/prompt-turn.mdx#stop-reasons`
  - Schema: `$defs.StopReason` → `refusal`

- Permission Policy Memory
  - Local client behavior; not standardized. Keep responses within `RequestPermissionOutcome` but allow “remember” UX to preselect options.

- Initialization & Version/Capability UI
  - Method: `initialize`
  - Docs: `docs/protocol/initialization.mdx`
  - Schema: `$defs.InitializeRequest/Response`, `$defs.AgentCapabilities`, `$defs.ClientCapabilities`, `$defs.ProtocolVersion`

- Large Content Truncation
  - Client policy; respect content shapes. When truncating, ensure schema validity and clearly indicate truncation in UI.

- Accessibility & Keybindings
  - RAT‑specific; map keys to the above events and flows. Keep non‑blocking.

- MCP Servers Summary & Configuration
  - Parameters: `session/new` and `session/load` `mcpServers[]` entries (`name`, `command`, `args`, `env`)
  - Docs: `docs/protocol/session-setup.mdx#mcp-servers`
  - Schema: `$defs.McpServer`, `$defs.EnvVariable`
  - UI: show a summary on connect and offer configuration in settings.

- Content Annotations (MCP-compatible)
  - Appear on content blocks and embedded resources; optional hints like `audience`, `priority`, `lastModified`
  - Docs: `docs/protocol/content.mdx` (links to MCP annotations)
  - Schema: `$defs.Annotations` and presence on `$defs.ContentBlock`, `$defs.EmbeddedResource`, `$defs.ResourceLink`
  - UI: render as subtle badges/tooltips when present.
