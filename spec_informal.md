# RAT2E Remote Control via Hosted Relay — Software Design Specification (v1)

This document defines the what, why, and how to ship RAT2E remote control via a tiny, hosted relay. It is implementation‑oriented: each component lists concrete deliverables, interfaces, constraints, and acceptance criteria.

## 1. Purpose & Goals (Why)
- Purpose: Allow anyone running RAT2E locally to control their agent from a hosted web UI over the public internet without exposing local ports.
- Goals:
  - One lightweight relay on 443/TLS; browser‑correct WebSockets. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
  - End‑to‑end privacy with Noise XX; relay cannot read application bytes. [Noise Protocol](https://noiseprotocol.org/noise.html)
  - ACP JSON‑RPC streamed over WS for realtime UX; deterministic reconnect via `session/load`. [ACP Overview](https://agentclientprotocol.com/protocol/overview)
  - Simple presence heartbeat and snapshot for a dashboard; RAM‑only state with TTL; fast, safe, NAT/mobile friendly.
- Non‑Goals (v1): Proxied file sync beyond thin UX endpoints; in‑relay application multiplexing; P2P ICE/QUIC fast‑path (future); persistent chat history. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 2. User Story (Context)
1) Setup: User installs and runs RAT2E locally. They open the hosted web UI.
2) Pair: In terminal, user runs `rat2e --pair`. RAT2E calls `POST /v1/pair/start`, receives `{user_code, device_code, relay_ws_url}`, and shows the `user_code`.
3) Confirm: In the browser, user enters the `user_code`. UI calls `POST /v1/pair/complete` and receives `{session_id, session_token, relay_ws_url}`.
4) Attach: RAT2E connects `wss://…/v1/connect?device_code=…` with `Sec-WebSocket-Protocol: acp.jsonrpc.v1`. Browser connects `wss://…/v1/connect?session_id=…` (token not in URL) with allowed `Origin` and `Sec-WebSocket-Protocol: acp.jsonrpc.v1, stk.sha256=BASE64URL(SHA256(session_token))`. Relay validates and pairs sockets.
5) Secure: Browser and RAT2E complete Noise XX over WS. Prologue binds `{session_id, session_token}`. After success, only binary ciphertext flows; relay is a blind forwarder.
6) Control: Browser sends ACP `initialize` then `session/load` (if supported) to resume. ACP messages stream over the encrypted tunnel to/from the local agent.
7) Presence: Endpoints send periodic encrypted beats; relay exposes a presence snapshot so the dashboard shows ONLINE/OFFLINE.
8) Reconnect: Browser/network drops → browser reconnects and resumes deterministically; RAT2E keeps/renews its anchor connection with backoff.
9) Stop: User ends session; state expires via TTL sweeper; metrics/logs show clean shutdown; no plaintext ever stored server‑side.

## 3. System Overview (What)
- Topology: Browser UI ↔ Relay (WS 443) ↔ RAT2E. Relay performs WS upgrade (Origin + subprotocol gate), admission checks, and then blindly forwards Noise ciphertext frames between peers.
- Data: Pairing uses short‑lived RAM rows; no external DB. ACP flows unchanged inside the encrypted tunnel.

```mermaid
flowchart LR
  subgraph Browser["Browser Web UI"]
    BWS["WSS /v1/connect\nSec-WebSocket-Protocol: acp.jsonrpc.v1"]
    NBR["Noise XX Responder (WASM)"]
    ACPFE["ACP Client Shim\ninitialize / session.load"]
  end
  subgraph RAT["RAT2E (Rust)"]
    RWS["WSS /v1/connect (outbound 443)"]
    NRT["Noise XX Initiator"]
    ACP["ACP JSON-RPC Client"]
  end
  subgraph Relay["Relay (Rust, RAM-only)"]
    UP["WS Upgrader\nOrigin allow-list + subprotocol"]
    TUN["Blind Tunnel\nBinary (Noise ciphertext)"]
    REG["In-Mem Registry + TTL Sweeper"]
    OBS["/health /metrics /version"]
  end
  BWS --> UP
  RWS --> UP
  UP --> TUN
  TUN <-.ciphertext ACP JSON-RPC.-> ACP
  REG -. presence snapshot .-
```

