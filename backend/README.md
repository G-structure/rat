# RAT (Rust Agent Terminal)

A high-performance terminal-based ACP (Agent Client Protocol) client written in Rust, leveraging tachyonfx for stunning visual effects. RAT provides a unified interface for interacting with multiple AI coding agents (Claude Code and Gemini CLI) with advanced multi-agent control capabilities.

## Features

- **Multi-Agent Control**: Simultaneous control of multiple Claude Code and Gemini agent instances
- **Tabbed Interface**: Manage concurrent agent sessions with easy switching
- **Rich Visual Experience**: Animated code diffs and smooth UI transitions (Phase 4)
- **Advanced Edit Management**: Structured edit review with diff visualization
- **Terminal Integration**: Embedded terminal sessions for agent tool execution
- **Configuration System**: Comprehensive TOML-based configuration
- **Session Management**: Persistent sessions with message history

## Installation

### Prerequisites

- Rust 1.70+ 
- Node.js (for Claude Code ACP adapter)
- API keys for the agents you want to use:
  - `ANTHROPIC_API_KEY` for Claude Code
  - `GOOGLE_API_KEY` for Gemini

### Build from Source

```bash
git clone <repository-url>
cd rat
cargo build --release
```

### Install Claude Code ACP Adapter

```bash
npm install -g @zed-industries/claude-code-acp
```

### Install Gemini CLI

```bash
npm install -g gemini-cli
# or on macOS with Homebrew
brew install gemini-cli
```

## Usage

### Basic Usage

```bash
# Start RAT with default configuration
rat

# Start with a specific agent
rat --agent claude-code

# Point RAT at a custom ACP agent (e.g., the simulator)
rat \
  --agent-cmd cargo \
  --agent-arg run --agent-arg --quiet \
  --agent-arg --example --agent-arg sim_agent \
  --agent-arg -- \
  --agent-arg --scenario --agent-arg happy-path-edit \
  --agent-arg --speed --agent-arg fast

## Simulator Scenarios

The included `sim_agent` example provides several test scenarios to demonstrate different ACP protocol behaviors:

### Available Scenarios

- **`happy-path-edit`** (default): Simulates a successful file edit operation with plan creation, tool execution, and completion messages
- **`failure-path`**: Demonstrates a failed tool call (search operation) with error handling and user feedback
- **`images-and-thoughts`**: Shows agent thought processes and image content in responses
- **`commands-update`**: Illustrates command availability updates (requires `unstable` feature)

### Usage Examples

```bash
# Happy path edit scenario (default)
RUST_LOG=trace cargo run -p rat -- -vvv --agent-cmd cargo --agent-arg run --agent-arg --quiet --agent-arg --example --agent-arg sim_agent --agent-arg -- --agent-arg --scenario --agent-arg happy-path-edit --agent-arg --speed --agent-arg fast

# Failure path scenario
RUST_LOG=trace cargo run -p rat -- -vvv --agent-cmd cargo --agent-arg run --agent-arg --quiet --agent-arg --example --agent-arg sim_agent --agent-arg -- --agent-arg --scenario --agent-arg failure-path --agent-arg --speed --agent-arg fast

# Images and thoughts scenario
RUST_LOG=trace cargo run -p rat -- -vvv --agent-cmd cargo --agent-arg run --agent-arg --quiet --agent-arg --example --agent-arg sim_agent --agent-arg -- --agent-arg --scenario --agent-arg images-and-thoughts --agent-arg --speed --agent-arg fast

# Commands update scenario
RUST_LOG=trace cargo run -p rat -- -vvv --agent-cmd cargo --agent-arg run --agent-arg --quiet --agent-arg --example --agent-arg sim_agent --agent-arg -- --agent-arg --scenario --agent-arg commands-update --agent-arg --speed --agent-arg fast
```

### Speed Options

The simulator supports different speed multipliers:
- `slomo` (0.25x): Very slow for detailed observation
- `normal` (1.0x): Standard speed
- `fast` (2.0x): Accelerated for quick testing
- `max` (100.0x): Maximum speed for rapid iteration

# Use custom configuration file
rat --config ~/.config/rat/custom.toml

# Enable verbose logging
rat -vv
```

### Configuration

RAT uses TOML configuration files. The default configuration is created at `~/.config/rat/config.toml` on first run.

Example configuration:

```toml
[general]
log_level = "info"
auto_save_sessions = true
max_session_history = 1000

[agents]
default_agent = "claude-code"
auto_connect = ["claude-code"]
max_concurrent_agents = 5

[agents.claude_code]
enabled = true
auto_install = true
api_key_env = "ANTHROPIC_API_KEY"
model = "claude-3-5-sonnet-20241022"

[agents.gemini]
enabled = true
auto_install = true
api_key_env = "GOOGLE_API_KEY"
model = "gemini-2.0-flash-exp"

[ui]
[ui.theme]
name = "default"
syntax_highlighting = true

[ui.keybindings]
quit = "q"
new_session = "n"
switch_agent = "a"
next_tab = "Tab"
prev_tab = "BackTab"

[ui.effects]
enabled = true
animation_speed = 1.0
typewriter_delay_ms = 50
```

### Claude Code Tool Permissions

RAT starts Claude Code with file edit and tool usage enabled by default. It allows both ACP‑bridged FS tools and Claude's built‑in edit tools. You can override the tool configuration via environment variables:

- `RAT_PERMISSION_PROMPT_TOOL`: permission tool id to use (default: `mcp__acp__permission`).
- `RAT_ALLOWED_TOOLS`: comma‑separated list of allowed tools (default: `mcp__acp__read,mcp__acp__write,Read,Write,Edit,MultiEdit`).
- `RAT_DISALLOWED_TOOLS`: comma‑separated list of disallowed tools. Leave unset/empty to omit.

