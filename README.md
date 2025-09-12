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

## Acknowledgments

- [Agent Client Protocol](https://agentclientprotocol.com/) by Zed Industries
- [ratatui](https://ratatui.rs/) for the excellent TUI framework
- [tachyonfx](https://github.com/junkdog/tachyonfx) for visual effects
- [Anthropic](https://www.anthropic.com/) for Claude Code
- [Google](https://ai.google.dev/) for Gemini CLI
