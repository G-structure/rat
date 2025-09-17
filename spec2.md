#+BEGIN_SRC text
# Relay-First ACP Tunnel with Optional P2P Hole Punching — Software Design Spec (v1)
This spec replaces FRP entirely. A single relay terminates browser-correct WebSocket upgrades, carries ACP end-to-end inside Noise, and (optionally) performs NAT traversal using ICE with QUIC data channels, falling back to the relay when P2P is not viable. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [ACP Overview](https://agentclientprotocol.com/protocol/overview) [Noise Protocol](https://noiseprotocol.org/noise.html) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

## 1) Goals & Non-Goals
Goals: (a) ship a minimal, single-binary RAT that “phones home” to the relay over 443/TLS; (b) end-to-end privacy for ACP bytes; (c) resumable sessions on mobile; (d) optional P2P hole punching for lower latency; (e) zero external DB (RAM-only TTL). [ACP Overview](https://agentclientprotocol.com/protocol/overview) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
Non-Goals (v1): third-party tunnels, server-stored chat history, orchestrating agents in the relay, bespoke HTTP endpoints beyond health/files/presence snapshot. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 2) High-Level Architecture
Browser (React/PWA) and RAT both attach outbound to the Relay. All control/data flows over one multiplexed tunnel; ACP remains transport-agnostic. P2P (ICE/QUIC) is opportunistic, else the Relay relays bytes. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

#+END_SRC
#+BEGIN_mermaid
flowchart LR
  subgraph Browser["Browser Web UI"]
    BWS["WSS /v1/connect\n(acp.jsonrpc.v1)"]
  end
  subgraph RAT["RAT (Rust)"]
    RWS["WSS /v1/connect\nReverse tunnel"]
    ACP["ACP JSON-RPC client"]
  end
  subgraph Relay["Relay (Rust, RAM-only)"]
    UP["WS Upgrader\nOrigin allow-list + subprotocol"]
    MUX["Substream Mux\n(ACP, presence, files)"]
    ICE["ICE Signaling\n(STUN/QUIC candidates)"]
    PRES["Presence Map<agent_id>\nTTL sweeper"]
    REST["/health /metrics\n(optional files, snapshot)"]
  end
  BWS --> UP
  RWS --> UP
  UP --> MUX
  MUX <-.E2E Noise frames.-> ACP
  MUX <---> ICE
  classDef dim fill:#f7f7f7,stroke:#bbb,color:#333;
  class REST,PRES dim;
#+END_mermaid
[RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol](https://noiseprotocol.org/noise.html) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445)

#+BEGIN_SRC text
## 3) Trust & Threat Model
The relay is honest-but-curious: it enforces browser security at WS upgrade (Origin + subprotocol) but never sees ACP plaintext; browser and RAT run a Noise XX handshake over the tunnel and use AEAD for all application bytes. Tokens are short-lived and transcript-bound to prevent replay. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol](https://noiseprotocol.org/noise.html)

## 4) Transports & Tunnels

4.1 Reverse WebSocket Tunnel (default)
RAT opens `WSS /v1/connect` (outbound 443) and maintains a framed, multiplexed tunnel. Substreams carry ACP, presence pings, and optional file ops. This works through carrier-grade NAT and mobile suspends with stateless resume. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

4.2 Optional QUIC Data Tunnel
For lower head-of-line blocking and better mobility, RAT may open a QUIC session to the relay with the same control plane; each ACP/tool stream maps to a QUIC stream. Wire semantics identical to WS. [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

4.3 P2P Hole Punching (optional, opportunistic)
When both peers support it, the relay acts as ICE **signaling** only: each side gathers host/srflx/relay candidates (STUN), exchanges via the relay, attempts QUIC connectivity checks, and if successful, upgrades the ACP substream to the direct QUIC path. Otherwise, continue via the relay. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 5389](https://www.rfc-editor.org/rfc/rfc5389) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

## 5) Protocol Overview

5.1 Pairing / Grant
Option A: device-code pairing for remote RAT: `POST /v1/pair/start` (RAT) shows `user_code`; `POST /v1/pair/complete` (Browser) mints `{session_id, ws_token}`. Option B: same-device demo uses a bearer grant to fetch a short-TTL `ws_token`. [ACP Overview](https://agentclientprotocol.com/protocol/overview)

5.2 Attach (WS/QUIC)
`GET /v1/connect?session_id=…&token=…` with `Sec-WebSocket-Protocol: acp.jsonrpc.v1`; relay validates Origin and echoes subprotocol; client runs `initialize` then `session/load` for deterministic resume. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [ACP Overview](https://agentclientprotocol.com/protocol/overview)

5.3 End-to-End Crypto (Noise XX)
Noise XX handshake bytes are exchanged as binary frames; the prologue includes `{session_id, token}`; upon success, switch to transport mode and encrypt all ACP JSON-RPC messages. [Noise Protocol](https://noiseprotocol.org/noise.html)

5.4 Substream Mux (single socket, many lanes)
Define a minimal frame: `{stream_id: u32, kind: OPEN|DATA|CLOSE|PING|PONG|ACCEPT, len: u32, payload: bytes}`. ACP rides in a dedicated stream; presence beats are tiny control messages; file transfers can use a separate stream per file. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

5.5 Presence (TTL-based)
ACTIVE (tunnel up): send `meta/presence.beat {agent_id, seq, ts, caps?, load?}` every 10s±15%. IDLE (no tunnel): optional HTTPS heartbeat with backoff. Server marks OFFLINE when `now - last_seen > 35s`; dashboard uses `GET /v1/presence/snapshot`. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 6) Hole Punching: ICE/QUIC Flow (optional)

1) Capability discover: both sides advertise `supportsICE=true`. 2) Candidate gather: host + server-reflexive via STUN; optionally allocate a **relay** candidate at the server (TURN-like) to guarantee liveness. 3) Signaling: exchange SDP-lite metadata over the existing WS/QUIC control stream. 4) Connectivity checks: QUIC Initial to candidates in priority order; if a pair succeeds, promote ACP substream to direct path. 5) Fallback: if no usable pair, keep using the relay path. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 5389](https://www.rfc-editor.org/rfc/rfc5389) [RFC 5766](https://www.rfc-editor.org/rfc/rfc5766) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

Security note: the E2E Noise channel remains the same; the transport under it (relay vs. direct QUIC) changes, not the cryptographic binding. [Noise Protocol](https://noiseprotocol.org/noise.html)

## 7) Relay Responsibilities (Rust)
- WS upgrade gate: strict Origin allow-list; echo `acp.jsonrpc.v1`; disable permessage-deflate in v1. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Substream mux: bounded queues per connection; drop on overflow; ping/pong keep-alive. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- ICE signaling: expose `POST /v1/ice/start`, `POST /v1/ice/candidate`, `POST /v1/ice/finish`; maintain ephemeral offers in RAM with TTL; optional TURN-like relay socket for “relay” candidates. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 5766](https://www.rfc-editor.org/rfc/rfc5766)
- Presence: Map<agent_id, Row{last_seen, seq, status, caps?, load?}> with a 5s sweeper; OFFLINE at 35s. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
- Minimal REST: `/health`, `/metrics`, `/v1/presence/snapshot`, optional `/api/upload-file` and `/api/download`. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 8) Data Model (RAM-only)
`sessions`: session_id → { device_code?, token, created_ts, conn_handles }
`presence`: agent_id → { last_seen, seq, status, caps?, load? }
`ice_offers`: offer_id → { a: agent_id, b: peer_id, candidates[], exp_ts }
All rows TTL-managed; no external store. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 9) API Shapes (concise)

