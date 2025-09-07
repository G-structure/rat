# RAT (Rust Agent Terminal)
## High-Performance ACP Client for Claude Code & Gemini

### Executive Summary

RAT is a high-performance terminal-based ACP (Agent Client Protocol) client written in Rust, leveraging tachyonfx for stunning visual effects. The project creates a unified interface for interacting with multiple AI coding agents (Claude Code and Gemini CLI) through a standardized protocol, providing a superior alternative to traditional terminal interactions with rich visual feedback, structured edit reviews, and seamless agent switching.

### Project Architecture

#### Core Components

1. **ACP Client Core** (`src/acp/`)
   - **Client Implementation**: Rust-based ACP client using `agent-client-protocol` crate
   - **Session Management**: Multi-session support with concurrent agent connections
   - **Message Routing**: JSON-RPC 2.0 bidirectional communication handling
   - **Permission System**: Interactive permission prompts for file operations and tool calls

2. **Agent Adapters** (`src/adapters/`)
   - **Claude Code Adapter**: Integration with `@anthropic-ai/claude-code` SDK via subprocess
   - **Gemini Adapter**: Direct integration with Gemini CLI as ACP agent
   - **Unified Interface**: Common adapter trait for seamless agent switching
   - **Health Monitoring**: Agent availability and capability detection

3. **TUI Framework** (`src/ui/`)
   - **Main Interface**: Split-pane layout with agent selection, chat, and tool panels
   - **Chat View**: Message threading with syntax highlighting and code blocks
   - **Edit Review**: Diff viewer with hunk-level accept/reject using tachyonfx transitions
   - **Terminal Integration**: Embedded terminal sessions with streaming output
   - **Status Bar**: Real-time agent status, session info, and progress indicators

4. **Effects System** (`src/effects/`)
   - **Message Animations**: Typewriter effects for AI responses
   - **Code Highlighting**: Syntax-aware color transitions for code blocks
   - **Edit Transitions**: Smooth diff animations with fade/slide effects
   - **Status Indicators**: Pulsing connection status and activity indicators
   - **Theme System**: Dynamic color schemes with smooth transitions

5. **Configuration** (`src/config/`)
   - **Agent Settings**: API keys, model preferences, timeout configurations
   - **UI Preferences**: Themes, keybindings, layout preferences
   - **Project Context**: Per-project agent preferences and custom instructions

### Technical Specifications

#### Dependencies
```toml
[dependencies]
agent-client-protocol = "0.2.0-alpha.6"
tachyonfx = "0.18.0"
ratatui = "0.29.0"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
clap = { version = "4.5", features = ["derive"] }
anyhow = "1.0"
uuid = "1.11"
crossterm = "0.29"
```

#### Key Features

1. **Multi-Agent Support**
   - Seamless switching between Claude Code and Gemini agents
   - Session persistence across agent changes
   - Concurrent multi-agent conversations

2. **Rich Visual Experience**
   - Animated code diffs with tachyonfx effects
   - Smooth transitions between UI states
   - Syntax-highlighted code blocks with color animations
   - Real-time typing indicators and status updates

3. **Advanced Edit Management**
   - Structured edit review with diff visualization
   - Hunk-level accept/reject with animated feedback
   - Undo/redo support for edit operations
   - Batch edit operations with progress visualization

4. **Terminal Integration**
   - Embedded terminal sessions for agent tool execution
   - Streaming output with syntax highlighting
   - Interactive shell access with permission gating
   - Background process management

5. **Performance Optimizations**
   - Lazy loading of UI components
   - Efficient buffer management for large files
   - Streaming message processing
   - Minimal memory footprint

### Implementation Phases

#### Phase 1: Core Infrastructure (Weeks 1-2)
- [ ] Basic ACP client implementation using `agent-client-protocol`
- [ ] Message serialization/deserialization
- [ ] Session management and connection handling
- [ ] Basic TUI shell with ratatui
- [ ] Configuration system with TOML support

#### Phase 2: Claude Code Integration (Weeks 3-4)
- [ ] Claude Code subprocess adapter
- [ ] Permission system for file operations
- [ ] Basic edit review interface
- [ ] Terminal session embedding
- [ ] Error handling and recovery

#### Phase 3: Gemini Integration (Weeks 5-6)
- [ ] Gemini CLI integration as ACP agent
- [ ] Unified agent interface abstraction
- [ ] Agent switching and session management
- [ ] MCP server pass-through support
- [ ] Model selection and configuration

