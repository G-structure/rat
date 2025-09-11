# Refactor Plan for sim_agent.rs

## Overview
The current `examples/sim_agent.rs` manually implements a JSON-RPC 2.0 loop over stdio, parsing incoming requests and crafting JSON responses/notifications. This is error-prone, non-idiomatic, and doesn't leverage the `agent-client-protocol` crate's abstractions. The refactor will replace this with an implementation of the `acp::Agent` trait, using `AgentSideConnection` to handle protocol details, ensuring compliance with ACP schema and simplifying maintenance.

**Goals:**
- Eliminate manual JSON serialization/deserialization.
- Use `acp::Agent` methods: `initialize`, `authenticate`, `new_session`, `load_session`, `prompt`, `cancel`.
- In `prompt`, asynchronously send `SessionNotification`s (e.g., plan, tool_call, agent_message_chunk) via a channel to simulate scenarios.
- Handle `session/request_permission` by sending it as a client request through the connection (not a notification).
- Preserve CLI flags: `--scenario`, `--speed`, `--seed`, `--loop-run`.
- Support all existing scenarios: HappyPathEdit, FailurePath, ImagesAndThoughts, CommandsUpdate (add more as per PLAN.md Simulator Support Checklist).
- Ensure deterministic, seedable behavior for testing.

**Non-Goals:**
- Add new scenarios in this refactor (do incrementally post-MVP).
- Implement full FS methods (e.g., `fs/read_text_file`); simulate via mocks if needed for scenarios.
- Real randomness; use seed for reproducible delays/content.

## Current Implementation Analysis
- **Main Loop (lines 32-138):** Tokio stdin reader parses JSON-RPC, dispatches to handlers for methods like `initialize`, `session/new`, `session/prompt`.
- **Responses:** Manual `json!` and `write_json` for results/errors.
- **Notifications:** `send_update` crafts `session/update` notifications.
- **Scenarios in `prompt`:** Switch on `cli.scenario`, call async fns like `run_happy_path` that send updates and handle permission request/response.
- **Issues:**
  - No type safety; manual error-prone JSON.
  - Permission handling hacks into stdin reader.
  - No integration with `acp` types (e.g., `ToolCall`, `PlanEntry`).
  - Sleeps are ad-hoc; no jitter/seed.
  - Missing full ACP compliance (e.g., no proper `StopReason` in responses).

From `rat/PLAN.md` and Simulator Support Checklist, we need to cover: initialize capabilities, session lifecycle, prompt streaming (plan/tool_call/message), cancellation, permissions, commands_update, etc.

## Dependencies & Setup
- Ensure `Cargo.toml` includes `agent-client-protocol = "0.2.0-alpha.6"` (already present per PLAN.md).
- Add `anyhow`, `clap`, `env_logger`, `serde_json`, `tokio` (with full features) if not present.
- For scenarios, embed fixture data (e.g., diff texts, image base64) as constants.

## Refactor Steps

### Step 1: Study ACP Crate Interfaces (Prep Work)
- Review `agent-client-protocol/rust/{acp.rs, agent.rs, tool_call.rs, plan.rs, content.rs}` for types: `Agent`, `SessionNotification`, `ToolCall`, `Plan`, `ContentBlock`, `StopReason`.
- From `example_agent.rs`:
  - Implement `Agent` trait.
  - Use `mpsc::UnboundedSender<(SessionNotification, oneshot::Sender<()>)>` for sending updates.
  - `AgentSideConnection::new(agent, outgoing, incoming, spawn_local)` handles I/O.
  - Background task receives from channel, calls `conn.session_notification(notification).await`.
- For requests to client (e.g., `session/request_permission`): Use `conn.request_to_client` or similar (inspect `AgentSideConnection` API; if not exposed, extend or use raw RPC).
- Note: Permissions are agent-initiated requests to client, so need to send via connection's RPC sender.

### Step 2: Restructure sim_agent.rs
- **CLI Parsing:** Keep `Cli` struct with `clap`; parse early in `main()`.
- **Agent Struct:** 
  ```rust
  struct SimAgent {
      cli: Cli,
      session_update_tx: mpsc::UnboundedSender<(SessionNotification, oneshot::Sender<()>)>,
      next_session_id: Cell<u64>,
      // Add: scenario runners, speed_mult, seed RNG
  }
  ```
  - Init: Create channel `(tx, rx)`; spawn background notifier task.
- **main():**
  - Parse CLI, compute `speed_mult = parse_speed(&cli.speed)`.
  - Create `SimAgent::new(cli, tx)`.
  - Use `LocalSet` for non-Send futures.
  - `AgentSideConnection::new(sim_agent, stdout.compat_write(), stdin.compat(), spawn_local)`.
  - Spawn notifier: loop on `rx`, `conn.session_notification(notif).await; tx.send(())`.
  - `handle_io.await`.

### Step 3: Implement Agent Trait Methods
- **initialize(&self, args: InitializeRequest) -> InitializeResponse:**
  - Return `protocol_version: V1`, `agent_capabilities: default()` (enable `load_session: true`, `prompt_capabilities: {image: true, embedded_context: true}`), `auth_methods: vec![]`.
- **authenticate(&self, _): Result<(), Error>:**
  - `Ok(())` (no auth for sim).
- **new_session(&self, args: NewSessionRequest) -> NewSessionResponse:**
  - Generate `session_id: format!("sim-{}", self.next_session_id.get())`; increment.
- **load_session(&self, _): Result<(), Error>:**
  - Sleep scaled 50ms, return `Ok(())` or mock history if needed.
