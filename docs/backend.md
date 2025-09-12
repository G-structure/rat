# Backend Architecture — ACP Cache + Orchestrator Integration with Push/WebSocket Routing

This document specifies the backend for RAT’s ecosystem, focusing on:
- Preserving ACP end-to-end while adding durability and replay
- Integrating local orchestrators (Claude Code ACP, Gemini CLI ACP)
- Serving mobile and desktop clients over WebSocket with resume
- Using push notifications to wake mobile apps and nudge users
- Securing and operating the stack in production

Audience: backend engineers building and operating the ACP Cache Server and its edge components.

--------------------------------------------------------------------------------

## Design goals and invariants

- Protocol invariants
  - All outer edges speak pure ACP messages (JSON-RPC 2.0 framing) over a duplex transport (WebSocket or stdio).
  - The cache stores envelopes that wrap ACP frames for durability and replay; the ACP body is never mutated.
  - The replay control API is namespaced under acp.cache.* and orthogonal to ACP.

- Delivery semantics
  - Ordered, append-only per thread with strictly increasing sequence numbers.
  - At-least-once delivery to clients; clients de-duplicate using seq and msg_id.

- Mobile-first constraints
  - Foreground: one WSS connection; subscribe-from-offset and live tail.
  - Background: sockets will be suspended; rely on push notifications to alert users and opportunistically fetch minimal deltas.

- Security posture
  - TLS everywhere (WSS), OIDC for user identity, optional mTLS or SSH tunnel between on-box orchestrator and cache.
  - Short-lived access tokens; refresh tokens live on device (Keychain for iOS).

--------------------------------------------------------------------------------

## High-level topology

- Orchestrator (developer machine or server)
  - Supervises Claude Code ACP and Gemini CLI ACP processes.
  - Exposes a single ACP channel upstream.
  - Connects outbound to the ACP Cache Server over WSS (or over SSH forward to TCP).

- ACP Cache Server (backend core)
  - Ingests ACP frames from orchestrators and clients.
  - Persists envelopes and provides replay-then-tail via acp.cache.subscribe, acp.cache.fetch, acp.cache.ack.
  - Fans out live frames to connected subscribers at thread head.
  - Emits notifications to the Push Gateway when a thread’s head advances and eligible devices are not foreground-connected.

- Edge and identity
  - WebSocket reverse proxy (e.g., Traefik or an equivalent) terminates TLS and forwards upgrades to the cache.
  - Forward auth layer (e.g., oauth2-proxy) validates OIDC and injects identity to the cache.
  - JWKS validation can also be performed by the cache itself for defense in depth.

- Push Gateway
  - Provides APNs integration.
  - Stores device tokens and subscription preferences.
  - Generates signed APNs requests using a provider token (p8 key).

- State
  - Default single-node: SQLite with WAL for events and consumer offsets (development or small-scale).
  - Production: Postgres for events, offsets, and device registry; S3 for snapshots or large artifacts.

--------------------------------------------------------------------------------

## Data model and storage

- Envelope (storage-only wrapper; ACP body is unchanged)
  - thread_id: string
  - session_id: string
  - seq: integer (monotonic per thread_id)
  - ts: RFC 3339 timestamp
  - direction: agent_to_client or client_to_agent
  - kind: request or result or notification (JSON-RPC kinds)
  - jsonrpc: 2.0
  - body: the original ACP JSON object (request, result, or notification)
  - msg_id: UUID for idempotency/de-duplication
  - checksum: blake3(body) or equivalent

- Tables (starter schema)
  - events(thread_id, session_id, seq, ts, direction, kind, msg_id unique, body json/text, checksum blob)
  - consumers(consumer_id, thread_id, last_seq, updated_at)
  - snapshots(thread_id, seq, state json/text, created_at)
  - devices(user_id, platform, device_token, last_seen, push_enabled, metadata)
  - subscriptions(user_id, thread_id, preference, updated_at)

- Indexing
  - Primary key on (thread_id, seq).
  - Index on (thread_id, msg_id) for de-duplication checks.
  - Index on (user_id, thread_id) in subscriptions.
  - Covering index for sequential fetch by seq range.

