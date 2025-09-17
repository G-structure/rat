#+BEGIN_SRC text
#+TITLE: RAT2E Remote Control via Hosted Relay — Software Design Specification
#+SUBTITLE: Version 1.0.1
#+AUTHOR: RAT2E Working Group
#+DATE: 2025-09-16
#+OPTIONS: toc:3 num:t ^:nil
#+LANGUAGE: en
#+PROPERTY: header-args :results none :exports code
[BCP 14](https://www.rfc-editor.org/info/bcp14) [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)

* Status of This Document
This is a living engineering specification intended for internal and partner implementation. It uses normative key words per BCP 14 and distinguishes Normative vs Informative sections. It is not an IETF standard. [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) [RFC 8174](https://www.rfc-editor.org/info/bcp14) [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)

* Abstract
RAT2E enables secure remote control of a local agent from a hosted browser UI via a lightweight relay. Both the browser and RAT2E connect outbound over TLS on port 443, establish an end-to-end encrypted channel using Noise XX over a WebSocket tunnel, and carry ACP JSON-RPC messages for interactive UX. Admission is enforced at the relay with short-TTL single-use attach tokens. The relay is a blind forwarder of ciphertext and maintains RAM-based TTL state. Presence is exposed via a tenant-scoped read-only snapshot. Optional ICE or QUIC is deferred to future work. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750) [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628) [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Notational Conventions
The key words “MUST”, “MUST NOT”, “REQUIRED”, “SHALL”, “SHALL NOT”, “SHOULD”, “SHOULD NOT”, “RECOMMENDED”, “NOT RECOMMENDED”, “MAY”, and “OPTIONAL” are to be interpreted as described in BCP 14 when, and only when, they appear in all capitals. [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) [RFC 8174](https://www.rfc-editor.org/info/bcp14)

* Scope
This document specifies interfaces, behaviors, and conformance criteria for the Relay Service, the RAT2E client, and the Browser Web UI. It excludes P2P fast-paths and persistent server-side chat history in v1. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)

* Conformance
** Conformance Targets
- CT-RELAY: the hosted Relay Service. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
- CT-RAT: the RAT2E client. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
- CT-WEB: the Browser UI. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
** Conformance Levels
- “Compliant”: all MUST requirements satisfied. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
- “Conditionally Compliant”: all MUST except those gated by an explicitly disabled optional feature. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
** Requirements Identification
Normative requirements are labeled =RAT2E-REQ-###=. The RTM maps requirements to verification. [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)

* System Overview (Informative)
Topology: Browser UI ↔ Relay (WSS 443) ↔ RAT2E. The relay performs WebSocket upgrade checks, admits connections with an attach token, then blindly forwards binary frames. All application bytes are encrypted end-to-end by a Noise XX transport. ACP JSON-RPC runs unmodified on this tunnel. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [Noise Spec PDF](https://noiseprotocol.org/noise.pdf)

#+END_SRC
#+BEGIN_mermaid
flowchart LR
  subgraph Browser["Browser Web UI"]
    BWS["WSS /v1/connect\nSec-WebSocket-Protocol: acp.jsonrpc.v1.stksha256.<b64u>"]
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
#+BEGIN_SRC text
[RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Roles and Responsibilities (Informative)
- CT-WEB: manage WS connect, perform Noise XX as responder, implement ACP client and reconnection. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- CT-RAT: initiate Noise XX, maintain anchor connection, bridge ACP to local agent, emit presence. [Noise Spec PDF](https://noiseprotocol.org/noise.pdf)
- CT-RELAY: validate Origin and subprotocol, admit connections with tokens, forward ciphertext, manage RAM TTL state and metrics. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)

* Threat Model and Security Objectives (Informative)
Assume an honest-but-curious relay and untrusted networks. Objectives: confidentiality of ACP payloads end-to-end, integrity, peer authentication, replay resistance, minimal relay state, and prevention of CSWSH. [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552) [PortSwigger CSWSH](https://portswigger.net/web-security/websockets/cross-site-websocket-hijacking)

* Requirements
** Transport and Admission
- =RAT2E-REQ-001= (CT-RELAY): The relay MUST terminate TLS and perform a WebSocket upgrade that validates =Origin= for browsers and selects exactly one =Sec-WebSocket-Protocol= value. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- =RAT2E-REQ-001A= (CT-RELAY): The server MUST echo exactly one subprotocol token that equals one of the offered tokens byte-for-byte or close with 1008. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- =RAT2E-REQ-001B= (CT-RELAY): The server MUST reject missing or allow-list-mismatched =Origin= on browser handshakes to mitigate CSWSH. [PortSwigger CSWSH](https://portswigger.net/web-security/websockets/cross-site-websocket-hijacking)
- =RAT2E-REQ-002 ("Attach tokens")= (CT-RELAY): Browser attaches MUST be admitted only with a short-TTL, single-use attach token scoped to audience =relay= and scope =ws:connect=. The token MUST NOT appear in the URL or cookies. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- =RAT2E-REQ-003= (CT-WEB, CT-RAT): After admission, endpoints MUST execute a Noise XX handshake and then encrypt all application frames as WS Binary ciphertext. [Noise Spec PDF](https://noiseprotocol.org/noise.pdf)
- =RAT2E-REQ-004= (CT-WEB, CT-RAT): The Noise prologue MUST bind =(session_id, attach_token, context_tag)= to prevent replay and token stripping. Context tag SHOULD hash the WS upgrade context. [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552)
- =RAT2E-REQ-005= (CT-RELAY): The relay MUST NOT negotiate permessage-deflate on authenticated tunnels. [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)

** Pairing and Tokens
- =RAT2E-REQ-010= (CT-RAT): =POST /v1/pair/start= MUST return {user_code, device_code, relay_ws_url, expires_in, interval}. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- =RAT2E-REQ-011 ("complete")= (CT-WEB): =POST /v1/pair/complete= MUST return {session_id, attach_token, relay_ws_url} and invalidate the user_code. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- =RAT2E-REQ-012= (CT-RELAY): Attach tokens MUST have TTL ≤ 5 minutes, ≥ 128-bit entropy, and be single-use with =jti= replay cache semantics; only hashed token material MAY be stored. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)

** ACP Over Tunnel
- =RAT2E-REQ-020= (CT-WEB, CT-RAT): ACP JSON-RPC MUST be transported unchanged over the encrypted tunnel. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =RAT2E-REQ-021= (CT-WEB): On reconnect, CT-WEB SHOULD call =initialize= then =session/load= to resume deterministically. [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

** Presence
- =RAT2E-REQ-030= (CT-RAT or CT-WEB): Endpoints MUST emit periodic encrypted presence beats; the relay MAY expose a snapshot via HTTP. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =RAT2E-REQ-031= (CT-RELAY): =GET /v1/presence/snapshot= MUST require a viewer auth token, be tenant-scoped, and return only rows the caller is authorized to see. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

** File REST (Thin)
- =RAT2E-REQ-040= (CT-RELAY): Provide browser-facing endpoints: list, read, write, markdown list, markdown batch. Enforce request size limits and rate limits. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =RAT2E-REQ-041= (CT-RAT): Enforce path allow-lists, deny =..= and symlink escapes, and perform filesystem IO. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

** Operational Protections
- =RAT2E-REQ-050= (CT-RELAY): Implement bounded queues per peer and close on overflow with 1013. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- =RAT2E-REQ-051= (CT-RELAY): Expose metrics at =/metrics= and a basic health endpoint at =/health=. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

** Browser Key Management
- =RAT2E-REQ-060= (CT-WEB): The browser MUST generate and persist a non-extractable static Noise private key as a =CryptoKey= and store it in IndexedDB. If X25519 via WebCrypto is unavailable, CT-WEB MUST wrap any raw private key using a key derived from a WebAuthn PRF and store only wrapped bytes. [WICG Secure Curves](https://wicg.github.io/webcrypto-secure-curves/) [MDN CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey) [WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/) [Yubico PRF explainer](https://developers.yubico.com/WebAuthn/Concepts/PRF_Extension/)
- =RAT2E-REQ-061= (CT-WEB): On reload, CT-WEB MUST reuse the same static key for session resume and MUST zeroize any unwrapped keying material after completing the handshake. [Noise Spec PDF](https://noiseprotocol.org/noise.pdf)
- =RAT2E-REQ-062= (CT-WEB): Deploy CSP with Trusted Types and =Referrer-Policy: no-referrer= to reduce exfil risk of IndexedDB-resident material. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Protocol Details
** Pairing (HTTP)
Requests and responses:
- =POST /v1/pair/start= → ~{ user_code[A–Z0–9]{8}, device_code, relay_ws_url, expires_in, interval }~. Implement rate limit and =slow_down= responses if polling exceeds interval. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- =POST /v1/pair/complete= → ~{ session_id, attach_token, relay_ws_url }~; invalidate =user_code=. Apply per-IP throttles and lockouts on repeated bad codes. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)

** Attach (WebSocket) — RFC-Corrected Flow
Client and server MUST follow the subprotocol grammar in RFC 6455 which accepts only HTTP “token” values without parameters. The proof MUST be embedded inside a single token. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- RAT2E request: ~GET /v1/connect?device_code=...~ with ~Sec-WebSocket-Protocol: acp.jsonrpc.v1~. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Browser request: ~GET /v1/connect?session_id=...~ with ~Sec-WebSocket-Protocol: acp.jsonrpc.v1.stksha256.<BASE64URL(SHA256(attach_token))>~ and a valid ~Origin~. The token MUST NOT appear in the URL. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- Server behavior: validate Origin for browsers, select and echo exactly one subprotocol token that equals the offered token byte-for-byte or close with 1008. MUST disable permessage-deflate. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)

** End-to-End Crypto (Noise XX)
Pattern XX with static keys. Each side pins the peer’s static key from pairing. Prologue binds ~(session_id, attach_token, "rat2e-v1", transport_ctx)~ where ~transport_ctx=SHA256(method || path || origin || sec-websocket-key || connection-id)~. Abort on mismatch. [Noise Spec PDF](https://noiseprotocol.org/noise.pdf) [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552)

** ACP Resume
On reconnect, CT-WEB calls ~initialize~ then ~session/load { sessionId }~ and continues processing streamed ~session/update~. [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

* HTTP API (Normative)
** Relay HTTP and WS
- =POST /v1/pair/start=, =POST /v1/pair/complete=, =GET /v1/connect= (WS), =GET /v1/presence/snapshot=, =GET /health=, =GET /metrics=, =GET /version=. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
** File REST (browser-facing)
- =GET /api/files?path=dir=, =GET /api/file-content?path=path=, =POST /api/save-file= ~{ path, content }~, =GET /api/markdown-files?path=dir=, =POST /api/markdown-content= ~{ files: string[] }~. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Data Model (Normative)
- ~user_codes: Map<user_code, { device_code, exp_ts }>~
- ~devices: Map<device_code, { rat_pubkey_b64, caps[], exp_ts }>~
- ~sessions: Map<session_id, { device_code, attach_token_hash, created_ts, rat_tx?, web_tx? }>~
- ~presence: Map<agent_id, { last_seen, status, caps? }>~
- ~viewers: Map<viewer_token_id, { tenant_id, scopes[], exp_ts }>~ when presence auth is enabled. Rows are TTL-swept. Token storage SHOULD be hashed. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)

* State Machines (Informative)
- Pairing: CREATED → CLAIMED → ATTACHED → IDLE → EXPIRED. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- Attach: CONNECTING → AUTHENTICATING (Noise) → TRANSPORT (ciphertext) → CLOSING (idle or backpressure or error). [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Resume: Browser reconnects and resumes with ~session/load~. [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

* Close Codes and Backoff (Normative)
- 1008 Policy violation for bad or missing Origin, bad or missing subprotocol, or subprotocol mismatch. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- 1013 Try again later for bounded queue overflow. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- 1001 Going away for idle timeout; 1011 Internal error for unexpected server failure. Clients back off exponentially with jitter: base 250 ms, factor 2.0, cap 30 s; reset after 60 s stable. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Performance Targets and Limits (Normative)
p50 attach or resume ≤ 800 ms; first presence paint ≤ 1.2 s; OFFLINE flip ≤ 35 s; soak target 5k IDLE + 500 ACTIVE without error. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Observability (Normative)
Metrics: ~active_sessions~, ~ws_open~, ~presence_online~, ~bytes_rx_total~, ~bytes_tx_total~, ~backpressure_closes_total~, ~resume_latency_ms~, ~pairing_rate~. Health: =/health= 200; =/version=. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Security Considerations (Normative)
Admission control with scoped attach tokens and single-use semantics mitigates abuse. Binding ~(session_id, attach_token, transport_ctx)~ into the Noise prologue prevents replay or downgrade. Strict Origin allow-list and subprotocol echo enforce the browser security model. The relay MUST not retain ACP plaintext. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750) [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

* Privacy Considerations (Informative)
Store only transient IDs and counters. Presence reveals ONLINE or OFFLINE status and MUST be tenant-scoped at the HTTP layer. [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552)

* Internationalization and Accessibility (Informative)
All server messages are UTF-8. User-visible UI text SHOULD be externalized for localization. CLI pairing codes use A–Z0–9 to minimize ambiguity. [W3C Manual of Style](https://www.w3.org/Style/)

* Versioning and Change Management (Normative)
Semantic Versioning applies. Backward-incompatible wire changes increment MAJOR. Compatible additions increment MINOR. Editorial fixes increment PATCH. [SemVer 2.0.0](https://semver.org/)

* Implementation Guidance (Informative)
For future deployments, HTTP/2 WebSocket bootstrapping MAY be used to reuse the TLS connection. Optional P2P fast-path MAY leverage ICE and QUIC with quotas. [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441.html) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

* File REST Clarity (Informative)
In v1, file payloads are visible to the relay process because the browser calls HTTP on the relay which maps to File RPC over the tunnel. For strict end-to-end file privacy, migrate to streaming file RPC entirely inside the WS Noise transport in a post-v1 revision. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

* Test Plan (Normative)
- Admission gates: Origin allow-list, single echoed subprotocol token equality, attach token proof inside subprotocol, expected 1008 on failure. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- Noise XX: golden vectors for handshake; replay rejection via prologue binding and peer static pinning; negative tests on altered transport_ctx. [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552) [Noise Spec PDF](https://noiseprotocol.org/noise.pdf)
- Resume: drop or reload browser and measure ~resume_latency_ms~; validate deterministic ~session/load~ replay. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)
- Presence: TTL and hysteresis; tenant-scoped snapshot; unauthorized viewer returns 401 or 403. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- File REST: REST→RPC mapping, allow-lists, symlink denial, size limits, rate limits. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- Backpressure: forced overflow causes 1013 and increments metric; client backoff behavior verified. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Browser key path: IndexedDB eviction simulation and re-pair UX; WebAuthn PRF wrapping path must not expose raw key on failure; CSP with Trusted Types enabled with no violations. [WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/) [MDN CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey)

* Requirements Traceability Matrix (RTM) (Normative)
| Req ID         | Target     | Verification                                    | Status |
|----------------+------------+--------------------------------------------------+--------|
| RAT2E-REQ-001  | CT-RELAY   | WS upgrade test, Origin and subprotocol         | Must   |
| RAT2E-REQ-001A | CT-RELAY   | Echo single subprotocol token or 1008           | Must   |
| RAT2E-REQ-001B | CT-RELAY   | Origin rejection for CSWSH                      | Must   |
| RAT2E-REQ-002  | CT-RELAY   | Token TTL or single-use or audience or no URL   | Must   |
| RAT2E-REQ-003  | CT-WEB/RAT | Cipher-only frame inspection post-Noise         | Must   |
| RAT2E-REQ-004  | CT-WEB/RAT | Prologue binding or replay tests                | Must   |
| RAT2E-REQ-005  | CT-RELAY   | No permessage-deflate on authenticated sockets  | Must   |
| RAT2E-REQ-010  | CT-RAT     | Pair/start contract test                        | Must   |
| RAT2E-REQ-011  | CT-WEB     | Pair/complete contract test                     | Must   |
| RAT2E-REQ-012  | CT-RELAY   | Token entropy or TTL or jti cache               | Must   |
| RAT2E-REQ-020  | CT-WEB/RAT | ACP transparency test                           | Must   |
| RAT2E-REQ-021  | CT-WEB     | Resume determinism integration test             | Should |
| RAT2E-REQ-030  | CT-WEB/RAT | Presence beats update                           | Must   |
| RAT2E-REQ-031  | CT-RELAY   | Presence viewer auth and tenant scoping         | Must   |
| RAT2E-REQ-040  | CT-RELAY   | REST limits and rate-limit tests                | Must   |
| RAT2E-REQ-041  | CT-RAT     | Path allow-list and symlink denial              | Must   |
| RAT2E-REQ-050  | CT-RELAY   | Queue overflow → 1013                           | Must   |
| RAT2E-REQ-051  | CT-RELAY   | Metrics or health endpoints                     | Must   |
| RAT2E-REQ-060  | CT-WEB     | CryptoKey persistence or WebAuthn wrapping      | Must   |
| RAT2E-REQ-061  | CT-WEB     | Key reuse on reload or zeroization              | Must   |
| RAT2E-REQ-062  | CT-WEB     | CSP or Trusted Types or no-referrer headers     | Must   |
[RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Change Log (Informative)
- 1.0.1: Corrected WS subprotocol proof grammar; added presence authZ; added browser key management requirements; clarified file REST visibility in v1; added testable additions and close code guidance; replaced placeholder citations with authoritative sources. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692) [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) [RFC 8174](https://www.rfc-editor.org/info/bcp14) [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
#+END_SRC
