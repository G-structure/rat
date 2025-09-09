use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::fs;
use tokio::process::Command;
use which::which_in;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCommand {
    pub path: PathBuf,
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
}

impl AgentCommand {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            args: Vec::new(),
            env: None,
        }
    }

    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = Some(env);
        self
    }
}

pub struct AgentInstaller {
    data_dir: PathBuf,
}

impl AgentInstaller {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .or_else(|| dirs::home_dir().map(|h| h.join(".local/share")))
            .ok_or_else(|| anyhow::anyhow!("Cannot determine data directory"))?
            .join("rat")
            .join("agents");

        Ok(Self { data_dir })
    }

    /// Find or install Claude Code with robust fallback strategy
    pub async fn get_or_install_claude_code(&self) -> Result<AgentCommand> {
        info!("Looking for Claude Code agent...");

        // Strategy 1: Check if it's already in PATH
        if let Some(path) = self.find_in_path("claude-code-acp").await {
            info!("Found claude-code-acp in PATH: {}", path.display());
            return Ok(AgentCommand::new(path));
        }

        // Strategy 2: Check known installation locations
        let known_paths = self.get_claude_code_known_paths();
        for path in known_paths {
            if self.is_executable(&path).await {
                info!(
                    "Found claude-code-acp at known location: {}",
                    path.display()
                );
                return Ok(AgentCommand::new(path));
            }
        }

        // Strategy 3: Check our local installation
        if let Some(command) = self.get_local_claude_code().await? {
            return Ok(command);
        }

        // Strategy 4: Try to install locally
        info!("Claude Code not found, attempting local installation...");
        self.install_claude_code_locally().await
    }

    /// Find or install Gemini CLI with robust fallback strategy
    pub async fn get_or_install_gemini(&self) -> Result<AgentCommand> {
        info!("Looking for Gemini CLI agent...");

        // Strategy 1: Check if it's already in PATH
        if let Some(path) = self.find_in_path("gemini").await {
            info!("Found gemini in PATH: {}", path.display());
            // Add ACP flag if it supports it
            return Ok(AgentCommand::new(path).with_args(vec!["--experimental-acp".to_string()]));
        }

        // Strategy 2: Check known installation locations
        let known_paths = self.get_gemini_known_paths();
        for path in known_paths {
            if self.is_executable(&path).await {
                info!("Found gemini at known location: {}", path.display());
                return Ok(
                    AgentCommand::new(path).with_args(vec!["--experimental-acp".to_string()])
                );
            }
        }

        // Strategy 3: Check our local installation
        if let Some(command) = self.get_local_gemini().await? {
            return Ok(command);
        }

        // Strategy 4: Try to install locally
        info!("Gemini CLI not found, attempting local installation...");
        self.install_gemini_locally().await
    }

    /// Build a login command for Claude Code similar to Zed's flow.
    ///
    /// Preference order:
    /// - Use the locally installed claude-code-acp's node_modules to run
    ///   `@anthropic-ai/claude-code/cli.js /login` via `node`.
    /// - Fallback to a `claude` executable in PATH with `/login`.
    pub async fn get_claude_login_command(&self) -> Result<AgentCommand> {
        // Try to leverage the local claude-code installation we manage, and derive
        // the Claude Code CLI path from it (like Zed does).
        if let Some(local_acp_command) = self.get_local_claude_code().await? {
            // The first arg should be the JS entry path to the ACP adapter, e.g.
            //   .../node_modules/@zed-industries/claude-code-acp/dist/index.js
            // From that, derive:
            //   .../node_modules/@anthropic-ai/claude-code/cli.js
            if let Some(first_arg) = local_acp_command.args.get(0) {
                let acp_entry = PathBuf::from(first_arg);
                // Walk up to node_modules
                let node_modules_dir = acp_entry
                    .parent() // dist
                    .and_then(|p| p.parent()) // @zed-industries/claude-code-acp
                    .and_then(|p| p.parent()) // @zed-industries
                    .and_then(|p| p.parent()) // node_modules
                    .map(|p| p.to_path_buf());

                if let Some(node_modules_dir) = node_modules_dir {
                    let cli_js = node_modules_dir
                        .join("@anthropic-ai")
                        .join("claude-code")
                        .join("cli.js");
                    if cli_js.exists() {
                        return Ok(AgentCommand::new(PathBuf::from("node"))
                            .with_args(vec![cli_js.to_string_lossy().to_string(), "/login".into()]));
                    }
                }
            }
        }

        // Fallback: try a `claude` executable in PATH.
        if let Some(path) = self.find_in_path("claude").await {
            return Ok(AgentCommand::new(path).with_args(vec!["/login".into()]));
        }

        Err(anyhow::anyhow!(
            "Unable to locate Claude login CLI. Try installing @zed-industries/claude-code-acp or ensure `claude` is in PATH."
        ))
    }

    async fn find_in_path(&self, binary_name: &str) -> Option<PathBuf> {
        debug!("Searching for {} in PATH", binary_name);

        // Get current PATH from environment
        let path_var = std::env::var("PATH").ok()?;
        let current_dir = std::env::current_dir().ok()?;

        // Use which_in to search in PATH
        match which_in(binary_name, Some(&path_var), &current_dir) {
            Ok(path) => {
                debug!("Found {} at: {}", binary_name, path.display());
                Some(path)
            }
            Err(e) => {
                debug!("Could not find {} in PATH: {}", binary_name, e);
                None
            }
        }
    }

    async fn is_executable(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        // On Unix, check if it's executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = fs::metadata(path).await {
                let permissions = metadata.permissions();
                return permissions.mode() & 0o111 != 0;
            }
            return false;
        }

        // On Windows, check if it's a .exe file or has no extension
        #[cfg(windows)]
        {
            if let Some(ext) = path.extension() {
                return ext == "exe" || ext == "cmd" || ext == "bat";
            }
            return true; // Files without extension might be executable
        }

        #[cfg(not(any(unix, windows)))]
        return path.is_file();
    }

    fn get_claude_code_known_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Global npm installations
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home.clone()).join(".npm/bin/claude-code-acp"));
            paths.push(PathBuf::from(home).join(".local/bin/claude-code-acp"));
        }

        // System-wide npm installations
        paths.push(PathBuf::from("/usr/local/bin/claude-code-acp"));
        paths.push(PathBuf::from("/usr/bin/claude-code-acp"));

        // macOS specific paths
        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/opt/homebrew/bin/claude-code-acp"));
            paths.push(PathBuf::from(
                "/usr/local/Cellar/node/*/bin/claude-code-acp",
            ));
        }

        // Node modules (project-local)
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("node_modules/.bin/claude-code-acp"));
        }

        paths
    }

    fn get_gemini_known_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        // Global npm installations
        if let Ok(home) = std::env::var("HOME") {
            paths.push(PathBuf::from(home.clone()).join(".npm/bin/gemini"));
            paths.push(PathBuf::from(home).join(".local/bin/gemini"));
        }

        // System-wide npm installations
        paths.push(PathBuf::from("/usr/local/bin/gemini"));
        paths.push(PathBuf::from("/usr/bin/gemini"));

        // macOS specific paths
        #[cfg(target_os = "macos")]
        {
            paths.push(PathBuf::from("/opt/homebrew/bin/gemini"));
        }

        // Node modules (project-local)
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join("node_modules/.bin/gemini"));
        }

        paths
    }

    async fn get_local_claude_code(&self) -> Result<Option<AgentCommand>> {
        let claude_dir = self.data_dir.join("claude-code");
        if !claude_dir.exists() {
            return Ok(None);
        }

        // Look for the latest version
        let mut versions = Vec::new();
        let mut entries = fs::read_dir(&claude_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(version) = semver::Version::parse(name) {
                        let entry_path = claude_dir
                            .join(name)
                            .join("node_modules/@zed-industries/claude-code-acp/dist/index.js");
                        if self.is_executable(&entry_path).await || entry_path.exists() {
                            versions.push((version, entry_path));
                        }
                    }
                }
            }
        }

        if versions.is_empty() {
            return Ok(None);
        }

        versions.sort_by(|a, b| a.0.cmp(&b.0));
        let (version, entry_path) = versions.into_iter().last().unwrap();

        info!(
            "Using local Claude Code installation: {} (v{})",
            entry_path.display(),
            version
        );

        // Use node to run the JS entry point
        Ok(Some(
            AgentCommand::new(PathBuf::from("node"))
                .with_args(vec![entry_path.to_string_lossy().to_string()]),
        ))
    }

    async fn get_local_gemini(&self) -> Result<Option<AgentCommand>> {
        let gemini_dir = self.data_dir.join("gemini");
        if !gemini_dir.exists() {
            return Ok(None);
        }

        // Look for the latest version
        let mut versions = Vec::new();
        let mut entries = fs::read_dir(&gemini_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if entry.file_type().await?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if let Ok(version) = semver::Version::parse(name) {
                        let entry_path = gemini_dir
                            .join(name)
                            .join("node_modules/@google/gemini-cli/dist/index.js");
                        if self.is_executable(&entry_path).await || entry_path.exists() {
                            versions.push((version, entry_path));
                        }
                    }
                }
            }
        }

        if versions.is_empty() {
            return Ok(None);
        }

        versions.sort_by(|a, b| a.0.cmp(&b.0));
        let (version, entry_path) = versions.into_iter().last().unwrap();

        info!(
            "Using local Gemini CLI installation: {} (v{})",
            entry_path.display(),
            version
        );

        // Use node to run the JS entry point with ACP flag
        Ok(Some(AgentCommand::new(PathBuf::from("node")).with_args(
            vec![
                entry_path.to_string_lossy().to_string(),
                "--experimental-acp".to_string(),
            ],
        )))
    }

    async fn install_claude_code_locally(&self) -> Result<AgentCommand> {
        let package_name = "@zed-industries/claude-code-acp";
        let binary_name = "claude-code";

        let version_dir = self
            .install_npm_package_locally(package_name, binary_name)
            .await?;
        let entry_path = version_dir
            .join("node_modules")
            .join(package_name)
            .join("dist/index.js");

        if !entry_path.exists() {
            return Err(anyhow::anyhow!(
                "Entry point not found after installation: {}",
                entry_path.display()
            ));
        }

        info!(
            "Successfully installed Claude Code locally: {}",
            entry_path.display()
        );
        Ok(AgentCommand::new(PathBuf::from("node"))
            .with_args(vec![entry_path.to_string_lossy().to_string()]))
    }

    async fn install_gemini_locally(&self) -> Result<AgentCommand> {
        let package_name = "@google/gemini-cli";
        let binary_name = "gemini";

        let version_dir = self
            .install_npm_package_locally(package_name, binary_name)
            .await?;
        let entry_path = version_dir
            .join("node_modules")
            .join(package_name)
            .join("dist/index.js");

        if !entry_path.exists() {
            return Err(anyhow::anyhow!(
                "Entry point not found after installation: {}",
                entry_path.display()
            ));
        }

        info!(
            "Successfully installed Gemini CLI locally: {}",
            entry_path.display()
        );
        Ok(AgentCommand::new(PathBuf::from("node")).with_args(vec![
            entry_path.to_string_lossy().to_string(),
            "--experimental-acp".to_string(),
        ]))
    }

    async fn install_npm_package_locally(
        &self,
        package_name: &str,
        binary_name: &str,
    ) -> Result<PathBuf> {
        // Create agent-specific directory
        let agent_dir = self.data_dir.join(binary_name);
        fs::create_dir_all(&agent_dir)
            .await
            .with_context(|| format!("Failed to create directory: {}", agent_dir.display()))?;

        // Get latest version
        info!("Fetching latest version of {}...", package_name);
        let output = Command::new("npm")
            .args(&["view", package_name, "version", "--json"])
            .output()
            .await
            .with_context(|| format!("Failed to get version info for {}", package_name))?;

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "Failed to fetch package version: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let version_str = String::from_utf8(output.stdout)?
            .trim()
            .trim_matches('"')
            .to_string();

        let version_dir = agent_dir.join(&version_str);

        // Check if this version is already installed
        if version_dir.exists() {
            info!("Version {} already installed", version_str);
            return Ok(version_dir);
        }

        // Create temporary directory for installation
        let temp_dir = tempfile::tempdir_in(&agent_dir)
            .with_context(|| "Failed to create temporary directory")?;

        info!(
            "Installing {} version {} to local directory...",
            package_name, version_str
        );

        // Try multiple installation methods
        let mut install_success = false;
        let mut last_error = None;

        // Method 1: Try npm install with --prefix (works in most environments)
        let result = Command::new("npm")
            .args(&[
                "install",
                package_name,
                "--prefix",
                temp_dir.path().to_str().unwrap(),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .status()
            .await;

        match result {
            Ok(status) if status.success() => {
                install_success = true;
            }
            Ok(_) | Err(_) => {
                // Method 2: Try with npm init and then install
                debug!("Trying alternative installation method...");

                let init_result = Command::new("npm")
                    .args(&["init", "-y"])
                    .current_dir(temp_dir.path())
                    .stdout(Stdio::null())
                    .status()
                    .await;

                if let Ok(status) = init_result {
                    if status.success() {
                        let install_result = Command::new("npm")
                            .args(&["install", package_name])
                            .current_dir(temp_dir.path())
                            .stdout(Stdio::null())
                            .stderr(Stdio::piped())
                            .status()
                            .await;

                        if let Ok(status) = install_result {
                            if status.success() {
                                install_success = true;
                            }
                        }
                    }
                }

                if !install_success {
                    // Method 3: Try with --no-package-lock and --legacy-peer-deps
                    debug!("Trying installation with compatibility flags...");

                    let result = Command::new("npm")
                        .args(&[
                            "install",
                            package_name,
                            "--prefix",
                            temp_dir.path().to_str().unwrap(),
                            "--no-package-lock",
                            "--legacy-peer-deps",
                        ])
                        .stdout(Stdio::null())
                        .stderr(Stdio::piped())
                        .output()
                        .await;

                    match result {
                        Ok(output) => {
                            if output.status.success() {
                                install_success = true;
                            } else {
                                last_error =
                                    Some(String::from_utf8_lossy(&output.stderr).to_string());
                            }
                        }
                        Err(e) => {
                            last_error = Some(e.to_string());
                        }
                    }
                }
            }
        }

        if !install_success {
            let error_msg = last_error.unwrap_or_else(|| "Unknown installation error".to_string());
            return Err(anyhow::anyhow!(
                "Failed to install {}: {}",
                package_name,
                error_msg
            ));
        }

        // Move the installed package to the versioned directory
        fs::rename(temp_dir.into_path(), &version_dir)
            .await
            .with_context(|| format!("Failed to move installation to {}", version_dir.display()))?;

        info!(
            "Successfully installed {} version {}",
            package_name, version_str
        );
        Ok(version_dir)
    }

    pub async fn verify_agent_command(&self, command: &AgentCommand) -> Result<String> {
        debug!("Verifying agent command: {:?}", command);

        let output = Command::new(&command.path)
            .args(&command.args)
            .arg("--version")
            .output()
            .await
            .with_context(|| {
                format!("Failed to run version check for {}", command.path.display())
            })?;

        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            info!("Agent version: {}", version);
            Ok(version)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Version check failed: {}", error))
        }
    }
}

impl Default for AgentInstaller {
    fn default() -> Self {
        Self::new().expect("Failed to create AgentInstaller")
    }
}
