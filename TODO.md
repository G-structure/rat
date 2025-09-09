# RAT ACP Client Implementation TODO

## 🎉 STATUS UPDATE - ALMOST COMPLETE!

✅ **MAJOR ACHIEVEMENT**: Fixed the TUI blocking issue! Agent connections now work immediately when pressing 'n' key.

### 🚀 What's Working Now:
- ✅ Real ACP session creation (no more dummy UUIDs)
- ✅ Authentication flow with external `/login` command
- ✅ Agent connection and session management
- ✅ Message routing infrastructure (UI → App → Manager → ACP)
- ✅ Error handling and recovery mechanisms
- ✅ Terminal integration for authentication
- ✅ Status updates and feedback

### 🔧 Remaining Work:
- ✅ **ALL CRITICAL ITEMS COMPLETE!** Chat input IS properly connected to message sending via TUI manager
- ✅ All major functionality is implemented!

The application is **100% COMPLETE** for core functionality! Chat input works through the TUI manager's Enter key handling.

---

## Historical Context (Original Issues - Now Resolved)

The RAT application currently shows successful ACP connection logs but lacks interactive functionality. The core problems are:

1. **Dummy Session Management**: UI creates fake UUID sessions instead of real ACP sessions
2. **Disconnected Message Flow**: Chat input doesn't connect to the ACP client
3. **Missing Authentication Handling**: No detection or handling of `AUTH_REQUIRED` errors
4. **Incomplete UI-App Communication**: UI layer lacks proper channels to communicate with ACP

## Success Criteria

- Users can authenticate with Claude Code via `/login` command
- Users can create real ACP sessions and send messages
- Messages are sent through ACP protocol and responses are displayed
- Authentication errors are handled gracefully with login prompts
- Terminal integration for login commands works properly

## Implementation Plan

### Phase 1: Fix Session Management and Message Routing
Important the TUI must remain responsive, do not block the UI thread.
Need to handle the session creation asynchronously without blocking the UI
#### 1.1 Replace Dummy Sessions with Real ACP Sessions
**Files to modify:**
- `rat/src/ui/app.rs:183-196` - `create_new_session()` method
- `rat/src/app.rs:136-148` - `create_session()` method

**Zed reference:**
Make sure to look at the zed reference code before writing any code. Feel free to explore the zed codebase to understand the context of these reference snips.
- `zed/crates/agent_servers/src/acp.rs:151-165` - Real session creation with AUTH_REQUIRED handling
- `zed/crates/agent_ui/src/acp/thread_view.rs:310-330` - Connection to new_thread

**Tasks:**
- [x] Remove UUID session generation in UI layer
- [x] Wire `create_new_session()` to call `app.create_session(agent_name).await`
- [x] Handle session creation errors and display them in UI
- [x] Pass real SessionId back to UI for tab creation

**Implementation details:**
```rust
// In rat/src/ui/app.rs - create_new_session()
pub async fn create_new_session(&mut self) -> Result<()> {
    // Remove this line:
    // let session_id = SessionId(format!("session-{}", uuid::Uuid::new_v4()));

    // Add real session creation via message passing
    let (tx, rx) = oneshot::channel();
    self.message_tx.send(AppMessage::CreateSession {
        agent_name: "claude-code".to_string(),
        respond_to: tx,
    })?;

    let session_id = rx.await??;
    // Continue with tab creation using real session_id
}
```

#### 1.2 Connect Chat Input to ACP Client
**Files to modify:**
- `rat/src/ui/chat.rs:106` - Replace comment with actual message sending
- `rat/src/ui/app.rs` - Add message passing channel
- `rat/src/app.rs` - Handle message sending requests

**Zed reference:**
- `zed/crates/agent_ui/src/acp/message_editor.rs:send()` - Emits send event
- `zed/crates/agent_ui/src/acp/thread_view.rs:1247` - Handles MessageEditorEvent::Send
- `zed/crates/agent_ui/src/acp/thread_view.rs:1289-1350` - send_impl() method

**Tasks:**
- [x] Add message channel between UI and App layers
- [x] Create `SendMessage` variant in `AppMessage` enum
- [x] Handle message sending in app layer, route to appropriate ACP client
- [x] Convert input text to `acp::ContentBlock` format

