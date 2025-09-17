# RAT Web (Vite + SolidJS)

A minimal local web UI that connects to RAT's local WebSocket bridge using the `acp.jsonrpc.v1` subprotocol. This supports the local websockets mode (unencrypted) to exercise ACP JSON‑RPC easily from the browser.

## Prerequisites
- Node 18+ (or 20+)
- pnpm / npm / yarn
- RAT local WebSocket server running (see below)

## Start the local WebSocket in RAT

In the repository root:

```bash
cargo run -p rat -- --local-ws --local-port 8081
```

This starts a dev WebSocket at `ws://127.0.0.1:8081` that echoes the `Sec-WebSocket-Protocol: acp.jsonrpc.v1` subprotocol for browser correctness and can bridge to a local ACP agent if configured, or run in echo mode otherwise.

## Run the web UI

```bash
cd rat-web
pnpm i   # or: npm i / yarn
pnpm dev # or: npm run dev
```

Open http://localhost:5173 and:
- Click Connect (opens `ws://127.0.0.1:8081` with `acp.jsonrpc.v1`)
- Click “Start Session” (sends `session/new` with `cwd: "."` and `mcpServers: []`)
- When the session is created, a prompt input appears; type a prompt and click “Send Prompt” (sends `session/prompt`)

Notes:
- The UI minifies JSON when sending programmatic messages so each frame is a single line (some agents parse line-delimited JSON).

## Notes
- This is a local‑only, unencrypted development path. The hosted relay + Noise flow from `spec_done.md` is not implemented here.
- If you set `RAT2E_AGENT_CMD`/`RAT2E_AGENT_ARGS` in your environment before starting RAT, the local bridge will pipe WS frames directly to that ACP agent process.
- For subprotocol correctness: browsers require the server to echo the chosen protocol token; RAT’s local WS does this for `acp.jsonrpc.v1`.