- **cancel(&self, args: CancelNotification): Result<(), Error>:**
  - Set internal `cancelling = true`; return `Ok(())`.
- **prompt(&self, args: PromptRequest) -> PromptResponse:**
  - Reset `cancelling = false`.
  - Match `self.cli.scenario` to run async scenario simulation:
    - Spawn task(s) to send notifications via `self.session_update_tx` (e.g., plan, tool_call, chunks).
    - For permissions: Send `session/request_permission` request via connection (need API access; if not, use raw JSON-RPC send).
    - Wait for scenario completion (use barriers or joins).
  - Return `PromptResponse { stop_reason: if self.cancelling { StopReason::Cancelled } else { EndTurn } }`.

### Step 4: Implement Scenario Runners
- Move `run_happy_path`, etc., to methods or async fns on `SimAgent`.
- Use `acp` types:
  - **Plan:** `SessionUpdate::Plan(Plan { entries: vec![PlanEntry { content: "Open file...".to_string(), priority: Priority::Medium, status: Status::InProgress }] })`.
  - **Tool Call:** `SessionUpdate::ToolCall(ToolCall { tool_call_id: "call_edit_1".into(), title: "Edit src/lib.rs".into(), kind: ToolCallKind::Edit, status: Status::InProgress, content: vec![ContentBlock::Diff { path: "/workspace/src/lib.rs".into(), old_text: None, new_text: "pub fn greet()...".into() }], locations: vec![Location { path: "/workspace/src/lib.rs".into(), line: Some(1) }] })`.
  - **Chunks:** `SessionUpdate::AgentMessageChunk { content: ContentBlock::Text("Applied the change...".into()) }`.
  - **Update:** `SessionUpdate::ToolCallUpdate(ToolCallUpdate { tool_call_id: "...".into(), status: Status::Completed })`.
  - **Commands:** `SessionUpdate::AvailableCommandsUpdate(AvailableCommandsUpdate { commands: vec![Command { id: "new-session".into(), name: "New Session".into(), description: "Create...".into() }] })`.
- Timing: Use `tokio::time::sleep(Duration::from_millis((ms as f32 / speed) as u64))`; add seed-based jitter if `--jitter-ms`.
- Permissions in HappyPathEdit:
  - Construct `RequestToClient::SessionRequestPermission { session_id, tool_call: ToolCall { id: "...", ... }, options: vec![Option { option_id: "allow-once".into(), name: "Allow once".into(), kind: OptionKind::AllowOnce }] }`.
  - Send via `conn.request_to_client(req).await?;` (assume API; if not, fallback to manual JSON).
  - Wait for response by polling or channel (but since it's stdio loop, connection handles it).

### Step 5: Handle Permissions Properly
- Current hack: Manually write JSON request, read response from stdin.
- Refactored: Use connection's RPC to send request, await response in scenario runner.
- If `AgentSideConnection` doesn't expose `send_request`, inspect/extend it or use internal RPC sender.
- For sim, assume client responds; in runner, sleep/wait for simulated response handling.
- Test: Ensure round-trip works without blocking main prompt.

### Step 6: Add Seeding & Looping
- Use `rand::SeedableRng` with `--seed` for reproducible delays/content variations.
- `--loop-run`: If true, restart prompt loop indefinitely.
- Add `--pause` or signal handling for step/debug (future).

### Step 7: Testing & Verification
- **Unit Tests:** Add `#[cfg(test)] mod tests { ... }` with mock connection; test each Agent method returns expected types.
- **Integration:** `cargo test --test sim_agent_acp` – spawn sim_agent as subprocess, send JSON-RPC via stdin, assert stdout.
- **Manual:** Run `cargo run --example sim_agent -- --scenario happy-path-edit --speed fast`; pipe to `cargo run --example basic_client` if available, or use RAT with `--agent-cmd`.
- **With RAT:** Run user command; check `rat/logs/rat.log` for no errors, proper streaming (plan/tool/permission/message/stop). No manual 'q' needed; auto-end_turn.
- **Edge Cases:** Cancellation mid-stream, invalid JSON (graceful ignore), oversized content (truncate), malformed updates (log/warn).
- Commands: `cargo build --example sim_agent`, `cargo test`, `RUST_LOG=trace cargo run -p rat -- ...` (verify log shows ACP compliance, no JSON parse errors).

## Risks & Mitigations
- **API Gaps:** If `AgentSideConnection` lacks request sending, add thin wrapper or use raw `serde_json` for requests only (keep notifications typed).
- **Async Complexity:** Use `tokio::spawn` for scenario tasks; ensure all await properly to avoid races.
- **Performance:** Scaled sleeps ensure fast tests; max speed=100x for instant.
- **Scenario Fidelity:** Map current JSON exactly to `acp` structs; diff before/after.
- **Versioning:** Pin to "0.2.0-alpha.6"; test against schema.json if changes.

## Timeline
- Day 1: Implement Agent trait skeleton + initialize/new_session/prompt stub.
- Day 2: Refactor one scenario (HappyPathEdit) with typed notifications + permission.
- Day 3: Add remaining scenarios, seeding, tests.
- Day 4: Integrate with RAT, manual verification, iterate on logs.

## Next Steps Post-Refactor
- Add missing scenarios (e.g., multi_tools, large_diff).
- Implement FS mocks for `fs/*` methods in Agent trait.
- Expose sim_agent as RAT default for UI dev.
- Update PLAN.md with completed Simulator Support Checklist items.
- Snapshot TUI frames via insta for each scenario.