- Durability
  - SQLite WAL for dev: simple, safe for a single writer and multiple readers.
  - Postgres for prod: higher write throughput and better concurrency; WAL archiving for backups.
  - Snapshots cap replay time for large threads; cadence is tuned to keep resume p50 under 200 ms at target sizes.

--------------------------------------------------------------------------------

## Control API (namespaced, ACP-preserving)

All ACP bodies pass through unmodified. Only the following methods are added to support replay and offsets:

- acp.cache.subscribe
  - params: thread_id (string), from_seq (integer), live (boolean), optional filters
  - behavior: streams envelopes from from_seq through head; if live is true, switches to live push after head.

- acp.cache.fetch
  - params: thread_id (string), from_seq (integer), limit (integer)
  - behavior: returns a finite page of envelopes for offline paging or background prefetch.

- acp.cache.ack
  - params: thread_id (string), seq (integer)
  - behavior: records last delivered offset for the consumer; used for resuming and backpressure management.

- Authentication and authorization
  - Requests are authenticated using OIDC access tokens in the Authorization header.
  - The cache enforces scopes such as thread:read, thread:write, terminal:attach, tool:run mapped to requested operations.
  - Tenancy is bound to the user/tenant in the token; thread_id namespaces must be validated to prevent cross-tenant access.

--------------------------------------------------------------------------------

## Orchestrator ↔ Cache integration

- Transport
  - Orchestrator opens a single long-lived ACP connection to the cache over WSS (or an SSH-tunneled TCP).
  - The orchestrator behaves as an ACP client to the underlying agents and as an ACP server to the cache.

- Routing
  - The orchestrator routes session methods (session/prompt, session/load) to an upstream agent bound to the thread_id.
  - Optional control methods for orchestrator introspection can be namespaced acp.orch.* (e.g., list_agents, route).

- Persistence behavior
  - The cache observes duplex ACP traffic and appends envelopes (direction tagged).
  - Session history for a thread is fully reconstructible by replaying envelopes for that thread.

- Failure and reconnect
  - If the orchestrator disconnects, the cache continues serving history to clients.
  - On reconnection, live frames resume and fan-out to connected subscribers.

--------------------------------------------------------------------------------

## WebSocket gateway and routing

