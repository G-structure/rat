#+BEGIN_SRC text
#+TITLE: RAT2E Remote Control via Hosted Relay — Software Design Specification
#+SUBTITLE: Version 1.0.4
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

* Terminology (Normative)
- attach_nonce: A 128-bit, single-use, time-limited, relay-issued random value distributed to both peers during pairing and bound into the Noise prologue. [RFC 4086](https://www.rfc-editor.org/info/rfc4086) [Noise Protocol Framework](https://noiseprotocol.org/noise.pdf)
- effective_subprotocol: The exact Sec-WebSocket-Protocol value that the relay will echo to the browser on successful upgrade. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

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
      NBR["Noise XX Responder (WebCrypto/WASM)"]
     ACPFE["ACP Client\n@zed-industries/agent-client-protocol"]
     UI["UI Elements\nChat, File Editor, Terminal, Permissions, Commands"]
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
- =RAT2E-REQ-004= (CT-WEB, CT-RAT): The Noise prologue MUST bind =LP("rat2e-v1"), LP(session_id), LP(stksha256), LP(attach_nonce), LP(effective_subprotocol)= exactly. Any mismatch MUST abort the handshake. [Noise Protocol Framework](https://noiseprotocol.org/noise.pdf)
- =RAT2E-REQ-005= (CT-RELAY): The relay MUST NOT negotiate permessage-deflate on authenticated tunnels. [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)
- =RAT2E-REQ-001C= (CT-RELAY): The server MUST disable permessage-deflate for /v1/connect and reject any client offer on authenticated tunnels. [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)

** Pairing and Tokens
- =RAT2E-REQ-010= (CT-RAT): =POST /v1/pair/start= MUST return {user_code, device_code, relay_ws_url, expires_in, interval}. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- =RAT2E-REQ-011 ("complete")= (CT-WEB): =POST /v1/pair/complete= MUST return {session_id, attach_token, relay_ws_url} and invalidate the user_code. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- =RAT2E-REQ-012= (CT-RELAY): Attach tokens MUST have TTL ≤ 5 minutes, ≥ 128-bit entropy, and be single-use with =jti= replay cache semantics; only hashed token material MAY be stored. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- =RAT2E-REQ-011A= (CT-WEB): For resume, CT-WEB MUST obtain a fresh attach ticket via POST /v1/session/attach-ticket before reconnecting. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- =RAT2E-REQ-012A= (CT-RELAY): Replay protection MUST be enforced cluster-wide using a distributed TTL store for token and nonce hashes; first-use marking MUST be atomic. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750) [RFC 4086](https://www.rfc-editor.org/info/rfc4086)

** ACP Over Tunnel
- =RAT2E-REQ-020= (CT-WEB, CT-RAT): ACP JSON-RPC MUST be transported unchanged over the encrypted tunnel. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =RAT2E-REQ-021= (CT-WEB): On reconnect, CT-WEB SHOULD call =initialize= then =session/load= to resume deterministically. [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

** Web UI ACP Implementation
- =RAT2E-REQ-022= (CT-WEB): The Browser Web UI MUST speak ACP as a client, implementing all ACP messages defined in the official "@zed-industries/agent-client-protocol" npm package (version 0.3.1-alpha.1 or later) for ACP protocol implementation. [ACP Protocol](https://agentclientprotocol.com/)
- =RAT2E-REQ-023= (CT-WEB): The Browser Web UI MUST provide UI elements for all ACP message types to enable interactive remote control functionality.

*** ACP Messages Supported by Web UI
The Browser Web UI MUST support the following ACP messages as defined in the official @zed-industries/agent-client-protocol npm package (version 0.3.1-alpha.1 or later):

**** Client-to-Agent Messages (Web UI Sends)
- =initialize=: Establish connection and negotiate capabilities. UI element: Connection status indicator.
- =authenticate=: Authenticate with agent if required. UI element: Authentication form/modal.
- =session/new=: Create new conversation session. UI element: "New Session" button and session creation dialog.
- =session/load=: Load existing session (if agent supports). UI element: Session selection dropdown/list.
- =session/prompt=: Send user prompts with content. UI element: Chat input field supporting text, images, and file attachments.
- =session/cancel=: Cancel ongoing operations. UI element: Cancel button in chat interface.
- =session/set_mode=: Change the agent's operating mode. UI element: Mode selection dropdown in session interface.

**** Agent-to-Client Messages (Web UI Receives)
- =session/update=: Real-time session updates. UI elements:
  - Message display area for text chunks
  - Progress indicators for tool calls (including switch_mode tool type)
  - Plan display for execution plans
  - File diff viewer for code changes
  - Terminal output display
  - Available commands list for slash commands
  - Current mode indicator and mode change notifications
- =session/request_permission=: Request user permission for tool calls. UI element: Permission dialog with allow/reject options.
- =fs/read_text_file=: Read file content. UI element: File content viewer/editor.
- =fs/write_text_file=: Write file content. UI element: File editor with save functionality.
- =terminal/create=: Create terminal session. UI element: Terminal emulator interface.
- =terminal/output=: Get terminal output. UI element: Terminal output display area.
- =terminal/wait_for_exit=: Wait for terminal command completion. UI element: Terminal status indicators.
- =terminal/kill=: Terminate terminal process. UI element: Terminal kill button.
- =terminal/release=: Release terminal resources. UI element: Terminal session management.

*** UI Element Requirements
- =RAT2E-REQ-024= (CT-WEB): The Browser Web UI MUST implement a chat-like interface for =session/prompt= and =session/update= messages, displaying user messages, agent responses, and real-time progress.
- =RAT2E-REQ-025= (CT-WEB): The Browser Web UI MUST provide file browsing and editing capabilities for =fs/read_text_file= and =fs/write_text_file= operations.
- =RAT2E-REQ-026= (CT-WEB): The Browser Web UI MUST include permission dialogs for =session/request_permission= with clear options for allowing or denying tool execution.
- =RAT2E-REQ-027= (CT-WEB): The Browser Web UI MUST support terminal emulation for terminal-related ACP messages, providing command input and output display.
- =RAT2E-REQ-028= (CT-WEB): The Browser Web UI MUST display execution plans from =session/update= messages with visual indicators for task status (pending, in_progress, completed).
- =RAT2E-REQ-029= (CT-WEB): The Browser Web UI MUST show tool call progress and results, including file diffs, terminal output, and other tool-generated content.
- =RAT2E-REQ-030= (CT-WEB): The Browser Web UI MUST display available commands from =session/update= notifications with =available_commands_update=, providing a command palette or slash command interface for users to invoke agent commands.
- =RAT2E-REQ-031= (CT-WEB): The Browser Web UI MUST support session modes, displaying available modes from =session/new= and =session/load= responses, allowing users to change modes via =session/set_mode=, and updating the UI when receiving =current_mode_update= notifications.
- =RAT2E-REQ-032= (CT-WEB): The Browser Web UI SHOULD support ACP extensibility features including the =_meta= field for custom data and extension methods starting with underscore (=_=) for custom functionality.

** Presence
- =RAT2E-REQ-033= (CT-WEB/RAT): Endpoints MUST emit encrypted presence beats. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =RAT2E-REQ-034= (CT-RELAY): GET /v1/presence/snapshot MUST require viewer auth and tenant scoping. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

** File REST (Thin)
- =RAT2E-REQ-040= (CT-RELAY): Provide browser-facing endpoints: list, read, write, markdown list, markdown batch. Enforce request size limits and rate limits. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =RAT2E-REQ-041= (CT-RAT): Enforce path allow-lists, deny =..= and symlink escapes, and perform filesystem IO. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

** Operational Protections
- =RAT2E-REQ-050= (CT-RELAY): Implement bounded queues per peer and close on overflow with 1013. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- =RAT2E-REQ-051= (CT-RELAY): Expose metrics at =/metrics= and a basic health endpoint at =/health=. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

** Browser Key Management
- =RAT2E-REQ-060= (CT-WEB): SHOULD use WebCrypto X25519 when available; otherwise WASM fallback; MUST zeroize; SHOULD deploy COOP/COEP + Trusted Types + no-referrer. [WICG Secure Curves](https://wicg.github.io/webcrypto-secure-curves/) [MDN CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey) [web.dev COOP/COEP](https://web.dev/articles/coop-coep) [MDN Trusted Types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types) [MDN Referrer-Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Referrer-Policy)
- =RAT2E-REQ-061= (CT-WEB): On reload, CT-WEB MUST reuse the same static key for session resume; if WASM fallback used, CT-WEB MUST zeroize JS/WASM buffers promptly and prevent SAB usage unless cross-origin isolated. [Noise Spec PDF](https://noiseprotocol.org/noise.pdf) [web.dev COOP/COEP](https://web.dev/articles/coop-coep)

* Protocol Details
** Pairing (HTTP)
Requests and responses:
- =POST /v1/pair/start= → ~{ user_code[A–Z0–9]{8}, device_code, relay_ws_url, expires_in, interval }~. Implement rate limit and =slow_down= responses if polling exceeds interval. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- =POST /v1/pair/poll { device_code }= → ~{ status: "pending"|"ready", session_id?, attach_nonce?, effective_subprotocol?, interval, expires_in }~. Rate-limit and return 429 { error: "slow_down" } if calls precede interval. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628) [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =POST /v1/pair/complete= → ~{ session_id, attach_token, attach_nonce, relay_ws_url, effective_subprotocol }~; invalidate =user_code=. Apply per-IP throttles and lockouts on repeated bad codes. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628) [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- =POST /v1/session/attach-ticket { session_id }= → ~{ attach_token, attach_nonce, effective_subprotocol }~. For reconnects/resume only; single-use, TTL ≤ 5m. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)

** Attach (WebSocket) — Flow (Normative)
- Browser request: GET /v1/connect?session_id=... with Sec-WebSocket-Protocol: effective_subprotocol; must include a valid Origin. Server MUST echo the exact token or close with 1008. MUST NOT negotiate permessage-deflate. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)
- RAT request: GET /v1/connect?device_code=... with Sec-WebSocket-Protocol: acp.jsonrpc.v1. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

Subprotocol ABNF:
subprotocol = "acp.jsonrpc.v1" "." "stksha256" "." base64url_nopad
base64url_nopad = 1*( ALPHA / DIGIT / "-" / "_" )
The "stksha256" component is BASE64URL_NOPAD(SHA-256(attach_token)); no "=" padding permitted. [RFC 4648 §5](https://datatracker.ietf.org/doc/html/rfc4648#section-5) [RFC 7515 Appx C](https://datatracker.ietf.org/doc/html/rfc7515#appendix-C)

** End-to-End Crypto (Noise XX)
Pattern XX with static keys. Each side pins the peer's static key from pairing. prologue = LP("rat2e-v1") || LP(session_id) || LP(stksha256) || LP(attach_nonce) || LP(effective_subprotocol) LP(x) = 2-byte big-endian length || bytes(x); stksha256 = BASE64URL_NOPAD(SHA-256(attach_token)). Both peers learn attach_nonce + effective_subprotocol during pairing; the raw attach_token stays in-browser. [Noise Protocol Framework](https://noiseprotocol.org/noise.pdf) [RFC 4648 §5](https://datatracker.ietf.org/doc/html/rfc4648#section-5)

** ACP Resume
On reconnect, CT-WEB calls ~initialize~ then ~session/load { sessionId }~ and continues processing streamed ~session/update~. [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

* HTTP API (Normative)
** Relay HTTP and WS
- =POST /v1/pair/start=, =POST /v1/pair/poll=, =POST /v1/pair/complete=, =POST /v1/session/attach-ticket=, =GET /v1/connect= (WS), =GET /v1/presence/snapshot=, =GET /health=, =GET /metrics=, =GET /version=. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
** File REST (browser-facing)
- =GET /api/files?path=dir=, =GET /api/file-content?path=path=, =POST /api/save-file= ~{ path, content }~, =GET /api/markdown-files?path=dir=, =POST /api/markdown-content= ~{ files: string[] }~. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Data Model (Normative)
- ~user_codes: Map<user_code, { device_code, exp_ts }>~
- ~devices: Map<device_code, { rat_pubkey_b64, caps[], exp_ts }>~
- ~sessions: Map<session_id, { device_code, attach_token_hash, attach_nonce_hash, effective_subprotocol, created_ts, used:boolean, exp_ts }>~
- ~presence: Map<agent_id, { last_seen, status, caps? }>~
- ~viewers: Map<viewer_token_id, { tenant_id, scopes[], exp_ts }>~ when presence auth is enabled. Rows are TTL-swept. Token storage SHOULD be hashed. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)

* State Machines (Informative)
- Pairing: CREATED → CLAIMED → ATTACHED → IDLE → EXPIRED. [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628)
- Attach: CONNECTING → AUTHENTICATING (Noise) → TRANSPORT (ciphertext) → CLOSING (idle or backpressure or error). [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Resume: Browser reconnects and resumes with ~session/load~. [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)

* Close Codes and Backoff (Normative)
- 1000 Normal Closure for graceful drain/user disconnect; clients SHOULD reconnect with backoff without requiring re-pair. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- 1008 Policy violation for bad or missing Origin, bad or missing subprotocol, or subprotocol mismatch. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- 1013 Try again later for bounded queue overflow with reason "bounded-queue-overflow". [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- 1001 Going away for idle timeout; 1011 Internal error for unexpected server failure. Clients back off exponentially with jitter: base 250 ms, factor 2.0, cap 30 s; reset after 60 s stable. Define queue size defaults (e.g., 64 KiB per peer), server PING every 20s, client PONG within 10s, idle timeouts 60s. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Performance Targets and Limits (Normative)
p50 attach or resume ≤ 800 ms; first presence paint ≤ 1.2 s; OFFLINE flip ≤ 35 s; soak target 5k IDLE + 500 ACTIVE without error. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Observability (Normative)
Metrics: ~active_sessions~ (gauge), ~ws_open~ (gauge), ~presence_online~ (gauge), ~bytes_rx_total~ (counter), ~bytes_tx_total~ (counter), ~backpressure_closes_total~ (counter), ~pairing_rate~ (counter), ~origin_rejects_total~ (counter), ~subprotocol_mismatch_total~ (counter), ~replay_detected_total~ (counter), ~attach_ticket_issued_total~ (counter), ~attach_ticket_used_total~ (counter), ~resume_latency_ms~ (histogram with p50/p95/p99). Health: =/health= 200; =/version= { version, commit, build_time }. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Security Considerations (Normative)
Admission control with scoped attach tokens and single-use semantics mitigates abuse. Attach artifacts (attach_token, attach_nonce) are sender-constrained by binding stksha256 + attach_nonce + effective_subprotocol into the Noise prologue; replay reuse is prevented by single-use checks at the relay. Strict Origin allow-list and subprotocol echo enforce the browser security model. The relay MUST not retain ACP plaintext. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750) [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

* Privacy Considerations (Informative)
Store only transient IDs and counters. Presence reveals ONLINE or OFFLINE status and MUST be tenant-scoped at the HTTP layer. [RFC 3552](https://datatracker.ietf.org/doc/html/rfc3552)

* Internationalization and Accessibility (Informative)
All server messages are UTF-8. User-visible UI text SHOULD be externalized for localization. CLI pairing codes use A–Z0–9 to minimize ambiguity. [W3C Manual of Style](https://www.w3.org/Style/)

* Versioning and Change Management (Normative)
Semantic Versioning applies. Backward-incompatible wire changes increment MAJOR. Compatible additions increment MINOR. Editorial fixes increment PATCH. [SemVer 2.0.0](https://semver.org/)

* Implementation Guidance (Informative)
Deployments MAY use RFC 8441 to bootstrap WS over HTTP/2. Intermediary support varies; implementations MUST support HTTP/1.1 fallback and SHOULD probe ALPN. Optional P2P fast-path MAY leverage ICE and QUIC with quotas. [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441.html) [RFC 8445](https://www.rfc-editor.org/rfc/rfc8445) [RFC 9000](https://www.rfc-editor.org/rfc/rfc9000)

* File REST Clarity (Informative)
In v1, file payloads are visible to the relay process because the browser calls HTTP on the relay which maps to File RPC over the tunnel. For strict end-to-end file privacy, migrate to streaming file RPC entirely inside the WS Noise transport in a post-v1 revision. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)

* Test Plan (Normative)
- Admission gates: Origin allow-list, single echoed subprotocol token equality, attach token proof inside subprotocol, expected 1008 on failure. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- Noise XX: golden vectors for handshake; replay rejection via prologue binding and peer static pinning; negative tests on mutated =effective_subprotocol= (e.g., change one character) and reused or altered =attach_nonce= (replay or flip bits). [Noise Protocol Framework](https://noiseprotocol.org/noise.pdf)
- Resume: drop or reload browser and measure ~resume_latency_ms~; validate deterministic ~session/load~ replay. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html) [ACP session/load](https://agentclientprotocol.com/protocol/session-setup)
- Presence: TTL and hysteresis; tenant-scoped snapshot; unauthorized viewer returns 401 or 403. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- File REST: REST→RPC mapping, allow-lists, symlink denial, size limits, rate limits. [RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)
- Backpressure: forced overflow causes 1013 and increments metric; client backoff behavior verified. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Browser key path: IndexedDB eviction simulation and re-pair UX; WebAuthn PRF wrapping path must not expose raw key on failure; CSP with Trusted Types enabled with no violations. [WebAuthn Level 3](https://www.w3.org/TR/webauthn-3/) [MDN CryptoKey](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey)
- ACP UI elements: All ACP message types must have corresponding UI elements; chat interface must handle all session/update variants including available_commands_update and current_mode_update; permission dialogs must present all options correctly; file operations must work with proper error handling; terminal interface must support all terminal operations; command palette must display available commands and handle slash command invocation; session modes must be displayed and changeable via UI controls. [ACP Protocol](https://agentclientprotocol.com/)
- Subprotocol echo: Offer ["acp.jsonrpc.v1.stksha256.X","bogus"]; server MUST echo exact token or close 1008. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455)
- Replay: Use same attach_nonce twice → first OK, second 1008; metric replay_detected_total++. [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750)
- Compression: Client offers permessage-deflate → server MUST decline; verify no "Sec-WebSocket-Extensions" in handshake. [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692)
- H2 bootstrap: behind an intermediary lacking RFC 8441 → graceful H1.1 fallback. [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441.html)
- Browser hardening: Trusted Types enabled, Referrer-Policy:no-referrer, COOP/COEP set; pages still function. [MDN Trusted Types](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types) [web.dev COOP/COEP](https://web.dev/articles/coop-coep) [MDN Referrer-Policy](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Content-Security-Policy/trusted-types)

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
| RAT2E-REQ-001C | CT-RELAY   | Compression disabled test on /v1/connect        | Must   |
| RAT2E-REQ-010  | CT-RAT     | Pair/start contract test                        | Must   |
| RAT2E-REQ-011  | CT-WEB     | Pair/complete contract test                     | Must   |
| RAT2E-REQ-011A | CT-WEB     | Reconnect requires /v1/session/attach-ticket    | Must   |
| RAT2E-REQ-012  | CT-RELAY   | Token entropy or TTL or jti cache               | Must   |
| RAT2E-REQ-012A | CT-Relay   | Distributed single-use + atomic first-use test  | Must   |
| RAT2E-REQ-020  | CT-WEB/RAT | ACP transparency test                           | Must   |
| RAT2E-REQ-021  | CT-WEB     | Resume determinism integration test             | Should |
| RAT2E-REQ-022  | CT-WEB     | ACP client implementation with all messages     | Must   |
| RAT2E-REQ-023  | CT-WEB     | UI elements for all ACP message types           | Must   |
| RAT2E-REQ-024  | CT-WEB     | Chat interface for session messages             | Must   |
| RAT2E-REQ-025  | CT-WEB     | File browsing and editing capabilities          | Must   |
| RAT2E-REQ-026  | CT-WEB     | Permission dialogs for tool calls               | Must   |
| RAT2E-REQ-027  | CT-WEB     | Terminal emulation interface                    | Must   |
| RAT2E-REQ-028  | CT-WEB     | Execution plan display with status indicators   | Must   |
| RAT2E-REQ-029  | CT-WEB     | Tool call progress and results display          | Must   |
| RAT2E-REQ-030  | CT-WEB     | Available commands display and slash commands   | Must   |
| RAT2E-REQ-031  | CT-WEB     | Session modes support and UI                    | Must   |
| RAT2E-REQ-032  | CT-WEB     | ACP extensibility features support              | Should |
| RAT2E-REQ-033  | CT-WEB/RAT | Presence beats update                           | Must   |
| RAT2E-REQ-034  | CT-RELAY   | Presence viewer auth and tenant scoping         | Must   |
| RAT2E-REQ-040  | CT-RELAY   | REST limits and rate-limit tests                | Must   |
| RAT2E-REQ-041  | CT-RAT     | Path allow-list and symlink denial              | Must   |
| RAT2E-REQ-050  | CT-RELAY   | Queue overflow → 1013                           | Must   |
| RAT2E-REQ-051  | CT-RELAY   | Metrics or health endpoints                     | Must   |
| RAT2E-REQ-060  | CT-WEB     | CryptoKey persistence or WebAuthn wrapping      | Must   |
| RAT2E-REQ-061  | CT-WEB     | Key reuse on reload or zeroization              | Must   |

[RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html)

* Change Log (Informative)
- 1.0.4: Added attach_nonce + effective_subprotocol; defined subprotocol ABNF; introduced /v1/pair/poll and /v1/session/attach-ticket; cluster-safe single-use semantics; revised key management (WebCrypto X25519 SHOULD; WASM fallback + zeroization + COOP/COEP, Trusted Types, no-referrer); added observability metrics and backpressure details; fixed REQ ID collisions (Presence now 033/034); clarified H2 bootstrap with H1.1 fallback. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 4648](https://datatracker.ietf.org/doc/html/rfc4648#section-5) [RFC 8628](https://datatracker.ietf.org/doc/html/rfc8628) [RFC 6750](https://datatracker.ietf.org/doc/html/rfc6750) [Noise Spec](https://noiseprotocol.org/noise.pdf) [RFC 8441](https://www.rfc-editor.org/rfc/rfc8441.html) [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692) [MDN](https://developer.mozilla.org/en-US/docs/Web/API/CryptoKey) [web.dev](https://web.dev/articles/coop-coep)
- 1.0.3: Updated ACP protocol support to include session modes, extensibility features, and switch_mode tool type; aligned with @zed-industries/agent-client-protocol v0.3.1-alpha.1 changes. [ACP Protocol](https://agentclientprotocol.com/)
- 1.0.2: Added comprehensive Web UI ACP implementation requirements; specified use of @zed-industries/agent-client-protocol TypeScript library (v0.3.1-alpha.1+); documented all ACP message types with corresponding UI elements; added available commands and session modes feature support; updated system diagram to reflect ACP client implementation. [ACP Protocol](https://agentclientprotocol.com/)
- 1.0.1: Corrected WS subprotocol proof grammar; added presence authZ; added browser key management requirements; clarified file REST visibility in v1; added testable additions and close code guidance; replaced placeholder citations with authoritative sources. [RFC 6455](https://datatracker.ietf.org/doc/html/rfc6455) [RFC 7692](https://datatracker.ietf.org/doc/html/rfc7692) [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) [RFC 8174](https://www.rfc-editor.org/info/bcp14) [RFC 7322](https://datatracker.ietf.org/doc/html/rfc7322)
#+END_SRC
