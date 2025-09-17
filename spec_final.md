# RAT2E Remote Control via Hosted Relay — Software Design Specification (v1)

This document defines the what, why, and how to ship RAT2E remote control via a tiny, hosted relay. It is implementation‑oriented: each component lists concrete deliverables, interfaces, constraints, and acceptance criteria.

## 0. Status of This Document (Informative)
- Short name: RAT2E-Relay Spec
- Version: v1.0.0 (2025-09-17)
- Editors: RAT team (contact: issues in repository)
- This is an Editor’s Draft intended for implementation; requirements may evolve behind SemVer pre-releases before 1.0.0. Issues and discussions are tracked in this repo.

## 0.1 Terminology & Notational Conventions (Normative)
- Key Words: The key words MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, NOT RECOMMENDED, MAY, and OPTIONAL in this document are to be interpreted as described in RFC 2119 and RFC 8174 when, and only when, they appear in all capitals.
- Definitions: RAT2E (local terminal client), Relay (hosted gateway), Browser Client (hosted UI running in a web browser), Session (paired RAT2E↔Browser binding), Session Token (single-use bearer minted at pair/complete), Presence Beat (encrypted heartbeat message), ACP (Agent Client Protocol JSON‑RPC).
- Notation: times are ISO‑8601 UTC; UUIDs per RFC 4122; base64url per RFC 7515; JSON examples are UTF‑8; header tokens are case‑insensitive per RFC 7230.

## 0.2 Conformance Model (Normative)
- Conformance Targets: this specification defines three roles — Relay, RAT2E Client, and Browser Client. A conforming implementation MUST implement the requirements tagged for its role.
- Requirement IDs: Normative statements are assigned IDs in the form RAT2E-REQ-###. Example: RAT2E-REQ-012: “The Relay MUST echo Sec-WebSocket-Protocol: acp.jsonrpc.v1 in the 101 response.”
- Profiles: Hosted vs Self‑Hosted deployments MAY restrict Origin allow‑lists and viewer tokens but MUST maintain wire compatibility.

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
4) Attach: RAT2E connects `wss://…/v1/connect?device_code=…` with `Sec-WebSocket-Protocol: acp.jsonrpc.v1`. Browser connects `wss://…/v1/connect?session_id=…&token=…` with allowed `Origin` and same subprotocol. Relay validates and pairs sockets.
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

### 6.3 Noise XX
- Pattern: XX; libs: `snow` (Rust) + WASM port in browser.
- Prologue: `concat(session_id, session_token, "rat2e-v1")`.
- Key pinning: endpoints verify peer static keys exchanged during pairing; abort on mismatch.
- Transport: all application frames as WS Binary (ciphertext) after handshake.

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
- Implement Noise XX on both endpoints; bind `{session_id, session_token}`; ciphertext‑only after handshake.
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

## 15. Security Considerations (Normative)
- Threats: token theft, URL leakage, origin spoofing, replay/downgrade, DoS via backpressure, plaintext logging. Mitigations include removing token from URL (stk.sha256 in subprotocol), strict Origin allow‑lists, channel binding in Noise prologue, bounded queues with explicit 1013 close, and redaction in logs.
- Endpoint Auth: static key pinning across session lifetime; abort on mismatch; bind session_id, session_token, and transport_ctx in Noise prologue to prevent replay/downgrade.
- Transport: disable permessage‑deflate on authenticated sockets; require exact subprotocol echo, else 1008; enforce idle timeouts.
- File REST: Relay sees file contents in v1; minimize exposure via rate limits, logging redaction, and planned migration to WS streaming for strict E2E.

## 16. Privacy Considerations (Normative)
- Metadata visible to Relay: connection times, IPs, sizes of frames, presence timestamps, and v1 file contents; no ACP plaintext. No server persistence beyond RAM TTL. Presence snapshot is tenant‑scoped and requires a viewer token.
- User identifiers: session_id and device_code are pseudonymous; do not log tokens, pubkeys, or content.

## 17. IANA‑like Considerations (Normative)
- Subprotocol tokens (e.g., acp.jsonrpc.v1, stk.sha256=...) and error code texts MAY later be registered in a public registry. Placeholder reserved for future policy.

## 18. Interoperability & Versioning (Normative)
- Wire Compatibility: minor versions MUST be backward compatible at the WS and HTTP header level; new features MUST be negotiated via optional headers or capability discovery.
- SemVer & Change Log: this spec uses SemVer 2.0.0. Breaking changes increment MAJOR.