**Implementation details:**
```rust
// In rat/src/ui/chat.rs - handle_key_event()
KeyCode::Enter => {
    if self.input_mode && !self.input_buffer.trim().is_empty() {
        // Send message through channel to app layer
        if let Some(sender) = &self.message_sender {
            let _ = sender.send(UiMessage::SendMessage {
                content: self.input_buffer.clone(),
            });
        }
        self.input_buffer.clear();
    }
    self.input_mode = false;
}
```

#### 1.3 Create Message Routing Infrastructure
**Files to modify:**
- `rat/src/app.rs` - Add message routing enum and handling
- `rat/src/ui/app.rs` - Add message sender to UI components

**Tasks:**
- [x] Create `UiMessage` enum for UI->App communication
- [x] Add message sender to chat components
- [x] Handle message routing in main app loop
- [x] Convert messages to proper ACP format before sending

### Phase 2: Implement Authentication Flow

#### 2.1 Detect Authentication Errors
**Files to modify:**
- `rat/src/acp/client.rs:116-130` - `create_session()` method
- `rat/src/adapters/claude_code.rs:85-95` - `create_session()` method

**Zed reference:**
- `zed/crates/agent_servers/src/acp.rs:151-165` - AUTH_REQUIRED error detection
- `zed/crates/agent_ui/src/acp/thread_view.rs:2168-2178` - Authentication error handling

**Tasks:**
- [x] Add error code checking in session creation
- [x] Create `AuthRequired` error type
- [x] Emit authentication required events to UI
- [x] Display authentication prompts in UI

**Implementation details:**
```rust
// In rat/src/acp/client.rs - create_session()
match connection.create_session().await {
    Err(err) if err.code == acp::ErrorCode::AUTH_REQUIRED.code => {
        // Emit auth required event
        let _ = self.message_tx.send(AppMessage::AuthRequired {
            agent_name: self.agent_name.clone(),
            method_id: None, // Will be populated from available methods
        });
        return Err(anyhow::anyhow!("Authentication required"));
    }
    Ok(session_id) => Ok(session_id),
    Err(err) => Err(err.into()),
}
```

#### 2.2 Implement Login Command Execution
**Files to modify:**
- `rat/src/adapters/claude_code.rs` - Add login command method
- `rat/src/ui/app.rs` - Add login UI handling

**Zed reference:**
- `zed/crates/agent_servers/src/claude.rs:19-35` - login_command() method
- `zed/crates/agent_ui/src/acp/thread_view.rs:2297-2337` - spawn_claude_login()

**Tasks:**
- [x] Implement `trigger_login_flow()` method in claude_code adapter
- [x] Spawn terminal process with `/login` command
- [x] Add terminal integration for login interaction
- [x] Handle login completion and retry session creation

SEE THESE TWO EXAMPLES I WROTE:
- `/Users/luc/projects/vibes/rat/examples/simple_acp_login_check.rs`
- `/Users/luc/projects/vibes/rat/examples/simple_acp_test.rs`

**Implementation details:**
```rust
// In rat/src/adapters/claude_code.rs
async fn trigger_login_flow(&self) -> Result<()> {
    let command = self.get_or_install_command().await?;

    // Spawn login command in terminal
    let mut login_process = Command::new(&command.path)
        .args(&["/login".to_string()])
        .spawn()?;

    // Wait for login to complete
    login_process.wait().await?;
    Ok(())
}
```

#### 2.3 Add Authentication Methods Support
**Files to modify:**
- `rat/src/acp/client.rs` - Add auth methods tracking
- `rat/src/ui/app.rs` - Display authentication options

**Zed reference:**
- `zed/crates/acp_thread/src/connection.rs:40-44` - auth_methods() trait method
- `zed/crates/agent_ui/src/acp/thread_view.rs:2143-2157` - Authentication method selection

**Tasks:**
- [x] Store and expose authentication methods from ACP connection
- [x] Create UI for selecting authentication method
- [x] Handle different authentication flows (API key, login, etc.)
- [x] Retry connection after successful authentication

### Phase 3: Terminal Integration and Polish

#### 3.1 Add Terminal Support for Login
**Files to modify:**
- `rat/src/ui/terminal.rs` - Existing terminal UI component
- `rat/src/adapters/claude_code.rs` - Terminal integration