#### Phase 4: Visual Enhancement (Weeks 7-8)
- [ ] Tachyonfx integration for UI animations
- [ ] Code diff visualization with effects
- [ ] Syntax highlighting with color transitions
- [ ] Theme system implementation
- [ ] Status indicators and progress bars

#### Phase 5: Advanced Features (Weeks 9-10)
- [ ] Multi-session management
- [ ] Project-specific configurations
- [ ] Keybinding customization
- [ ] Plugin system for custom effects
- [ ] Performance profiling and optimization

#### Phase 6: Polish & Documentation (Weeks 11-12)
- [ ] Comprehensive testing suite
- [ ] User documentation and tutorials
- [ ] Installation and packaging
- [ ] Performance benchmarks
- [ ] Release preparation

### File Structure
```
rat/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs
│   ├── app.rs                    # Main application state
│   ├── config/
│   │   ├── mod.rs
│   │   ├── agent.rs             # Agent configurations
│   │   ├── ui.rs                # UI preferences
│   │   └── project.rs           # Project-specific settings
│   ├── acp/
│   │   ├── mod.rs
│   │   ├── client.rs            # ACP client implementation
│   │   ├── session.rs           # Session management
│   │   ├── message.rs           # Message handling
│   │   └── permissions.rs       # Permission system
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── claude_code.rs       # Claude Code adapter
│   │   ├── gemini.rs            # Gemini CLI adapter
│   │   └── traits.rs            # Common adapter interfaces
│   ├── ui/
│   │   ├── mod.rs
│   │   ├── app.rs               # Main UI coordinator
│   │   ├── chat.rs              # Chat interface
│   │   ├── diff.rs              # Edit review interface
│   │   ├── terminal.rs          # Terminal embedding
│   │   ├── statusbar.rs         # Status bar
│   │   └── components/          # Reusable UI components
│   ├── effects/
│   │   ├── mod.rs
│   │   ├── text.rs              # Text animation effects
│   │   ├── code.rs              # Code-specific effects
│   │   ├── transitions.rs       # UI transition effects
│   │   └── themes.rs            # Theme and color effects
│   └── utils/
│       ├── mod.rs
│       ├── diff.rs              # Diff utilities
│       ├── syntax.rs            # Syntax highlighting
│       └── terminal.rs          # Terminal utilities
├── examples/
│   ├── basic_client.rs          # Simple ACP client example
│   └── effects_demo.rs          # Tachyonfx effects showcase
├── tests/
│   ├── integration/
│   └── unit/
└── docs/
    ├── user_guide.md
    ├── configuration.md
    └── development.md
```

### Technical Challenges & Solutions

1. **ACP Protocol Complexity**
   - **Challenge**: Managing bidirectional JSON-RPC with multiple concurrent sessions
   - **Solution**: Use `agent-client-protocol` crate with careful async state management

2. **Cross-Platform Agent Integration**
   - **Challenge**: Different agents have varying installation and execution patterns
   - **Solution**: Abstract agent management with capability detection and auto-installation

3. **Performance with Visual Effects**
   - **Challenge**: Maintaining 60fps with complex tachyonfx animations
   - **Solution**: Selective effect application, frame rate limiting, and effect LOD system

4. **Terminal Integration**
   - **Challenge**: Embedding interactive terminals within TUI
   - **Solution**: Use `portable-pty` with custom rendering and input routing

5. **Edit Review UX**
   - **Challenge**: Making diff review intuitive and efficient
   - **Solution**: Hunk-level navigation with clear visual feedback and batch operations

### Success Metrics

1. **Performance**: Sub-100ms response times for UI interactions
2. **Reliability**: 99%+ uptime for agent connections
3. **Usability**: Intuitive interface requiring minimal learning curve
4. **Extensibility**: Plugin system supporting custom agents and effects
5. **Adoption**: Positive community feedback and contribution activity

### Risk Mitigation

1. **Agent API Changes**: Version pinning with update notifications
2. **Platform Compatibility**: Extensive testing on major platforms
3. **Performance Issues**: Profiling throughout development with optimization sprints
4. **User Experience**: Regular user testing and feedback incorporation

### Future Enhancements

- **Multi-Language Support**: Internationalization for global usage
- **Cloud Sync**: Configuration and session synchronization
- **Collaborative Features**: Shared sessions and pair programming
- **AI Training Integration**: Custom model fine-tuning support
- **Extension Marketplace**: Community-driven plugins and themes

---

This project positions RAT as the premier terminal-based interface for AI coding agents, combining the performance of Rust with the visual appeal of modern UIs, setting a new standard for developer-AI interaction paradigms.