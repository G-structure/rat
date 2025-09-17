#+TITLE: RAT2E Remote Control via Hosted Relay — Software Design Specification
#+SUBTITLE: Version 1.0.0
#+AUTHOR: RAT2E Working Group
#+DATE: 2025-09-16
#+OPTIONS: toc:3 num:t ^:nil
#+LANGUAGE: en
#+PROPERTY: header-args :results none :exports code

* Status of This Document
This is a living engineering specification intended for internal and partner implementation. It uses normative key words per BCP 14 and distinguishes Normative vs Informative sections. It is not an IETF standard. [BCP 14](source) [RFC 7322](source) [W3C Manual of Style](source)

* Abstract
RAT2E enables secure remote control of a local agent from a hosted browser UI via a lightweight relay. Both the browser and RAT2E connect outbound over TLS on port 443, establish an end-to-end encrypted channel using Noise XX over a WebSocket tunnel, and carry ACP JSON-RPC messages for interactive UX. Admission is enforced at the relay with short-TTL bearer “attach tokens.” The relay is a blind forwarder of ciphertext and maintains only RAM-based TTL state. Presence is exposed via a read-only snapshot. Optional ICE/QUIC is deferred to future work. [RFC 6455](source) [RFC 6750](source) [RFC 8628](source) [RFC 9110](source)

* Notational Conventions
The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “NOT RECOMMENDED”, “MAY”, and “OPTIONAL” are to be interpreted as described in BCP 14 when, and only when, they appear in all capitals. [BCP 14](source)

* Scope
This document specifies interfaces, behaviors, and conformance criteria for:
- Relay Service implementation.
- RAT2E client.
- Browser Web UI.
It excludes P2P fast-paths and persistent server-side chat history in v1. [RFC 7322](source)

* Conformance
** Conformance Targets
- CT-RELAY: the hosted Relay Service.
- CT-RAT: the RAT2E client.
- CT-WEB: the Browser UI. [OASIS Conformance Guidelines](source)
** Conformance Levels
- “Compliant”: all MUST requirements satisfied.
- “Conditionally Compliant”: all MUST except those gated by an explicitly disabled optional feature. [OASIS Conformance Guidelines](source)
** Requirements Identification
Normative requirements are labeled =RAT2E-REQ-###=. Test mapping appears in the RTM. [RFC 7322](source)

* System Overview (Informative)
Topology: Browser UI ↔ Relay (WSS 443) ↔ RAT2E. The relay performs WebSocket upgrade checks (Origin allow-list and subprotocol), admits connections with an attach token, then blindly forwards binary frames. All application bytes are encrypted end-to-end by a Noise XX transport. ACP JSON-RPC runs unmodified on this tunnel. [RFC 6455](source) [Noise Protocol](source)

#+BEGIN_mermaid
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
#+END_mermaid
[RFC 6455](source) [RFC 9110](source)

* Roles and Responsibilities (Informative)
- CT-WEB: manage WS connect, perform Noise XX as responder, implement ACP client and reconnection. [RFC 6455](source)
- CT-RAT: initiate Noise XX, maintain anchor connection, bridge ACP to local agent, emit presence. [Noise Protocol](source)
- CT-RELAY: validate Origin and subprotocol, admit connections with tokens, forward ciphertext, manage RAM TTL state and metrics. [RFC 6455](source) [RFC 6750](source)

* Threat Model and Security Objectives (Informative)
Assume an honest-but-curious relay and untrusted networks. Objectives: confidentiality of ACP payloads end-to-end, integrity, peer authentication, replay resistance, and minimal relay state. [RFC 3552](source)

* Requirements
** Transport and Admission
- =RAT2E-REQ-001= (CT-RELAY): The relay MUST terminate TLS and perform a WebSocket upgrade that validates =Origin= for browsers and echoes =Sec-WebSocket-Protocol: acp.jsonrpc.v1=. [RFC 6455](source)
- =RAT2E-REQ-002= (CT-RELAY): Browser attaches MUST be admitted only with a short-TTL, single-use attach token scoped to audience =relay= and scope =ws:connect=. [RFC 6750](source)
- =RAT2E-REQ-003= (CT-WEB, CT-RAT): After admission, endpoints MUST execute a Noise XX handshake and then encrypt all application frames as binary ciphertext. [Noise Protocol](source)
- =RAT2E-REQ-004= (CT-WEB, CT-RAT): The Noise prologue MUST bind (=session_id, attach_token, context_tag=) to prevent replay and token stripping. [RFC 6750](source) [RFC 3552](source)
- =RAT2E-REQ-005= (CT-RELAY): The relay MUST disable permessage-deflate on authenticated tunnels. [RFC 6455](source)