- Single endpoint for WSS upgrades (e.g., wss://host/acp)
  - Reverse proxy terminates TLS and forwards upgraded connections to the cache.
  - Sticky routing is not required because the cache is authoritative for offsets; however, for future horizontal scale, use consistent hashing on (tenant, thread_id).

- Multiplexing
  - One socket carries both ACP pass-through frames and acp.cache.* control methods.
  - Heartbeats via ping/pong or lightweight JSON-RPC notifications.

- Reconnect policy (clients)
  - Exponential backoff with jitter.
  - On success, clients immediately subscribe from last_seq + 1 and switch to live.

--------------------------------------------------------------------------------

## Push notifications and mobile wake-ups

- Why push
  - iOS suspends background sockets; APNs is the only reliable wake-up channel for backgrounded apps.
  - Pushes are best-effort signals to fetch minimal deltas or prompt users to open the app.

- Device registry
  - Clients register device tokens with the backend, bound to user identity and optional metadata (app version, locale).
  - Store last_seen to expire stale tokens; handle feedback for invalid tokens.

- Triggering pushes
  - On new head for a thread where the subscriber is not actively connected (or has unacked backlog), enqueue a push to the user’s devices according to preferences.
  - Coalesce pushes (cooldown) to avoid noise under bursts.
  - Two kinds
    - Alert pushes: user-visible; tap opens the app directly into the thread.
    - Silent pushes: content-available = 1; intended for limited background fetch to update unread counts or prefetch a small page. Delivery is not guaranteed.

- APNs integration (provider token)
  - Use an APNs provider token (p8 file) with Key ID and Team ID; sign short-lived JWTs to authorize APNs requests over HTTP/2.
  - Keep signing keys off the application container or load at boot from a secret store.
  - Never include sensitive content in push payloads; only thread_id, titles, or counts if required by UX.

- Client behavior on push
  - Silent push: if allowed a short background window, call acp.cache.fetch for small updates and update badge counts; do not attempt long-lived sockets.
  - Alert push: when the user opens the app, the client connects WSS and acp.cache.subscribe from last_seq + 1 to render full context.

--------------------------------------------------------------------------------

## Authentication and authorization

- Identity
  - OIDC with PKCE for mobile and desktop clients; orchestrators may use service credentials or SSH+mTLS.
  - Access tokens are short-lived; refresh using standard flows. On iOS, refresh tokens are stored in Keychain (optionally biometrics-gated).

- Scopes and enforcement
  - thread:read to subscribe or fetch envelopes for threads the user may access.
  - thread:write to emit client_to_agent ACP frames (e.g., prompt).
  - terminal:attach and tool:run are enforced for ACP features that warrant stronger checks.
  - The cache enforces audience, issuer, validity period, signature algorithms, and scope checks.

- Network security
  - TLS enforced (ATS on iOS).
  - Certificate pinning recommended for first-party mobile deployments.
  - Optional mTLS channel between orchestrator and cache for added trust.

--------------------------------------------------------------------------------

## Ordering, delivery, and backpressure

- Append-only ordering
  - Each thread_id has a strictly increasing seq; server assigns seq at ingest.

- Delivery model
  - At-least-once with client-side de-dup.
  - Client maintains last_seq and msg_id set (rolling) to discard duplicates.

- Acknowledgements
  - Clients send acp.cache.ack periodically (every N envelopes or T seconds).
  - The server advances the consumer’s offset for monitoring and push suppression heuristics.

- Backpressure
  - Server uses a window size per client; pauses live push when unacked window exceeds threshold and resumes after acks.
  - Large backlog is replayed in pages; clients can request fetch for historical browsing.

--------------------------------------------------------------------------------

## Multi-tenant and routing

- Tenancy model
  - thread_id may include a tenant prefix; authorization binds token subject to a tenant keyspace.
  - Per-tenant databases or schemas can be used in Postgres for isolation.

- Orchestrator tagging
  - Orchestrators include tenant and thread metadata in the envelope’s thread_id; cache does not inspect ACP bodies.

- Routing futures
  - For horizontal scale, a routing layer can hash (tenant, thread_id) to shard the event log by partition, with read replicas for fan-out load.

--------------------------------------------------------------------------------

## Observability

- Metrics (server)
  - acp_events_ingest_total (counter)
  - acp_events_out_total (counter)
  - acp_replay_latency_ms (histogram)
  - acp_open_sockets (gauge)
  - acp_unacked_window (histogram)
  - auth_failures_total (counter)
  - push_enqueued_total, push_delivered_total, push_failed_total (counters)
  - db_write_latency_ms, db_read_latency_ms (histograms)

- Logs
  - Structured JSON with fields: tenant, user, thread_id, method, seq, latency_ms, bytes.
  - Redact secrets and tokens.

- Tracing
  - Ingest → persist → fan-out spans with context propagation from edge to storage.
  - Emit OTLP to a collector and backend (e.g., Tempo).

--------------------------------------------------------------------------------

## Deployment and operations (reference stack)

- Edge and identity
  - WebSocket reverse proxy that supports TLS and ACME certificates.
  - Forward auth layer integrating with your OIDC provider.
  - Optional SSH bastion/tunnel for orchestrator connections when direct egress is restricted.

- Core services
  - acp-cache server process (HTTP+WSS)
  - Postgres (prod) or SQLite (dev) for events and offsets
  - Push Gateway for APNs (can be a component of acp-cache or a separate service)
  - Observability suite: Prometheus, Grafana, logs, traces (stack is flexible)

- Secrets
  - APNs p8 key, OIDC client secrets in a secure store.
  - Database credentials, signing keys, and JWKS cache validation.

- Sizing guidance
  - Start single-node for MVP: acp-cache + Postgres on a single VM.
  - Observe events/sec, replay latency, and open sockets. Scale vertically first; add read replicas or partitions later.

--------------------------------------------------------------------------------

## End-to-end flows

- Orchestrator to mobile live tail
  1) Orchestrator connects to cache and initializes ACP passthrough.
  2) Developer prompts via RAT (or editor) → orchestrator → cache → stored as envelopes.
  3) iOS app opens WSS to cache; subscribes from last_seq + 1.
  4) Cache streams backlog to client, then switches to live; app acks periodically.
  5) Cache delivers live envelopes in order as orchestrator emits them.