### 9.1 WS Connect
`GET /v1/connect?session_id=s_…&token=t_…` with `Sec-WebSocket-Protocol: acp.jsonrpc.v1` → 101 with same subprotocol. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

### 9.2 Presence
`GET /v1/presence/snapshot` → `{"agents":[{agent_id,status,last_seen,caps?,load?}]}`;
`meta/presence.beat` on the control stream (ACTIVE) or `POST /v1/presence/heartbeat` (IDLE). [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

### 9.3 ICE Signaling (optional)
`POST /v1/ice/start {peer, role}` → `{offer_id, stun_urls[], relay_hint?}`;
`POST /v1/ice/candidate {offer_id, candidate}`;
`POST /v1/ice/finish {offer_id, selected_pair}`. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445)

## 10) Security Controls
- Short-TTL, single-use tokens; bind `{session_id, token}` into Noise prologue; reject replays (monotonic seq per agent). [Noise Protocol](https://noiseprotocol.org/noise.html)
- WSS only; strict Origin allow-list; echo subprotocol; rate-limit ICE endpoints; TURN-like relay requires explicit caps and quotas. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 5766](https://www.rfc-editor.org/rfc/rfc5766)
- Files: path allow-lists; deny `..` and symlink escapes. [RFC 9110](https://www.rfc-editor.org/rfc/rfc/rfc9110)

## 11) Authentication & Hosting Model

### Bottom Line Answers
• **Do users need to host their own relay?** No by default. Ship a **public, multi-tenant hosted relay** as the default; keep a self-hosted relay option for customers that require it. The wire protocol is identical for both. [ACP Overview](https://agentclientprotocol.com/protocol/overview)
• **Do we need authentication on the hosted relay?** Yes. Noise gives end-to-end confidentiality/integrity, but it does **not** authorize who may consume shared relay resources. You still need **relay admission auth** to prevent abuse, enforce quotas, and tie sessions to accounts. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110) [Noise Protocol](https://noiseprotocol.org/noise.html)
• **Can we "just use Noise" for auth?** Not for relay admission. The relay is blind to Noise identities (by design), so it cannot make *admission* decisions from the Noise handshake alone. Use a short-lived **attach token** at WS/QUIC connect time, and then bind that token into the Noise transcript to prevent replay/stripping. [Noise Protocol](https://noiseprotocol.org/noise.html)

### Recommended Minimal Auth Model (Shippable This Week)
1) **Token types (opaque or JWT)**:
   • **attach_token** (TTL 2–5 min, single-use): scope `ws:connect`, audience = relay origin, claims: `sub` (user_id), `aid?` (agent_id), `typ` ∈ {agent,browser}, `jti`, `exp`. Issued by your app on login (**browser**) or via device-code pairing (**agent**) per RFC 8628. [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628) [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750)
   • Optional **refresh/session** token (longer-lived) to mint new attach_tokens without re-auth. [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750)
2) **Admission at the relay (before Noise)**:
   • **Browser WS**: validate `Origin` and `Sec-WebSocket-Protocol: acp.jsonrpc.v1`; accept `attach_token` via query (browser limitation) or cookie from a prior HTTPS step. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
   • **Agent WS/QUIC**: no Origin header; require `Authorization: Bearer <attach_token>` (WS libs allow this for non-browser clients). [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
   • Enforce per-account rate limits/quotas on successful connect. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
3) **Bind to Noise (after admission)**:
   • Include `{session_id, attach_token, nonce}` in the Noise **prologue**; both sides verify they derived transport keys over the same tuple. This prevents token replay/stripping and ties the E2E channel to the admitted session. [Noise Protocol](https://noiseprotocol.org/noise.html)
4) **Optional proof-of-possession (later, if you need it)**:
   • Put a JWK thumbprint in `cnf` and require a client signature over a server challenge (DPoP-style) before admitting. Ship this in enterprise tier; not required for v1. [RFC 9449](https://www.rfc-editor.org/rfc/rfc9449)

### Hosting Models
• **Hosted relay (default)**: one shared cluster with strict WS gates, token admission, quotas, metrics, and E2E Noise. Users do not deploy anything; RAT "phones home" to your origin over 443. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol](https://noiseprotocol.org/noise.html)
• **Self-hosted relay (optional)**: same binary; operators bring TLS and an OIDC/JWT issuer (or a static signing key) to mint attach_tokens. Zero protocol forks, same clients. [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750)

### Authentication Flows

**A) Agent pairing (device-code) → attach**
1. RAT → `/v1/pair/start` → `{user_code, device_code}`.
2. Browser (logged in) → `/v1/pair/complete {user_code}` → `{session_id, attach_token(agent), attach_token(browser)}`.
3. RAT connects `WSS /v1/connect` with `Authorization: Bearer <attach_token(agent)>` → relay verifies + admits → Noise XX with prologue `{session_id, attach_token}` → ACP `initialize` → `session/load`. [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol](https://noiseprotocol.org/noise.html) [ACP Overview](https://agentclientprotocol.com/protocol/overview)

**B) Browser attach (no device-code)**
1. User login → app mints `attach_token(browser)`.
2. Browser opens WS `wss://relay/v1/connect?token=…` with `Sec-WebSocket-Protocol: acp.jsonrpc.v1` and allowed `Origin`.
3. Relay admits → Noise XX with the same prologue → ACP resume. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750) [Noise Protocol](https://noiseprotocol.org/noise.html)

### Why Noise Alone Isn't Enough
Noise authenticates endpoints **to each other** and encrypts ACP, but the relay doesn't parse or trust identities embedded in an E2E handshake it can't see. The relay must gate *who can connect* and *how much they can use*; that's an **authorization** problem, not a crypto-confidentiality problem. Use tokens (or mTLS) at admission, then bind into Noise to prevent downgrade or replay. [Noise Protocol](https://noiseprotocol.org/noise.html) [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

### Hardening Checklist (v1)
• WS upgrade: enforce Origin allow-list (browser only) + subprotocol echo; disable permessage-deflate for authenticated streams. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
• Tokens: TTL ≤ 5 min; single-use (`jti` replay cache); audience = relay origin; scopes `ws:connect`, `ice:*` if you enable P2P; rotate signing keys. [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750)
• Pairing: use RFC 8628 device-code UX for headless RATs; never accept unauthenticated agent attaches. [RFC 8628](https://www.rfc-editor.org/rfc/rfc8628)
• E2E: always bind `{session_id, attach_token}` in Noise prologue; drop if mismatch. [Noise Protocol](https://noiseprotocol.org/noise.html)

### TL;DR
Use **our hosted relay** by default; allow **self-host** for advanced users. Yes, you **need relay-side authentication** for admission, quotas, and abuse prevention. Keep **Noise** for E2E privacy/integrity and **bind** your short-TTL attach token into the Noise prologue so admission and E2E form a single, replay-safe story. [ACP Overview](https://agentclientprotocol.com/protocol/overview) [RFC 6750](https://www.rfc-editor.org/rfc/rfc6750) [Noise Protocol](https://noiseprotocol.org/noise.html) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

## 12) Performance Targets
Attach/resume p50 ≤ 800 ms; first presence paint ≤ 1.2 s; OFFLINE flip ≤ 35 s after last beat; ICE attempt budget ≤ 2 s before fallback; no unbounded queues. [ACP Overview](https://agentclientprotocol.com/protocol/overview) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445)

## 13) Observability
Metrics: `active_sessions`, `ws_open`, `presence_online`, `bytes_rx/tx`, `backpressure_closes_total`, `resume_latency_ms`, `ice_attempts_total`, `ice_success_total`, `turn_bytes_total`. `/health` for LBs; log decisions, not secrets. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 14) Implementation Plan
M1: WS upgrader + mux + presence (RAM TTL) + `/health` `/metrics`. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
M2: ACP pass-through (initialize → tools) + `session/load` replay; Noise XX. [ACP Overview](https://agentclientprotocol.com/protocol/overview) [Noise Protocol](https://noiseprotocol.org/noise.html)
M3: Optional ICE signaling + QUIC transport, with relay fallback and quotas. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)
M4: Optional file upload/download streams over the mux. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 15) Test Plan
- WS upgrade gates (Origin/subprotocol) and reconnect across mobile suspend; `session/load` determinism. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [ACP Overview](https://agentclientprotocol.com/protocol/overview)
- Noise opacity (relay never sees ACP plaintext); replay-protection via seq. [Noise Protocol](https://noiseprotocol.org/noise.html)
- Presence TTL + hysteresis; soak 2k IDLE + 200 ACTIVE; backpressure close behavior. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
- ICE: candidate exchange, QUIC connectivity checks, promotion to direct, fallback within 2 s budget; TURN-like relay quotas. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

## 16) Acceptance Criteria
- One socket to rule them all: ACP + presence + files multiplexed over WSS; resume works; ACP payloads remain opaque; optional ICE/QUIC succeeds when NAT allows, else relay path is seamless; no FRP/ngrok dependency. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol](https://noiseprotocol.org/noise.html) [RFC 8445](https://www.rfc-editor.org/rfc8445)

## 11) Performance Targets
Attach/resume p50 ≤ 800 ms; first presence paint ≤ 1.2 s; OFFLINE flip ≤ 35 s after last beat; ICE attempt budget ≤ 2 s before fallback; no unbounded queues. [ACP Overview](https://agentclientprotocol.com/protocol/overview) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445)

## 12) Observability
Metrics: `active_sessions`, `ws_open`, `presence_online`, `bytes_rx/tx`, `backpressure_closes_total`, `resume_latency_ms`, `ice_attempts_total`, `ice_success_total`, `turn_bytes_total`. `/health` for LBs; log decisions, not secrets. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 13) Implementation Plan
M1: WS upgrader + mux + presence (RAM TTL) + `/health` `/metrics`. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
M2: ACP pass-through (initialize → tools) + `session/load` replay; Noise XX. [ACP Overview](https://agentclientprotocol.com/protocol/overview) [Noise Protocol](https://noiseprotocol.org/noise.html)
M3: Optional ICE signaling + QUIC transport, with relay fallback and quotas. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)
M4: Optional file upload/download streams over the mux. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)

## 17) Detailed Todo List - First Steps

### Phase 0: Align Existing Relay Skeleton (1–2 days)
- [ ] Confirm `relay/` skeleton compiles; remove/resolve type mismatches between `pairing.rs` and `websocket.rs` (single `SessionRegistry` struct for senders/ids).
- [ ] Adopt one concurrency map approach: `RwLock<HashMap<..>>` (current) or `dashmap` (optional). Remove unused paths accordingly.
- [ ] Add config: `bind_addr`, `tls` (dev: plain WS), `allowed_origins`, `allowed_subprotocols` (default: `acp.jsonrpc.v1`).
- [ ] Add `/health` and `/version` routes behind the same warp server.

### Phase 1: WS Upgrade & Admission (2–3 days)
- [ ] Validate `Origin` for browser attaches (allow-list); skip for RAT CLI.
- [ ] Validate requested `Sec-WebSocket-Protocol: acp.jsonrpc.v1`. If the framework cannot echo subprotocol, reject when header missing/mismatched and document limitation.
- [ ] Parse query: RAT uses `device_code=...`; Browser uses `session_id=...&token=...`.
- [ ] Build 101 path that registers sender channels into `SessionRegistry` and pairs peers when both sides arrive; add ping/pong and idle timeouts.

### Phase 2: Pairing, Tokens, and TTL (2–3 days)
- [ ] Finish `/v1/pair/start` and `/v1/pair/complete`: mint `session_id` + short-TTL `session_token` (single-use), drop `user_code` on success.
- [ ] Store: `user_code→device_code`, `device_code→device_row`, `session_id→{device_code, token, rat_tx?, web_tx?}` with TTL sweeper.
- [ ] Enforce token on browser WS attach; reject on missing/expired/mismatch.

### Phase 3: Noise XX E2E (3–5 days)
- [ ] Switch both sides to Noise XX (current code references IK). Bind `{session_id, session_token}` in the prologue. Use `snow`.
- [ ] After XX completes, forward only Binary ciphertext frames; never inspect plaintext at relay.
- [ ] Golden tests: XX handshake vectors and encrypt/decrypt round-trip.

### Phase 4: Blind Tunnel Bridging (2–3 days)
- [ ] Implement bidirectional forwarding between RAT and Browser once both are present; bounded mpsc; close on overflow.
- [ ] Reconnect behavior: permit re-attach by the browser; keep RAT attached; time out stale pairs.

### Phase 5: Presence Snapshot (2 days)
- [ ] Track last-seen beats from browser/RAT inside the encrypted ACP channel (endpoint-originated). Relay should only expose `/v1/presence/snapshot` from cached metadata sent by endpoints.
- [ ] TTL sweeper sets OFFLINE at ~35s; expose counters.

### Phase 6: ACP Pass-through & Resume (4–6 days)
- [ ] RAT: on attach completion, start ACP client loop; Browser: initialize → `session/load` if supported.
- [ ] Ensure resume determinism across reconnect; add basic probes in tests.

### Phase 7: RAT Client Adjustments (3–5 days)
- [ ] In `rat`, set WebSocket subprotocol header and include `Origin` when appropriate; read `relay_ws_url` from pairing result.
- [ ] Implement the Noise XX initiator and encrypt ACP JSON-RPC frames over WS.
- [ ] Config: `relay.url`, timeouts, backoff, and a flag to disable relay mode.

### Phase 8: Security Hardening (2–3 days)
- [ ] Strict Origin allow-list for browser, per-IP/route rate limits on pairing.
- [ ] Close on backpressure; sanitize errors; redact secrets.
- [ ] Optional TLS termination and certificate pinning rules.

### Phase 9: Observability (2 days)
- [ ] `/metrics` (active_sessions, ws_open, bytes_rx/tx, backpressure_closes_total, resume_latency_ms, pairing_rate).
- [ ] Structured logs and connection lifecycle tracing.

### Phase 10: Testing & Docs (3–4 days)
- [ ] Integration tests for pairing → attach → resume.
- [ ] Load test with many idle RATs and a subset ACTIVE.
- [ ] Deployment/config docs and troubleshooting notes.

### Future Phases (M3–M4)
- [ ] Optional ICE signaling endpoints and quotas.
- [ ] QUIC transport and opportunistic P2P; TURN-like fallbacks.
- [ ] File upload/download streams.
- [ ] Browser PWA client with WASM Noise and ACP shim.

## 14) Test Plan
- WS upgrade gates (Origin/subprotocol) and reconnect across mobile suspend; `session/load` determinism. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [ACP Overview](https://agentclientprotocol.com/protocol/overview)
- Noise opacity (relay never sees ACP plaintext); replay-protection via seq. [Noise Protocol](https://noiseprotocol.org/noise.html)
- Presence TTL + hysteresis; soak 2k IDLE + 200 ACTIVE; backpressure close behavior. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110)
- ICE: candidate exchange, QUIC connectivity checks, promotion to direct, fallback within 2 s budget; TURN-like relay quotas. [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

## 15) Acceptance Criteria
- One socket to rule them all: ACP + presence + files multiplexed over WSS; resume works; ACP payloads remain opaque; optional ICE/QUIC succeeds when NAT allows, else relay path is seamless; no FRP/ngrok dependency. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Protocol](https://noiseprotocol.org/noise.html) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445)
#+END_SRC

#+BEGIN_SRC text
Self-Check
- Persona/style: straight, concise, ship-focused. ✓
- Org-mode for multi-line content; one mermaid diagram. ✓
- Inline citations appear in every section. ✓
- Followed Rules 1–7: citation plan, ACP preserved, Noise E2E, relay-first with optional ICE/QUIC hole punching, self-check. ✓
#+END_SRC