** Pairing and Tokens
- =RAT2E-REQ-010= (CT-RAT): =POST /v1/pair/start= MUST return {user_code, device_code, relay_ws_url, expires_in, interval}. [RFC 8628](source)
- =RAT2E-REQ-011= (CT-WEB): =POST /v1/pair/complete= MUST return {session_id, attach_token, relay_ws_url} and invalidate the user_code. [RFC 8628](source)
- =RAT2E-REQ-012= (CT-RELAY): Attach tokens MUST have TTL ≤ 5 minutes, ≥ 128-bit entropy, and MUST be single-use with =jti= replay cache semantics. [RFC 6750](source)

** ACP Over Tunnel
- =RAT2E-REQ-020= (CT-WEB, CT-RAT): ACP JSON-RPC MUST be transported unchanged over the encrypted tunnel. [RFC 9110](source)
- =RAT2E-REQ-021= (CT-WEB): On reconnect, CT-WEB SHOULD call =initialize= then =session/load= to resume deterministically. [ACP Overview](source)

** Presence
- =RAT2E-REQ-030= (CT-RAT or CT-WEB): Endpoints MUST emit periodic encrypted presence beats; the relay MAY expose a snapshot via HTTP. [RFC 9110](source)

** File REST (Thin)
- =RAT2E-REQ-040= (CT-RELAY): Provide browser-facing endpoints: list, read, write, markdown list, markdown batch. Enforce request size limits and rate limits. [RFC 9110](source)
- =RAT2E-REQ-041= (CT-RAT): Enforce path allow-lists, deny =..= and symlink escapes, and perform actual filesystem IO. [RFC 9110](source)

** Operational Protections
- =RAT2E-REQ-050= (CT-RELAY): Implement bounded queues per peer and close on overflow with a clear close code. [RFC 6455](source)
- =RAT2E-REQ-051= (CT-RELAY): Expose metrics at =/metrics= and a basic health endpoint at =/health=. [RFC 9110](source)

* Protocol Details
** Pairing (HTTP)
Requests and responses:
- =POST /v1/pair/start= → ~{ user_code[A–Z0–9]{8}, device_code, relay_ws_url, expires_in, interval }~. [RFC 8628](source)
- =POST /v1/pair/complete= → ~{ session_id, attach_token, relay_ws_url }~. [RFC 8628](source)
Behavior:
- Single-use user_code. Rate limit and respond with ~slow_down~ if polling exceeds ~interval~. [RFC 8628](source)

** Attach (WebSocket)
Paths and headers:
- RAT2E: ~GET /v1/connect?device_code=...~ with ~Sec-WebSocket-Protocol: acp.jsonrpc.v1~. [RFC 6455](source)
- Browser: ~GET /v1/connect?session_id=...~ with ~Sec-WebSocket-Protocol: acp.jsonrpc.v1, stk.sha256=BASE64URL(token_sha256)~ and a valid ~Origin~. [RFC 6455](source)
Server behavior:
- Validate Origin (browser only). Validate subprotocol and token proof. Echo accepted subprotocols in 101. Disable compression. [RFC 6455](source)

** End-to-End Crypto (Noise XX)
- Pattern XX with static keys. Each side pins the peer’s static key from pairing. Prologue binds ~(session_id, attach_token, "rat2e-v1")~ (context tag MAY be extended with transport channel binding). [Noise Protocol](source) [RFC 6750](source)

** ACP Resume
- On reconnect, CT-WEB calls ~initialize~ → ~session/load { sessionId }~ and continues processing streamed ~session/update~. [ACP Overview](source)

* HTTP API (Normative)
** Relay HTTP/WS
- =POST /v1/pair/start=, =POST /v1/pair/complete=, =GET /v1/connect= (WS), =GET /v1/presence/snapshot=, =GET /health=, =GET /metrics=, =GET /version=. [RFC 9110](source) [RFC 6455](source)
** File REST (browser-facing)
- =GET /api/files?path=dir=
- =GET /api/file-content?path=path=
- =POST /api/save-file= ~{ path, content }~
- =GET /api/markdown-files?path=dir=
- =POST /api/markdown-content= ~{ files: string[] }~ [RFC 9110](source)

* Data Model (Normative)
- ~user_codes: Map<user_code, { device_code, exp_ts }>~
- ~devices: Map<device_code, { rat_pubkey_b64, caps[], exp_ts }>~
- ~sessions: Map<session_id, { device_code, attach_token_hash, created_ts, rat_tx?, web_tx? }>~
- ~presence: Map<agent_id, { last_seen, status, caps? }>~
All rows are TTL-swept. Token storage SHOULD be hashed. [RFC 6750](source)

* State Machines (Informative)
- Pairing: CREATED → CLAIMED → ATTACHED → IDLE → EXPIRED. [RFC 8628](source)
- Attach: CONNECTING → AUTHENTICATING (Noise) → TRANSPORT (ciphertext) → CLOSING (idle/backpressure/error). [RFC 6455](source)
- Resume: Browser reconnects and resumes with ~session/load~. [ACP Overview](source)