These apply to both the TUI‑launched agent and the `--local-ws` bridge.

### Key Bindings

- `q` - Quit application
- `n` - Create new session with default agent
- `a` - Switch agent
- `Tab` / `Shift+Tab` - Navigate between tabs
- `?` - Show help
- `Enter` - Start typing message / Send message
- `Esc` - Cancel input / Close dialogs
- `y` / `n` - Accept / Reject edit proposals
- `Ctrl+C` - Force quit

## Development

### Project Structure

```
rat/
├── src/
│   ├── main.rs              # Application entry point
│   ├── app.rs               # Main application state
│   ├── acp/                 # ACP client implementation
│   ├── adapters/            # Agent adapters (Claude Code, Gemini)
│   ├── config/              # Configuration system
│   ├── ui/                  # TUI components
│   ├── effects/             # Visual effects (Phase 4)
│   └── utils/               # Utility modules
├── examples/                # Usage examples
├── tests/                   # Test suites
└── docs/                    # Documentation
```

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib

# Run with logging
RUST_LOG=debug cargo test
```

### Running Examples

```bash
# Basic client example
cargo run --example basic_client
```

## Architecture

RAT is built around the Agent Client Protocol (ACP) which standardizes communication between code editors and AI coding agents. The architecture consists of:

1. **ACP Client Core**: Handles the ACP protocol communication
2. **Agent Adapters**: Specific implementations for Claude Code and Gemini
3. **TUI Framework**: Terminal user interface built with ratatui
4. **Multi-Agent Manager**: Coordinates multiple concurrent agent connections
5. **Configuration System**: TOML-based configuration with validation
6. **Effects System**: Visual enhancements using tachyonfx (Phase 4)

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Roadmap

- [x] **Phase 1**: Core Infrastructure (Basic ACP client, TUI shell, configuration)
- [ ] **Phase 2**: Claude Code Integration
- [ ] **Phase 3**: Gemini Integration  
- [ ] **Phase 4**: Visual Enhancement (tachyonfx effects)
- [ ] **Phase 5**: Advanced Features (Multi-session management, plugin system)
- [ ] **Phase 6**: Polish & Documentation

## ACP over Local WebSocket (Dev Testing)

RAT can expose a local, plaintext WebSocket bridge for ACP testing without wscat.

- Start the local WS bridge: `RUST_LOG=trace cargo run -p rat -- --local-ws --local-port 8889`
- The server listens on `ws://localhost:8889` and echoes the subprotocol `acp.jsonrpc.v1` if requested.
- Ensure an ACP agent is available. RAT auto-resolves Claude Code; or set `RAT2E_AGENT_CMD`/`RAT2E_AGENT_ARGS`.

Option A: websocat
- Install: `brew install websocat`
- Connect: `websocat -t ws://localhost:8889`
- Paste JSON-RPC messages (one per line):
  1) initialize
  {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":false}}}
  2) newSession (adjust `cwd`)
  {"jsonrpc":"2.0","id":2,"method":"newSession","params":{"cwd":"/Users/luc/projects/vibes","mcpServers":[]}}
  3) prompt (replace SESSION_ID)
  {"jsonrpc":"2.0","id":3,"method":"prompt","params":{"sessionId":"SESSION_ID","prompt":[{"type":"text","text":"hi"}]}}

Option B: Node client (uses `ws`)
- Install: `npm i ws` (if needed)
- Run:
  `node -e 'const WebSocket=require("ws");const ws=new WebSocket("ws://localhost:8889",["acp.jsonrpc.v1"]);ws.on("open",()=>{ws.send(JSON.stringify({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{"fs":{"readTextFile":true,"writeTextFile":true},"terminal":false}}}));});ws.on("message",(m)=>{console.log("recv:",m.toString());try{const msg=JSON.parse(m.toString());if(msg.result&&msg.result.protocolVersion){ws.send(JSON.stringify({"jsonrpc":"2.0","id":2,"method":"newSession","params":{"cwd":"/Users/luc/projects/vibes","mcpServers":[]}}));}if(msg.result&&msg.result.sessionId){const sid=msg.result.sessionId;ws.send(JSON.stringify({"jsonrpc":"2.0","id":3,"method":"prompt","params":{"sessionId":sid,"prompt":[{"type":"text","text":"hi"}]}}));}}catch{}});'`

Browser flow (for Web UI)
- Connect to `ws://localhost:8889` with subprotocol `acp.jsonrpc.v1`.
- Send ACP JSON-RPC 2.0 messages as Text frames, one JSON object per frame.
- Call in order: `initialize` → `newSession` → `prompt` (content blocks: `{ type:"text", text:"..." }`).
- Handle responses by `id`, and notifications like `session/update`.
- If `newSession` returns `auth_required`, set `ANTHROPIC_API_KEY` before launching RAT.

Common pitfalls
- Missing JSON-RPC fields (`jsonrpc`, `id`, `method`, `params`).
- Using non-ACP shapes like `{ "type":"prompt", "content":"hi" }`.
- No credentials set → `auth_required` on `newSession`.

## Acknowledgments

- [Agent Client Protocol](https://agentclientprotocol.com/) by Zed Industries
- [ratatui](https://ratatui.rs/) for the excellent TUI framework
- [tachyonfx](https://github.com/junkdog/tachyonfx) for visual effects
- [Anthropic](https://www.anthropic.com/) for Claude Code
- [Google](https://ai.google.dev/) for Gemini CLI
