# RAT ACP Permission Flow Analysis

## Overview

RAT implements a simplified ACP permission system focused on local development scenarios. Unlike Zed's comprehensive ACP implementation, RAT's permission flow is designed as a lightweight WebSocket bridge that intercepts agent tool calls and prompts users for permission through a browser interface.

## Core Architecture

### 1. WebSocket Bridge Architecture

RAT's permission system operates as a local WebSocket server that bridges external ACP agents with browser-based permission prompts:

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   External      │    │   RAT WS Bridge  │    │   Browser       │
│   ACP Agent     │────│   (local_ws.rs)  │────│   Client        │
│                 │    │                  │    │                 │
│ • Tool Call     │    │ • Message        │    │ • Permission    │
│   Request       │    │   Interception   │    │   Dialog        │
│                 │    │                  │    │                 │
│ • Wait for      │    │ • Permission     │    │ • User Choice   │
│   Permission    │    │   Request        │    │                 │
│                 │    │                  │    │ • Response      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 2. Key Components

#### WebSocket Server (`src/local_ws.rs`)

The core permission handling is implemented in the local WebSocket server:

```rust
async fn run_acp_bridge_local<WS, WR>(
    mut ws_write: WS,
    mut ws_read: WR,
    resolved_agent: Option<AgentCommand>,
) -> Result<()>
```

**Key Features:**
- Accepts WebSocket connections with ACP subprotocol support
- Intercepts JSON-RPC messages from agents
- Maintains pending permission state using `Arc<Mutex<HashMap<String, oneshot::Sender<bool>>>>`
- Handles permission responses from browser clients

#### Permission Interception Logic

RAT intercepts specific tool calls and converts them to permission requests:

```rust
// Track permission prompts awaiting a browser decision
let pending_perms: Arc<Mutex<HashMap<String, oneshot::Sender<bool>>>> = Arc::new(Mutex::new(HashMap::new()));
```

**Intercepted Operations:**
- `fs/write_text_file` - File writing operations
- `fs/read_text_file` - File reading operations
- `fs/mkdir` / `fs/create_dir` - Directory creation
- `fs/delete_file` / `fs/remove_file` - File deletion
- `fs/rename` / `fs/move` - File/directory renaming
- `terminal/execute` - Terminal command execution

## Detailed Implementation Analysis

### 1. Permission Request Flow

#### Message Interception

RAT parses agent stdout as NDJSON and intercepts tool calls:

```rust
// Try to treat output as NDJSON and intercept fs/* requests locally
if let Ok(text) = std::str::from_utf8(data) {
    for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
        let maybe_json: Result<serde_json::Value, _> = serde_json::from_str(line);
        if let Ok(v) = maybe_json {
            if let Some(m) = v.get("method").and_then(|x| x.as_str()) {
                // Intercept and handle permission requests
            }
        }
    }
}
```

#### Permission Dialog Creation

For each intercepted operation, RAT creates a permission request:

```rust
let perm_req = serde_json::json!({
    "jsonrpc": "2.0",
    "id": id_str,
    "method": "session/request_permission",
    "params": {
        "tool": "write_text_file",
        "reason": format!("Agent requested to write {}", path),
        "options": [
            {"id": "allow", "label": "Allow"},
            {"id": "deny", "label": "Deny"}
        ]
    }
});
```

#### Response Handling

RAT waits for browser responses and processes permission decisions:

```rust
// Intercept permission responses addressed to local bridge
if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
    let is_response = v.get("method").is_none() && v.get("id").is_some();
    if is_response {
        let id_str = id_key(&v["id"]).unwrap_or_default();
        if let Some(tx) = perms_for_ws.lock().await.remove(&id_str) {
            // Process permission decision
            let allowed = parse_permission_response(&v);
            let _ = tx.send(allowed);
        }
    }
}
```

### 2. Operation Execution

After permission is granted, RAT executes operations locally:

```rust
// Spawn a task to wait for decision and then perform the write + reply to agent
tokio::spawn(async move {
    let allowed = rx.await.unwrap_or(false);
    let resp = if allowed {
        // Try to write the file locally
        match tokio::fs::write(&path, content).await {
            Ok(_) => serde_json::json!({"jsonrpc":"2.0","id": id, "result": {}}),
            Err(e) => serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": format!("failed to write {}: {}", path, e)}}),
        }
    } else {
        serde_json::json!({"jsonrpc":"2.0","id": id, "error": {"code": -32000, "message": "permission denied"}})
    };
    // Send response back to agent
    let s = resp.to_string() + "\n";
    let _ = stdin_for_agent.lock().await.write_all(s.as_bytes()).await;
});
```

### 3. Terminal Command Handling

For terminal operations, RAT streams output back to the browser:

```rust
// Stream stdout
if let Some(mut out) = child.stdout.take() {
    let mut rdr = tokio::io::BufReader::new(out);
    loop {
        let mut line = String::new();
        match rdr.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let term = serde_json::json!({"jsonrpc":"2.0","method":"terminal/output","params": {"stream":"stdout","line": line.trim_end()}});
                let _ = ws_write2.lock().await.send(Message::Text(term.to_string())).await;
            }
            Err(_) => break,
        }
    }
}
```

## Key Differences from Zed's ACP Implementation

### 1. Architecture Comparison

| Aspect | RAT Implementation | Zed Implementation |
|--------|-------------------|-------------------|
| **Scope** | Local development only | Full IDE integration |
| **Protocol** | Direct WebSocket bridge | ACP protocol with MCP server |
| **UI Integration** | Browser-based dialogs | Native IDE UI components |
| **Settings** | Environment variables | Global user settings |
| **State Management** | Simple HashMap tracking | Complex state machine |
| **Security Model** | Local development (no encryption) | Production-ready with encryption |
| **Agent Support** | External agents only | Native + external agents |
| **Error Handling** | Basic error responses | Comprehensive error handling |

### 2. Permission Model Differences

#### RAT's Permission Model
- **Simplicity**: Basic allow/deny for each operation
- **Scope**: Operation-specific permissions
- **Persistence**: No persistent permission state
- **Configuration**: Environment variables (`RAT_ALLOWED_TOOLS`, `RAT_DISALLOWED_TOOLS`)
- **UI**: Browser-based permission dialogs

#### Zed's Permission Model
- **Granularity**: AllowOnce/AllowAlways/RejectOnce/RejectAlways
- **Settings**: Global `always_allow_tool_actions` setting
- **Persistence**: Settings persist across sessions
- **UI**: Native permission dialogs with icons and descriptions
- **State Machine**: Complex tool call status tracking

### 3. Technical Implementation Differences

#### Message Handling
**RAT:**
```rust
// Direct JSON parsing and interception
for line in text.split('\n').filter(|l| !l.trim().is_empty()) {
    let maybe_json: Result<serde_json::Value, _> = serde_json::from_str(line);
    if let Ok(v) = maybe_json {
        // Intercept and handle
    }
}
```

**Zed:**
```rust
// Structured ACP protocol with type safety
impl acp::Client for ClientDelegate {
    async fn request_permission(
        &self,
        arguments: acp::RequestPermissionRequest,
    ) -> Result<acp::RequestPermissionResponse, acp::Error> {
        // Type-safe permission handling
    }
}
```

#### Permission Options
**RAT:**
```json
{
  "options": [
    {"id": "allow", "label": "Allow"},
    {"id": "deny", "label": "Deny"}
  ]
}
```

**Zed:**
```rust
vec![
    acp::PermissionOption {
        id: acp::PermissionOptionId("always_allow".into()),
        name: "Always Allow".into(),
        kind: acp::PermissionOptionKind::AllowAlways,
    },
    // ... more options
]
```

### 4. Security and Reliability Differences

#### RAT's Approach
- **Security**: Explicitly for local development - "WARNING: No encryption, no authentication!"
- **Reliability**: Simple error handling with basic JSON-RPC error codes
- **Scope**: Limited to file system and terminal operations
- **Audit**: Basic logging only

#### Zed's Approach
- **Security**: Production-ready with encryption and authentication
- **Reliability**: Comprehensive error handling and recovery
- **Scope**: Extensible to any tool type
- **Audit**: Detailed logging and monitoring capabilities

## Configuration and Environment Variables

RAT uses environment variables for configuration:

```rust
// Agent resolution
let resolved_agent: Option<AgentCommand> = match env::var("RAT2E_AGENT_CMD") {
    Ok(cmd_path) => {
        // Configure agent from environment
    }
    Err(_) => {
        // Auto-resolve Claude Code or Gemini
    }
}
```

```rust
// Tool configuration
let allowed = std::env::var("RAT_ALLOWED_TOOLS")
    .ok()
    .filter(|s| !s.is_empty())
    .unwrap_or_else(|| "mcp__acp__read,mcp__acp__write".to_string());
```

## Use Cases and Limitations

### Appropriate Use Cases
- Local development and testing
- Rapid prototyping with ACP agents
- Development environments where security is less critical
- Learning and experimentation with ACP protocols

### Limitations
- No persistent permission settings
- Limited to browser-based UI
- No support for complex permission workflows
- Not suitable for production environments
- No integration with IDE settings or preferences

## Conclusion

RAT's ACP permission flow represents a minimal, development-focused implementation that prioritizes simplicity and ease of use over the comprehensive security and integration features found in Zed's implementation. While Zed provides a production-ready, extensible permission system with native UI integration, RAT offers a lightweight alternative suitable for local development scenarios where rapid iteration and minimal setup are prioritized.

The key insight is that RAT's approach is fundamentally different in scope and complexity - it's a development tool bridge rather than a full IDE integration, which explains the significant differences in architecture, security model, and feature set.