**Zed reference:**
- Terminal spawning in `zed/crates/agent_ui/src/acp/thread_view.rs:2297-2337`

**Tasks:**
- [x] Integrate existing terminal component with login flow
- [x] Handle terminal output and user input for authentication
- [x] Manage terminal lifecycle (create, destroy, cleanup)
- [x] Switch back to chat interface after login completion

#### 3.2 Improve Error Handling and User Feedback
**Files to modify:**
- `rat/src/ui/app.rs` - Error display improvements
- `rat/src/adapters/claude_code.rs` - Better error propagation

**Tasks:**
- [x] Add specific error messages for different failure types
- [x] Implement retry mechanisms for failed connections
- [x] Show loading states during authentication
- [x] Add success notifications when authentication completes

### Phase 4: Message Display and Streaming

#### 4.1 Handle ACP Session Updates
**Files to modify:**
- `rat/src/acp/client.rs` - Session notification handling
- `rat/src/ui/chat.rs` - Message display improvements

**Zed reference:**
- `zed/crates/agent_servers/src/acp.rs:261-271` - session_notification handling
- `zed/crates/agent_ui/src/acp/thread_view.rs` - Message rendering

**Tasks:**
- [x] Implement proper session_notification callback
- [x] Handle streaming message updates
- [x] Display tool calls and their results
- [x] Show agent status updates (thinking, working, etc.)

#### 4.2 Add Content Block Support
**Files to modify:**
- `rat/src/ui/chat.rs` - Rich content rendering
- `rat/src/acp/message.rs` - Content block conversion

**Zed reference:**
- Content block handling in message editor and display components

**Tasks:**
- [x] Support text, image, and other content block types
- [x] Implement markdown rendering for agent responses
- [x] Add syntax highlighting for code blocks
- [x] Handle file references and mentions

## File Architecture Comparison

### RAT Current Structure
```
rat/src/
├── acp/
│   ├── client.rs       # ACP client implementation (working)
│   ├── message.rs      # Message types
│   └── session.rs      # Session management
├── adapters/
│   ├── claude_code.rs  # Claude Code adapter (connection works)
│   └── manager.rs      # Agent management
├── ui/
│   ├── app.rs          # UI management (dummy sessions)
│   ├── chat.rs         # Chat interface (no message sending)
│   └── terminal.rs     # Terminal UI component
└── app.rs              # Main application (incomplete routing)
```

### Zed Reference Structure
```
zed/crates/
├── agent_servers/src/
│   ├── acp.rs          # ACP connection management ⭐
│   ├── claude.rs       # Claude Code server ⭐
│   └── ...
├── agent_ui/src/acp/
│   ├── thread_view.rs  # Main chat interface ⭐
│   ├── message_editor.rs # Input handling ⭐
│   └── ...
├── acp_thread/src/
│   ├── acp_thread.rs   # Core thread logic ⭐
│   ├── connection.rs   # Connection trait ⭐
│   └── ...
```

⭐ = Key reference files for implementation

## Testing Strategy

### Unit Tests
- [x] Test session creation with mocked ACP responses
- [x] Test authentication error handling
- [x] Test message routing between UI and ACP client
- [x] Test content block conversion and display

### Integration Tests
- [x] Test full authentication flow with real Claude Code
- [x] Test message sending and receiving
- [x] Test error recovery and retry mechanisms
- [x] Test terminal integration for login

### Manual Testing Checklist
- [x] Start RAT and see connection logs
- [x] Try to create session, should show auth prompt
- [x] Complete authentication via `/login` command
- [x] Send a message and receive response
- [x] Test different content types (text, code, etc.)
- [x] Test error scenarios and recovery

## Implementation Order

1. **Session Management** (Phase 1.1) - Foundation for everything else
2. **Message Routing** (Phase 1.2-1.3) - Enable basic communication
3. **Auth Detection** (Phase 2.1) - Detect when authentication is needed
4. **Login Flow** (Phase 2.2-2.3) - Enable authentication
5. **Polish** (Phase 3-4) - Improve UX and add features

This plan provides a clear path from the current working-but-disconnected state to a fully functional ACP client UI.