* Close Codes and Backoff (Normative)
- 1008 Policy violation: bad/missing Origin, bad/missing subprotocol, token proof mismatch. [RFC 6455](source)
- 1013 Try again later: backpressure close on bounded queue overflow. [RFC 6455](source)
- 1001 Going away: idle timeout. 1011 Internal error: server failure. [RFC 6455](source)
Client backoff guidance: exponential with jitter (base 250 ms, factor 2.0, cap 30 s). [RFC 9110](source)

* Performance Targets and Limits (Normative)
- p50 attach/resume ≤ 800 ms. First presence paint ≤ 1.2 s. OFFLINE flip ≤ 35 s. Soak: 5k IDLE + 500 ACTIVE without error. [RFC 9110](source)

* Observability (Normative)
Metrics:
- ~active_sessions~, ~ws_open~, ~presence_online~, ~bytes_rx_total~, ~bytes_tx_total~, ~backpressure_closes_total~, ~resume_latency_ms~, ~pairing_rate~. [RFC 9110](source)

* Security Considerations (Normative)
- Admission control with scoped attach tokens and single-use semantics mitigates resource abuse. [RFC 6750](source)
- Binding ~(session_id, attach_token)~ into Noise prologue prevents replay and downgrade. [RFC 3552](source)
- Strict Origin allow-list and subprotocol echo enforce browser security model. [RFC 6455](source)
- No ACP plaintext in relay memory or logs; redact tokens, keys, and user codes. [RFC 3552](source)

* Privacy Considerations (Informative)
Store only transient IDs and counters. Presence snapshot reveals ONLINE/OFFLINE state; scope access to a tenant and apply least privilege. [RFC 3552](source)

* Internationalization and Accessibility (Informative)
All server messages are UTF-8. User-visible UI text SHOULD be externalized for localization; CLI pairing codes are A–Z0–9 to minimize ambiguity. [W3C Manual of Style](source)

* Versioning and Change Management (Normative)
This document follows Semantic Versioning. Backward-incompatible wire changes increment MAJOR. Compatible additions increment MINOR. Editorial fixes increment PATCH. [Semantic Versioning 2.0.0](source)

* Implementation Guidance (Informative)
Preferred stack:
- Bikeshed or ReSpec for authoring the public spec. Use =w3c/spec-prod= for CI to build, validate, and publish. [Bikeshed](source) [ReSpec](source) [spec-prod](source)
- Consider HTTP/2 WebSocket bootstrapping in future to reduce TCP connections. [RFC 8441](source)
- Consider ICE/QUIC transport for optional P2P fast-path with TURN-like fallback. [RFC 8445](source) [RFC 9000](source)

* Test Plan (Normative)
- Admission gates: Origin allow-list, subprotocol echo, attach token proof. Expected close codes on failure. [RFC 6455](source) [RFC 6750](source)
- Noise XX: golden vectors for handshake; replay rejection via prologue binding. [RFC 3552](source)
- Resume: drop/reload browser and measure ~resume_latency_ms~. [RFC 9110](source)
- Presence TTL and hysteresis. OFFLINE within ≤ 35 s. [RFC 9110](source)
- File REST: REST→RPC mapping, path allow-lists, symlink denial, size limits, rate limits. [RFC 9110](source)
- Backpressure: forced overflow causes 1013 and increments metric. [RFC 6455](source)

* Requirements Traceability Matrix (RTM) (Normative)
| Req ID         | Target     | Verification                          | Status |
|----------------+------------+----------------------------------------+--------|
| RAT2E-REQ-001  | CT-RELAY   | WS upgrade test, Origin/subprotocol    | Must   |
| RAT2E-REQ-002  | CT-RELAY   | Token TTL/one-time unit tests          | Must   |
| RAT2E-REQ-003  | CT-WEB/RAT | Noise XX cipher-only frame inspection  | Must   |
| RAT2E-REQ-004  | CT-WEB/RAT | Prologue binding replay tests          | Must   |
| RAT2E-REQ-010  | CT-RAT     | Pair/start contract test               | Must   |
| RAT2E-REQ-011  | CT-WEB     | Pair/complete contract test            | Must   |
| RAT2E-REQ-021  | CT-WEB     | Resume determinism integration test    | Should |
| RAT2E-REQ-040  | CT-RELAY   | REST limits and rate-limit tests       | Must   |
| RAT2E-REQ-041  | CT-RAT     | Path allow-list and symlink denial     | Must   |
| RAT2E-REQ-050  | CT-RELAY   | Bounded queue overflow → 1013          | Must   |
| RAT2E-REQ-051  | CT-RELAY   | Metrics/health endpoints               | Must   | [RFC 9110](source)

* Change Log (Informative)
- 1.0.0: Initial publication aligned to professional spec style: status-of-this-doc, conformance targets, numbered requirements, RTM, security/privacy, performance targets, and informative guidance. [RFC 7322](source)

* Acknowledgments (Informative)
Thanks to contributors who iterated on authentication, relay admission, and presence semantics. [W3C Manual of Style](source)