## 4. Security Model (How)
- Admission (Relay auth): short‑TTL, single‑use `session_token` minted at `POST /v1/pair/complete`; validated on browser WS attach. Token: ≥128‑bit random, TTL ≤ 5 min, one‑time (`jti` cache), `aud`=relay, scope=`ws:connect`. [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750)
- End‑to‑End Peer Auth: Noise XX with static keys; each endpoint pins the other’s static pubkey from pairing and aborts on mismatch. Prologue binds `{session_id, session_token}` to prevent replay/downgrade. [Noise Protocol](https://noiseprotocol.org/noise.html)
- Browser guardrails: strict Origin allow‑list; require `Sec-WebSocket-Protocol: acp.jsonrpc.v1` and echo it; rate‑limit pairing and attaches; redact secrets in logs. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- File REST security: REST payloads terminate at the Relay (HTTP layer). The Relay immediately translates to File RPC and forwards over the encrypted tunnel; authorize and validate paths in RAT2E. For strict E2E on files in the future, stream file ops inside WS (no HTTP).
 - Channel binding (attach context): Hash the successful WS upgrade context and/or LB connection ID into the Noise prologue along with `{session_id, session_token}`. Example `transport_ctx = sha256(method || path || origin || sec-websocket-key || connection-id)`; prologue = `concat(session_id, session_token, transport_ctx, "rat2e-v1")`. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
 - Browser hardening: enforce CSP with `require-trusted-types-for 'script'` and a minimal Trusted Types policy; set `Referrer-Policy: no-referrer`. See Deployment Notes. [MDN: CSP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP) [MDN: trusted-types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types)

## 5. Components & Deliverables

### 5.1 Relay Service (Rust)
Functional requirements
- HTTP
  - `POST /v1/pair/start` → `{ user_code, device_code, relay_ws_url, expires_in, interval }`.
  - `POST /v1/pair/complete` → `{ session_id, session_token, relay_ws_url }`.
  - `GET /v1/presence/snapshot` → read‑only rows; no secrets.
  - Health/ops: `GET /health`, `GET /metrics` (Prometheus), `GET /version`.
- WebSocket
  - `GET /v1/connect` with query:
    - RAT2E: `?device_code=…`
    - Browser: `?session_id=…&token=…`
  - Validate Origin (browser only) and subprotocol; validate token (browser). Set ping/pong keepalive; idle timeout.
  - On pair present (both endpoints connected): forward binary frames blindly in both directions. Close cleanly on backpressure or peer disconnect.
- Registry/state (RAM only)
  - Maps: `user_code→device_code`, `device_code→{rat_pubkey,caps,exp}`, `session_id→{device_code, session_token, rat_tx?, web_tx?, created_ts}`, `presence` cache.
  - TTL sweeper (e.g., 5–10s cadence); enforce single‑use semantics for `user_code` and `session_token`.
- Presence
  - Accept encrypted presence beats; update `{last_seen, status}`; provide `/v1/presence/snapshot`.
- Observability
  - Metrics: `active_sessions`, `ws_open`, `presence_online`, `bytes_rx_total`, `bytes_tx_total`, `backpressure_closes_total`, `resume_latency_ms`, `pairing_rate`.
  - Structured logs without tokens/keys; error taxonomy for admission/idle/backpressure closes.

Non‑functional/Constraints
- Performance: attach/resume p50 ≤ 800 ms; first presence paint ≤ 1.2 s; OFFLINE ≤ 35 s after last beat.
- Security: no plaintext ACP in memory/logs; strict Origin; bounded queues; rate limits; TLS termination guidance.

Acceptance criteria
- Compiles and runs locally; pair/start→complete→attach success path; subprotocol gate enforced; token admission enforced; binary ciphertext only after Noise; metrics/health live.

### 5.2 RAT2E Client (Rust)
Functional requirements
- Pairing CLI (`--pair`): call `/v1/pair/start`; display `user_code`; connect WS with `device_code` and subprotocol header.
- WS client: set `Sec-WebSocket-Protocol: acp.jsonrpc.v1`; backoff and reconnect strategy.
- Noise: XX initiator; bind `{session_id, session_token}`; encrypt all post‑handshake ACP frames.
- ACP bridge: connect to local agent; support initialize/prompt/tools; handle resume semantics.
- Presence beats: periodic encrypted heartbeat.
- Config: relay URL, timeouts, backoff, optional disable‑relay (direct local mode).
- File RPC: implement handlers for `fs` envelope (list, read, write, md_list, md_batch) with path allow‑lists and symlink denial; respond over WS Noise transport.

Acceptance criteria
- End‑to‑end attach with hosted UI; ACP initialize succeeds; prompts round‑trip; presence beats emitted; reconnect tested.

### 5.3 Browser Client (MVP)
Functional requirements
- WS client: connect with `session_id`+`token`; set subprotocol; obey Origin policies.
- Noise: XX responder (WASM); bind `{session_id, session_token}`; encrypt/decrypt stream.
- ACP shim: `initialize` → `session/load` when available; render events; send prompts.
- Presence: show ONLINE/OFFLINE using `/v1/presence/snapshot`.
- File UX: call the browser‑facing REST endpoints; render results; respect error shapes.
 - Static key persistence: persist browser static Noise key in IndexedDB (origin‑scoped) for the lifetime of the session; rotate per new pairing; erase on session end or logout. On reload, reuse the same static key so XX pinning and resume succeed.

Acceptance criteria
- Works against the relay and local RAT2E; clean resume after reload; presence visible; no plaintext ACP outside endpoint memory.

### 5.4 File REST (Required)
Endpoints (browser‑facing, served by Relay)
- `GET /api/files?path=dir` — list directory
- `GET /api/file-content?path=path` — read small file
- `POST /api/save-file` — write text file `{ path, content }`
- `GET /api/markdown-files?path=dir` — list markdown files
- `POST /api/markdown-content` — batch read markdown files `{ files: string[] }`

Routing model (browser → relay → RAT2E)
- Browser calls the HTTP endpoint on the Relay.
- Relay translates the HTTP request into a File RPC message over the existing WS Noise tunnel to RAT2E (see 6.5), awaits response, and returns HTTP response to the browser.
- RAT2E implements the File RPC handlers and touches the local filesystem; Relay does not perform filesystem IO.

Constraints & security
- Path allow‑lists, `..` and symlink escapes denied, and size limits are enforced in RAT2E (the authority that owns the filesystem). Relay enforces basic request size limits and rate limits. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
- Large/binary transfers are out of scope for v1 (text files, small markdown); future work may stream within the WS tunnel for strict E2E.

## 6. Interface Specifications

### 6.1 Pairing (HTTP)
- Request — `POST /v1/pair/start`
  - Body: `{ rat_pubkey: base64, caps: string[], rat_version: string }`
  - Response: `{ user_code: string[A–Z0–9]{8}, device_code: uuid, relay_ws_url: string, expires_in: int, interval: int }`
- Request — `POST /v1/pair/complete`
  - Body: `{ user_code: string, browser_pubkey: base64 }`
  - Response: `{ session_id: uuid, session_token: string, relay_ws_url: string }`
  - Token: ≥128‑bit random; TTL ≤ 5 minutes; single‑use.
 - Semantics (RFC 8628 alignment): enforce minimum poll interval via `interval`; if the UI polls/attempts faster than allowed or submits wrong codes repeatedly, respond with `slow_down` and/or lockout the `user_code` temporarily; apply per‑IP throttles.

### 6.2 Attach (WebSocket)
- Path: `GET /v1/connect`
  - RAT2E query: `?device_code=…`
  - Browser query: `?session_id=…` (no token in URL)
- Headers:
  - `Sec-WebSocket-Protocol: acp.jsonrpc.v1, stk.sha256=BASE64URL(token_sha256)`
  - `Origin: https://…` (browser only)
- Server behavior:
  - Verify `Origin` exact‑match against allow‑list; else reject with 1008 (policy violation).
  - Require `acp.jsonrpc.v1` and a single `stk.sha256=` parameter; echo both values in 101; else reject with 1008.
  - Validate `session_id` row and verify SHA‑256 of the stored `session_token` equals `stk.sha256`; on success, mark token as used (single‑use) and proceed; else 1008.
- Lifecycle: disable `permessage-deflate`; enable ping/pong; idle timeout; bounded mpsc; see Close Codes & Backoff.

Correct attach example (token not in URL)

```http
GET /v1/connect?session_id=sess_123 HTTP/1.1
Host: relay.example
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Protocol: acp.jsonrpc.v1, stk.sha256=BASE64URL(SHA256(session_token))
Origin: https://ui.example.com

HTTP/1.1 101 Switching Protocols
Sec-WebSocket-Protocol: acp.jsonrpc.v1, stk.sha256=BASE64URL(SHA256(session_token))
# 1008 on any mismatch or missing values
# permessage-deflate not negotiated
```

### 6.3 Noise XX
- Pattern: XX; libs: `snow` (Rust) + WASM port in browser.
- Prologue binding (channel‑bound): `concat(session_id, session_token, transport_ctx, "rat2e-v1")` where `transport_ctx = sha256(method || path || origin || sec-websocket-key || connection-id)` is derived from the successful WS upgrade context.
- Key pinning: endpoints verify peer static keys exchanged during pairing; abort on mismatch.
- Transport: all application frames as WS Binary (ciphertext) after handshake.

### 6.3.1 Browser Static Key Management (IndexedDB, Option A)
- Noise static private key is generated as a non‑extractable WebCrypto `CryptoKey` (X25519) and stored directly in IndexedDB via structured clone. The browser performs DH via `SubtleCrypto` so the private key never exists as raw bytes in JS.
- Reuse the stored `CryptoKey` across reloads until session end; pair a new key on re‑pair.
- If X25519 is unsupported, it is out of scope for v1; do not store raw private key bytes.
- Rationale: mitigates XSS exfiltration risk and avoids storing raw private key material at rest. [MDN: CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey) [WICG: secure curves](https://wicg.github.io/webcrypto-secure-curves/)

Operational notes
- Enforce CSP with Trusted Types to reduce DOM‑XSS surface that could access IndexedDB.
- Handle storage eviction (e.g., Safari 7‑day) by prompting a graceful re‑pair when keys disappear; document PWA install as a mitigation.

### 6.4 Presence
- Beats: endpoint‑emitted encrypted messages (shape flexible, e.g., `{agent_id, ts, status}`) cached server‑side with TTL.
- Snapshot (authZ required): `GET /v1/presence/snapshot` requires `Authorization: Bearer viewer_token` with scope `presence:read` and is tenant‑scoped to the caller. The relay returns only rows the caller is authorized to see (typically sessions created by that user/tenant).

### 6.5 File RPC over Tunnel (Relay ⇄ RAT2E)
- Transport: reuses the established WS Noise transport between Relay and RAT2E.
- Envelope (JSON): `{ type: "fs", op: "list|read|write|md_list|md_batch", id: string, payload: object }`
- Requests
  - list: `{ path: string }`
  - read: `{ path: string, line?: number, limit?: number }`
  - write: `{ path: string, content: string }`
  - md_list: `{ path: string }`
  - md_batch: `{ files: string[] }`
- Responses
  - Success: `{ id: string, ok: true, result: any }`
  - Error: `{ id: string, ok: false, code: string, message: string }`
- Mapping to REST (Relay)
  - `GET /api/files` → `op=list`
  - `GET /api/file-content` → `op=read`
  - `POST /api/save-file` → `op=write`
  - `GET /api/markdown-files` → `op=md_list`
  - `POST /api/markdown-content` → `op=md_batch`

## 7. Data Model (RAM‑only)
- `user_codes: Map<user_code, { device_code, exp_ts }>`
- `devices: Map<device_code, { rat_pubkey_b64, caps[], exp_ts }>`
- `sessions: Map<session_id, { device_code, session_token, created_ts, rat_tx?, web_tx? }>`
- `presence: Map<agent_id, { last_seen, status, caps? }>`
- `viewers: optional Map<viewer_token_id, { tenant_id, scopes[], exp_ts }>`
- TTL sweeper: interval 5–10s; drop expired rows; enforce single‑use.

## 8. Lifecycles & State Machines
- Pairing: CREATED → CLAIMED (browser completed) → ATTACHED (both sockets present) → IDLE (browser gone) → EXPIRED.
- Attach: CONNECTING → AUTHENTICATING (Noise) → TRANSPORT (ciphertext) → CLOSING (idle/backpressure/error).
- Resume: Browser reconnects; runs `initialize` → `session/load`; relay keeps RAT2E side anchored; cleans up stale rows on timeout.

Close Codes & Backoff (RFC 6455)
- 1008 Policy violation: bad/missing Origin, bad/missing subprotocol, token proof mismatch.
- 1013 Try again later: backpressure close due to bounded queue overflow; client should backoff and retry.
- 1001 Going away: idle timeout.
- 1011 Internal error: unexpected server failure.
- Client backoff guidance: exponential backoff with jitter (e.g., base 250ms, factor 2.0, max 30s; jitter ±20%). RAT2E and browser should reset backoff after a stable 60s period.

## 9. Performance Targets & Limits
- Attach/resume p50 ≤ 800 ms; first presence paint ≤ 1.2 s; OFFLINE ≤ 35 s.
- Bounded queues per peer; close on overflow; log and metric increment `backpressure_closes_total`.
- Soak: 5k IDLE + 500 ACTIVE without error.

## 10. Observability
- Metrics: `active_sessions`, `ws_open`, `presence_online`, `bytes_rx_total`, `bytes_tx_total`, `backpressure_closes_total`, `resume_latency_ms`, `pairing_rate`.
- Endpoints: `/health` (200), `/metrics` (Prometheus), `/version`.
- Logging: structured, redacted; include decision points and close reasons.

## 11. Implementation Plan & Deliverables
M1 — Relay Skeleton
- Build warp server; implement `/v1/pair/start`, `/v1/pair/complete`, `/v1/connect` (WS), `/health`, `/metrics`, `/version`.
- In‑mem registry + TTL sweeper; basic metrics and structured logs.
- Deliverables: compiling binary; local manual E2E pairing; health/metrics visible.

M2 — Admission + Noise XX
- Mint/validate `session_token` with TTL and single‑use; enforce Origin+subprotocol gate; remove token from URL and validate `stk.sha256` subprotocol parameter; echo on success; disable permessage‑deflate.
- Implement Noise XX on both endpoints; bind `{session_id, session_token, transport_ctx}`; ciphertext‑only after handshake.
- Deliverables: golden vector tests; decrypt/encrypt round‑trip; relay never logs plaintext.

M3 — ACP Pass‑Through & Resume
- RAT2E bridges ACP to local agent; browser initialize → `session/load` resume.
- Deliverables: automated integration test for drop/reconnect/resume; `resume_latency_ms` recorded.

M4 — File REST Routing & Presence (with authZ)
- Encrypted beats to update presence cache; `/v1/presence/snapshot` endpoint; dashboard shows status.
- Implement browser‑facing REST endpoints on Relay and File RPC handlers in RAT2E; map REST to RPC; enforce limits/allow‑lists; round‑trip integration tests.
- Deliverables: presence TTL + hysteresis tests; OFFLINE within ≤ 35s.

M5 — Hardening & Ops
- Rate limits, backpressure closes, TLS guidance, docs and troubleshooting; soak/load tests.
- Deliverables: soak target met; security checklist satisfied.

Post‑v1 (Future)
- P2P fast‑path (ICE/QUIC, TURN‑like fallback) with quotas. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)
- File transfer streams; WebTransport/H3 exploration. [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441)

## 12. Test Plan
- Admission: Origin allow‑list (reject on missing/mismatch), subprotocol echo with `stk.sha256` present, token proof validation success/failure paths, and compression disabled verification.
- Noise: XX handshake, prologue binding, replay rejection; ciphertext‑only post‑handshake.
- Keys (browser): IndexedDB eviction handling (simulate Safari ITP eviction → triggers re‑pair path); ensure no raw private key bytes are ever stored or exposed; XSS defense: run with CSP + Trusted Types enabled and assert no violations during normal operation.
- ACP resume: drop/reload browser → `session/load` determinism; continuous `session/update` stream.
- Presence: TTL/hysteresis; multi‑client snapshot sanity; OFFLINE budget.
- Presence authZ: only tenant‑scoped rows returned with valid `presence:read` scope; unauthorized returns 401/403.
- File REST: REST→RPC mapping works; path allow‑list and symlink denial enforced in RAT2E; write/read round‑trip; size limit rejections; rate‑limits.
- Backpressure: forced overflow triggers close + metric.
- Soak: 5k IDLE + 500 ACTIVE stability.

## 13. Acceptance Criteria
- Browser attach never includes a bearer token in the URL; token proof is carried as `stk.sha256` in `Sec-WebSocket-Protocol`.
- 101 Switching Protocols echoes `acp.jsonrpc.v1` and the exact `stk.sha256` value sent by the browser; otherwise the server closes with 1008.
- `permessage-deflate` is not negotiated on authenticated sockets.
- Presence snapshot is tenant‑scoped and requires a viewer token with `presence:read`.
- Endpoints complete Noise XX with `{session_id, session_token, transport_ctx}` bound; ciphertext‑only frames thereafter.
- Resume is deterministic via ACP `initialize` then `session/load`.

## 14. Deployment Notes
- Single binary behind TLS/LB; sticky routing by `session_id`/`device_code` if needed; shard for scale; no external datastore.
- Default to hosted relay; allow self‑host for advanced users.
- Browser security headers: set `Content-Security-Policy` with `require-trusted-types-for 'script'` and a minimal Trusted Types policy; set `Referrer-Policy: no-referrer`.
- Storage eviction UX: document Safari’s 7‑day eviction edge and provide a graceful re‑pair path; recommend PWA install as mitigation.
