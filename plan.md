# RAT (Rust Agent Terminal)
## High-Performance ACP Client for Claude Code & Gemini

### Executive Summary

RAT is a high-performance terminal-based ACP (Agent Client Protocol) client written in Rust, leveraging tachyonfx for stunning visual effects. The project creates a unified interface for interacting with multiple AI coding agents (Claude Code and Gemini CLI) through a standardized protocol, providing a superior alternative to traditional terminal interactions with rich visual feedback, structured edit reviews, and **multi-agent control capabilities**. 

RAT enables developers to manage and control multiple agents simultaneously - switching between different agents for different tasks, running parallel sessions, and maintaining multiple concurrent conversations with different AI models.

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
   - **Multi-Agent Manager**: Control multiple agent instances simultaneously
   - **Health Monitoring**: Agent availability and capability detection

3. **TUI Framework** (`src/ui/`)
   - **Main Interface**: Tabbed layout supporting multiple concurrent agent sessions
   - **Agent Selector**: Quick switching between active agents with visual indicators
   - **Chat View**: Message threading with agent identification and syntax highlighting
   - **Edit Review**: Diff viewer with hunk-level accept/reject using tachyonfx transitions
   - **Terminal Integration**: Embedded terminal sessions with streaming output
   - **Multi-Agent Dashboard**: Overview of all active agents and their current tasks
   - **Status Bar**: Real-time multi-agent status, session info, and progress indicators

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

1. **Multi-Agent Control**
   - Simultaneous control of multiple Claude Code and Gemini agent instances
   - Tabbed interface for managing concurrent agent sessions
   - Quick agent switching with session preservation
   - Per-agent configuration and state management
   - Parallel task execution across different agents

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

#### Phase 1: Core Infrastructure (Weeks 1-2) - ✅ MOSTLY COMPLETE
- [ ] **Basic ACP client implementation using `agent-client-protocol`** - ⚠️ NEEDS COMPLETION (client has dummy implementations)
- [x] **Message serialization/deserialization** - ✅ COMPLETED
- [ ] **Session management and connection handling** - ⚠️ PARTIAL (structure exists, needs real ACP integration)
- [x] **Basic TUI shell with ratatui** - ✅ COMPLETED (tabbed interface, keybindings, welcome screen)
- [x] **Configuration system with TOML support** - ✅ COMPLETED

#### Phase 2: Claude Code Integration (Weeks 3-4) - ⚠️ IN PROGRESS
- [ ] **Claude Code subprocess adapter** - ⚠️ PARTIAL (structure exists, needs implementation)
- [ ] **Permission system for file operations** - ⚠️ PARTIAL (basic structure, needs real ACP integration)
- [ ] **Basic edit review interface** - ❌ NOT STARTED (UI structure exists but diff logic missing)
- [ ] **Terminal session embedding** - ❌ NOT STARTED
- [x] **Error handling and recovery** - ✅ MOSTLY COMPLETE (basic error handling in place)

#### Phase 3: Gemini Integration (Weeks 5-6) - ⚠️ IN PROGRESS  
- [ ] **Gemini CLI integration as ACP agent** - ⚠️ PARTIAL (structure exists, needs implementation)
- [x] **Unified agent interface abstraction** - ✅ COMPLETED (AgentAdapter trait implemented)
- [x] **Agent switching and session management** - ✅ MOSTLY COMPLETE (AgentManager handles multiple agents)
- [ ] **MCP server pass-through support** - ❌ NOT STARTED
- [ ] **Model selection and configuration** - ⚠️ PARTIAL (config structure exists)

#### Phase 4: Visual Enhancement (Weeks 7-8) - ❌ EARLY STAGE
- [ ] **Tachyonfx integration for UI animations** - ⚠️ DEPENDENCY ADDED (effects modules exist but mostly empty)
- [ ] **Code diff visualization with effects** - ❌ NOT STARTED
- [ ] **Syntax highlighting with color transitions** - ⚠️ PARTIAL (basic structure exists)  
- [ ] **Theme system implementation** - ⚠️ PARTIAL (config support added)
- [x] **Status indicators and progress bars** - ✅ BASIC IMPLEMENTATION (status bar exists)

#### Phase 5: Advanced Features (Weeks 9-10) - ❌ EARLY STAGE
- [x] **Multi-session management** - ✅ COMPLETED (tabbed sessions, session switching implemented)
- [x] **Project-specific configurations** - ✅ COMPLETED (config system supports per-project settings)
- [x] **Keybinding customization** - ✅ COMPLETED (config system with keybinding support)
- [ ] **Plugin system for custom effects** - ❌ NOT STARTED
- [ ] **Performance profiling and optimization** - ❌ NOT STARTED

#### Phase 6: Polish & Documentation (Weeks 11-12) - ❌ NOT STARTED
- [ ] **Comprehensive testing suite** - ⚠️ PARTIAL (basic test structure exists)
- [ ] **User documentation and tutorials** - ❌ NOT STARTED  
- [ ] **Installation and packaging** - ❌ NOT STARTED
- [ ] **Performance benchmarks** - ❌ NOT STARTED
- [ ] **Release preparation** - ❌ NOT STARTED

---

## CURRENT STATUS SUMMARY (Updated: December 2024)

### ✅ **COMPLETED AREAS (~40% of project)**
- **Project Structure & Build System**: Full Rust project with proper dependencies
- **Configuration System**: TOML-based config with agent, UI, and project settings
- **Core Application Framework**: Event loop, async architecture, message passing
- **Basic TUI**: Tabbed interface, keybindings, status bar, welcome screen
- **Multi-Agent Architecture**: AgentManager, adapter pattern, session management
- **Multi-Session Support**: Concurrent sessions with tab switching

### ⚠️ **PARTIALLY COMPLETE AREAS (~30% of project)**
- **ACP Client Core**: Structure exists but needs real protocol implementation
- **Agent Adapters**: Framework in place, needs actual subprocess management
- **Permission System**: Basic structure, needs ACP integration
- **Effects System**: Dependencies added, modules exist but mostly empty
- **Message Routing**: Basic async messaging, needs ACP protocol integration

### ❌ **MISSING CRITICAL COMPONENTS (~30% of project)**
- **Real ACP Protocol Communication**: Currently using dummy implementations
- **Subprocess Management**: Agent processes not actually started/managed
- **Edit Review & Diff Visualization**: Core feature missing
- **Terminal Embedding**: No embedded terminal functionality
- **TachyonFX Visual Effects**: Minimal implementation
- **Testing & Documentation**: Comprehensive coverage missing

### 🚨 **IMMEDIATE PRIORITIES**
1. **Fix ACP Client Implementation** - Replace dummy implementations with real ACP protocol
2. **Implement Agent Subprocess Management** - Actually start and communicate with agents  
3. **Add Basic Edit Review** - Core diff viewing and approval workflow
4. **Test End-to-End Functionality** - Ensure agent communication works

The project has excellent architectural foundations but needs focused work on the core ACP functionality to become functional.

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
│   │   ├── manager.rs           # Multi-agent instance manager
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