# FRP Integration for Remote ACP Agent Access

## Overview

This document explains how to use [frp](https://github.com/fatedier/frp) (Fast Reverse Proxy) to expose ACP (Agent Client Protocol) messages over the network, enabling remote access to AI coding agents like Claude Code and Gemini CLI. This allows RAT (Rust Agent Terminal) to connect to agents running on remote machines or behind firewalls.

## Why FRP?

- **Open-source ngrok alternative**: frp provides similar tunneling capabilities to ngrok but is self-hosted and free
- **Per-agent URLs**: Each agent instance can get its own public URL for secure access
- **Protocol agnostic**: Works with TCP connections, perfect for ACP's JSON-RPC over TCP
- **Security**: No need to expose agent ports directly to the internet

## Architecture

```
Remote Machine (Agent)                Public Server (frps)          Local Machine (RAT)
┌───────────────────────────┐        ┌─────────────────────┐        ┌───────────────────────┐
│                           │        │                     │        │                       │
│ Agent (Claude/Gemini)     │◄───────┤ frps (server)       │        │ RAT TUI               │
│ listens on localhost:port │        │ exposes public URL  │        │ connects to public URL│
│                           │        │                     │        │                       │
│ frpc (client)             │───────►│                     │        │                       │
│ tunnels port to frps      │        │                     │        │                       │
└───────────────────────────┘        └─────────────────────┘        └───────────────────────┘
```

## Setup Instructions

### 1. Install FRP

Download the latest frp release from [GitHub](https://github.com/fatedier/frp/releases):

```bash
# On both server and client machines
wget https://github.com/fatedier/frp/releases/download/v0.58.0/frp_0.58.0_linux_amd64.tar.gz
tar -xzf frp_0.58.0_linux_amd64.tar.gz
cd frp_0.58.0_linux_amd64/
```

### 2. Set Up FRP Server (frps)

On a machine with a public IP address:

1. Create `frps.toml`:
```toml
bindPort = 7000
# Optional: enable dashboard
webServer.addr = "0.0.0.0"
webServer.port = 7500
webServer.user = "admin"
webServer.password = "admin"
```

2. Start the server:
```bash
./frps -c ./frps.toml
```

### 3. Configure Agent with FRP Tunneling

For each agent instance, create a unique tunnel. Here's how to integrate it with RAT's agent startup:

#### Option A: Manual Configuration

1. Start your agent normally (it will listen on a local port, e.g., 3000)
2. Create `frpc.toml` for this agent:
```toml
serverAddr = "your-frps-server.com"
serverPort = 7000

[[proxies]]
name = "claude-agent-1"
type = "tcp"
localIP = "127.0.0.1"
localPort = 3000  # The port your agent is listening on
remotePort = 0    # Let frps assign a random port
```

3. Start frpc:
```bash
./frpc -c ./frpc.toml
```

4. Check the frps dashboard or logs to get the assigned public port
5. Configure RAT to connect to `your-frps-server.com:assigned-port`

#### Option B: Automated Per-Agent Tunneling

Modify RAT's agent adapters to automatically start frpc when launching agents:

```rust
// In src/adapters/claude_code.rs
use std::process::Command;

impl ClaudeCodeAdapter {
    pub async fn start_with_tunnel(&self, frps_config: &FrpConfig) -> Result<String> {
        // Start the agent
        let agent_port = self.start_agent()?;

        // Generate unique tunnel config
        let tunnel_name = format!("claude-{}", uuid::Uuid::new_v4());
        let remote_port = self.create_frp_config(&tunnel_name, agent_port, frps_config)?;

        // Start frpc
        Command::new("frpc")
            .arg("-c")
            .arg(format!("frpc-{}.toml", tunnel_name))
            .spawn()?;

        // Return the public URL
        Ok(format!("{}:{}", frps_config.server_addr, remote_port))
    }

    fn create_frp_config(&self, name: &str, local_port: u16, config: &FrpConfig) -> Result<u16> {
        let content = format!(r#"
serverAddr = "{}"
serverPort = {}

[[proxies]]
name = "{}"
type = "tcp"
localIP = "127.0.0.1"
localPort = {}
remotePort = 0
"#,
            config.server_addr, config.server_port, name, local_port
        );

        std::fs::write(format!("frpc-{}.toml", name), content)?;

        // For now, return a placeholder - in practice, you'd need to query frps
        // or use a fixed port range
        Ok(20000 + (local_port % 1000)) // Simple port assignment
    }
}
```

### 4. Configure RAT for Remote Agents

Update your `rat/config.toml`:

```toml
[agents.claude_code]
# Instead of localhost
remote_url = "your-frps-server.com:20001"

[frp]
server_addr = "your-frps-server.com"
server_port = 7000
auto_tunnel = true
```

### 5. Dynamic URL Assignment

For truly dynamic URLs per agent boot:

1. **Use FRP's HTTP proxy** for web-based access (if converting ACP to HTTP)
2. **Implement a URL registry service** that tracks active tunnels
3. **Use FRP's subdomain feature** with wildcard DNS

Example with HTTP proxy:

```toml
# frpc.toml
[[proxies]]
name = "claude-agent-http"
type = "http"
localIP = "127.0.0.1"
localPort = 3000
customDomains = ["claude-agent-1.yourdomain.com"]
```

This gives each agent a unique subdomain like `claude-agent-1.yourdomain.com`.

## Security Considerations

1. **Authentication**: Configure FRP with authentication:
```toml
# frps.toml
auth.method = "token"
auth.token = "your-secret-token"
```

2. **TLS**: Enable TLS for encrypted connections:
```toml
# frps.toml
transport.tls.enable = true
transport.tls.certFile = "server.crt"
transport.tls.keyFile = "server.key"
```

3. **Firewall**: Only expose necessary ports on your FRP server

4. **Access Control**: Use FRP's permission system to limit which clients can create tunnels

## Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure each agent gets a unique local port
2. **Firewall blocking**: Check that FRP ports are open on server
3. **DNS issues**: For custom domains, ensure wildcard DNS is configured
4. **Connection timeouts**: Increase FRP timeouts for slow networks

### Monitoring

- Use FRP's dashboard at `http://your-server:7500`
- Check frpc/frps logs for connection issues
- Monitor network latency for remote agents

## Future Enhancements

1. **Automatic tunnel management**: Clean up tunnels when agents shut down
2. **Load balancing**: Distribute agents across multiple FRP servers
3. **Web-based RAT**: Convert RAT to a web application for browser access
4. **P2P mode**: Use FRP's xtcp for direct peer-to-peer connections
5. **Integration with cloud providers**: Auto-setup FRP on cloud instances

## Alternative Solutions

If FRP doesn't meet your needs, consider:
- **ngrok**: Commercial tunneling service
- **Cloudflare Tunnel**: Free for basic usage
- **Tailscale**: Mesh VPN for private networking
- **WireGuard**: Self-hosted VPN solution

## References

- [FRP Documentation](https://github.com/fatedier/frp)
- [ACP Specification](https://agentclientprotocol.com/)
- [RAT Configuration](README.md#configuration)