## 19. Requirements Traceability (Normative)
- Example RTM row:
  - RAT2E-REQ-012 | 6.2 | WS-UPG-ECHO-001 | Integration | 101 includes echoed acp.jsonrpc.v1 and stk.sha256
- Full RTM will be maintained alongside tests.

## 20. References
### 20.1 Normative References
- RFC 6455: The WebSocket Protocol
- RFC 8174: Ambiguity of Uppercase Key Words
- RFC 3552: Security Considerations Guidelines
- RFC 9110: HTTP Semantics
- RFC 8441/8445/9000: WS over HTTP/2, ICE, QUIC (where cited)
- SemVer 2.0.0

### 20.2 Informative References
- Noise Protocol
- OWASP URL query leakage note
- W3C Manual of Style; ISO/IEC/IEEE 29148 and 42010; OASIS Conformance; RFC 8126; Bikeshed/ReSpec/spec‑prod

---
Appendix — External Review Notes

VERDICT
This version is strong. You closed the biggest hole by moving the browser token out of the URL and proving it via a `stk.sha256` parameter in `Sec-WebSocket-Protocol`, you added presence authZ, explicit close codes, channel binding guidance, and a concrete IndexedDB plan. You now meet the user story technically, with a few tidy-ups below. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [OWASP](https://owasp.org/www-community/vulnerabilities/Information_exposure_through_query_strings_in_url) [Noise Protocol (PDF)](https://noiseprotocol.org/noise.pdf) [ACP Overview](https://agentclientprotocol.com/protocol/overview)

WHAT IMPROVED
- Token handling: URL query no longer carries the bearer. Validation via `stk.sha256` and echo on 101 is the right pattern. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [OWASP](https://owasp.org/www-community/vulnerabilities/Information_exposure_through_query_strings_in_url)
- Presence: endpoint-emitted encrypted beats with tenant-scoped `/v1/presence/snapshot` fix prior leakage concerns. [ACP Overview](https://agentclientprotocol.com/protocol/overview)
- Close semantics: 1008, 1013, 1001, 1011 plus jittered backoff removes ambiguity in overload and policy cases. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Compression: you explicitly disable `permessage-deflate` on authenticated sockets. Good. [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)
- Channel binding: hashing upgrade context into the Noise prologue strengthens binding beyond token+session. [Noise Protocol (PDF)](https://noiseprotocol.org/noise.pdf) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

NITS AND PATCHES TO APPLY
1) Keep the doc consistent: Section 2 step 4 must not show `?session_id=…&token=…`. It now aligns with 6.2; token removed from URL. [OWASP](https://owasp.org/www-community/vulnerabilities/Information_exposure_through_query_strings_in_url)
2) Make the echo rule explicit: 101 must echo `acp.jsonrpc.v1` and the exact `stk.sha256` value or fail with 1008. This is captured in Acceptance Criteria. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
3) File REST clarity: v1 REST terminates at the relay; file contents are visible to the relay process. Strict E2E for files will migrate to streaming-over-WS inside the Noise tunnel. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
4) Browser hardening: adopt CSP with `require-trusted-types-for 'script'` and a Trusted Types policy; also set `Referrer-Policy: no-referrer`. Documented in Components and Deployment Notes. [MDN: CSP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP) [MDN: trusted-types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types)
5) Storage eviction UX: Safari 7‑day eviction edge is documented with a graceful re‑pair path and PWA note. [MDN: Storage quotas and eviction](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria) [web.dev: Storage for the web](https://web.dev/articles/storage-for-the-web)

Corrected browser attach example (token not in URL)
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

DOES THIS SPEC NOW SATISFY THE USER STORY
Yes. Hosted UI controls a local RAT2E through a tiny relay on 443 with a blind tunnel, ACP rides the pipe, and resume is deterministic via `session/load`. Presence is scoped and observable. The only must-do was removing the stale token-in-URL example in Section 2 for consistency (fixed). [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol (PDF)](https://noiseprotocol.org/noise.pdf) [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

INDEXEDDB KEY MANAGEMENT — IS IT GOOD ENOUGH
Short answer: IndexedDB is fine if you store keys as non-extractable `CryptoKey` objects or you store wrapped bytes under a hardware-backed secret. Raw private key bytes in IndexedDB are not acceptable. [MDN: CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey) [UDN: SubtleCrypto storing keys](https://udn.realityripple.com/docs/Web/API/SubtleCrypto) [MDN: Structured clone](https://developer.mozilla.org/en-US/docs/Web/API/Web_Workers_API/Structured_clone_algorithm)

Option A — Best when feasible: keep the Noise static private key as a non-extractable `CryptoKey` and store that object directly in IndexedDB via structured clone. Use WebCrypto X25519 for DH so the private key never becomes raw bytes. Your WASM calls delegate DH to `SubtleCrypto`. Pros: key material is non-extractable at rest and in JS; mitigates XSS exfil of the private key. Cons: requires wiring the Noise ops to WebCrypto and browser support for X25519. [MDN: generateKey X25519](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/generateKey) [WICG: secure curves](https://wicg.github.io/webcrypto-secure-curves/)

Option B — Next best: if your Noise stack needs raw bytes, wrap the Noise private key before storing. Derive a wrapping key from a WebAuthn PRF output (user-verified), HKDF it to AES-GCM, and store `AES-GCM(wrapped_noise_private_key)` in IndexedDB. On use, prompt WebAuthn, unwrap in memory, zeroize after handshake. Pros: at-rest protection tied to hardware key presence; stops silent at-rest theft even with IndexedDB access. Cons: requires a WebAuthn ceremony on reuse and PRF support. [WebAuthn L3](https://www.w3.org/TR/webauthn-3/) [Yubico PRF guide](https://developers.yubico.com/WebAuthn/Concepts/PRF_Extension/) [Bitwarden PRF explainer](https://bitwarden.com/blog/prf-webauthn-and-its-role-in-passkeys/)

Option C — Avoid: storing raw private key bytes unwrapped in IndexedDB. This is trivially exfiltrated by XSS, and can be evicted silently on Safari without recent interaction. [OWASP](https://owasp.org/www-community/vulnerabilities/Information_exposure_through_query_strings_in_url) [MDN: Storage quotas and eviction](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria)

Operational notes: whichever option you choose, enforce a strict CSP with Trusted Types to shrink the DOM-XSS surface that could otherwise exfiltrate IndexedDB contents; add recovery for eviction by re-pairing when keys disappear; and keep the browser static key per session pairing to preserve pinning. [MDN: CSP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP) [MDN: trusted-types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types) [web.dev: Trusted Types](https://web.dev/articles/trusted-types)

RECOMMENDED KEY PLAN FOR THIS SPEC
1) Prefer Option A: implement Noise DH via WebCrypto X25519, generate a non-extractable private `CryptoKey`, store the `CryptoKey` object in IndexedDB, and reuse it across reloads until session end. Fall back to Option B when X25519 is unsupported. [MDN: generateKey X25519](https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto/generateKey) [WICG: secure curves](https://wicg.github.io/webcrypto-secure-curves/)
2) If using Option B: create a passkey during pairing and use WebAuthn PRF to derive the wrapping key; store only AES-GCM-wrapped key bytes in IndexedDB; require user verification to unwrap. [WebAuthn L3](https://www.w3.org/TR/webauthn-3/) [Yubico PRF guide](https://developers.yubico.com/WebAuthn/Concepts/PRF_Extension/)
3) Add CSP and Trusted Types in the browser client build; document the required headers in Deployment Notes. [MDN: CSP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP) [web.dev: Trusted Types](https://web.dev/articles/trusted-types)

TEST PLAN ADDITIONS FOR KEYS
- Verify IndexedDB eviction handling by simulating Safari ITP eviction; ensure re-pair flow re-provisions the key. [MDN: Storage quotas and eviction](https://developer.mozilla.org/en-US/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria)
- WebAuthn PRF path: fail-open tests must not reveal raw private key if PRF is unavailable; fail with a clear UX prompt instead. [WebAuthn L3](https://www.w3.org/TR/webauthn-3/)
- XSS defense: enable CSP with Trusted Types in tests and assert no violations during normal operation. [MDN: trusted-types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types)

---
Self‑Check
- E2E preserved (Noise XX); relay is blind forwarder.
- Browser‑correct WS (Origin + subprotocol) and token‑gated admission.
- ACP semantics unchanged; deterministic resume via `session/load`.
- RAM‑only with TTL sweeper; presence snapshot; metrics and health.
- Future paths noted, excluded from v1 scope.