- Mobile background event
  1) Orchestrator emits new envelopes; cache persists.
  2) Push Gateway checks subscriptions and device state; enqueues APNs (alert or silent).
  3) User taps alert or app gets a short background window.
  4) Foreground resume or background fetch pulls minimal deltas (fetch), then WSS subscribe on open.

- RAT TUI via backend to mobile (remote nudge)
  1) RAT TUI invokes an action targeting a thread tied to a mobile session (e.g., request review).
  2) Cache persists the intent as client_to_agent envelopes and updates thread head.
  3) Push Gateway sends alert push to the user’s device; opening the app resumes and shows the latest state via subscribe.

--------------------------------------------------------------------------------

## Security considerations

- Validate every JWT on the cache boundary; check audience, issuer, expiration, nbf, and signature algorithm.
- Keep APNs keys offline except within the Push Gateway; rotate regularly and implement token caching with periodic refresh.
- Avoid embedding secrets or private file paths in ACP payloads; if agents require secrets, scope them to the orchestrator’s environment.
- Enforce absolute file paths and scope them to allowable roots to mitigate path traversal.

--------------------------------------------------------------------------------

## SLOs and targets

- Resume-to-head P50 less than 200 ms for threads with up to 10k events when snapshotting is enabled.
- Added latency budget under 20 ms P50 for live pass-through relative to direct connection.
- Push alert delivery median under typical conditions: under 2 seconds in-region (best-effort).

--------------------------------------------------------------------------------

## Migration and scale-out path

- Phase 1 (MVP)
  - Single cache instance; SQLite WAL or Postgres; OIDC; APNs; subscribe/fetch/ack.
- Phase 2 (Resilience)
  - Postgres primary with streaming replicas; snapshot compaction worker; rate-limited push; backpressure tuning.
- Phase 3 (Scale)
  - Partitioned event log by (tenant, thread_id) hash; stateless cache instances; HA edge; multi-region read replicas; analytics (replay lag, error rates).

--------------------------------------------------------------------------------

## Minimal API surface (HTTP auxiliary)

These auxiliary HTTP endpoints complement the WSS ACP edge:

- POST /v1/devices/register
  - Body: platform, device_token, app_version, locale, push_enabled
  - Auth: user access token
  - Effect: stores or updates the device token for the user

- POST /v1/subscriptions/update
  - Body: thread_id, preference (e.g., all, mentions, none)
  - Auth: user access token
  - Effect: updates push preference per thread

- GET /v1/threads/heads
  - Query: optional filter
  - Auth: user access token
  - Effect: returns current known head seq per visible thread (for badge counts)

Note: all ACP traffic remains on the WSS endpoint; these HTTP endpoints manage device and notification metadata.

--------------------------------------------------------------------------------

## Open issues and risks

- APNs silent push reliability is inherently best-effort; UX must not depend on it for correctness.
- ACP terminal flows are unstable; gate terminal attach behind capability checks and feature flags.
- Multi-device conflict resolution for last_seq and unread counts requires consistent semantics across app instances.
- Cost pressure at high fan-out: use paging and live tail windowing to avoid overloading mobile devices.

--------------------------------------------------------------------------------

## What “done” looks like

- Orchestrators can connect and stream ACP unchanged; cache persists all frames with ordered seq.
- Mobile and desktop clients resume from last_seq quickly and switch to live tail seamlessly.
- Pushes nudge users effectively; opening the app shows an up-to-date session without gaps.
- Security is enforced at the edge and within the cache; observability provides visibility into ingest, replay, and push pipelines.
