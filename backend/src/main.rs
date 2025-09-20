use anyhow::Result;
use clap::Parser;
use log::{info, warn};
use std::fs::OpenOptions;
use std::io::Write;

mod acp;
mod adapters;
mod app;
mod config;
mod effects;
mod pairing;
mod ui;
mod utils;
mod local_ws;

use app::App;
use config::Config;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Verbose logging
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Agent to start with (claude-code, gemini)
    #[arg(short, long)]
    agent: Option<String>,

    /// Override agent command (path or program). When set, RAT registers
    /// an external agent with this command and optional args.
    #[arg(long)]
    agent_cmd: Option<String>,

    /// Arguments for --agent-cmd; can be repeated. Hyphenated values allowed.
    #[arg(long = "agent-arg", allow_hyphen_values = true)]
    agent_args: Vec<String>,

    /// Name to register the external agent under (default: "sim").
    #[arg(long)]
    agent_name: Option<String>,

    /// Disable all effects (theme animations, chat sweeps, etc.)
    #[arg(long)]
    no_effects: bool,

    /// Disable startup intro animation (Matrix rain morph)
    #[arg(long)]
    no_intro: bool,

    /// Start pairing mode for hosted UI
    #[arg(long)]
    pair: bool,

    /// Start local WebSocket server for direct connections (development mode)
    #[arg(long)]
    local_ws: bool,

    /// Port for local WebSocket server (default: 8081)
    #[arg(long, default_value = "8081")]
    local_port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.pair {
        crate::pairing::start_pairing().await?;
        return Ok(());
    }

    if cli.local_ws {
        crate::local_ws::start_local_ws_server(cli.local_port).await?;
        return Ok(());
    }

    // Initialize logging
    let log_level = match cli.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    // Create logs directory if it doesn't exist
    std::fs::create_dir_all("logs").unwrap_or_else(|e| {
        eprintln!("Failed to create logs directory: {}", e);
    });

    // Set up file logging
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("logs/rat.log")
        .expect("Failed to create log file");

    // Allow environment variable override for log level
    let mut builder = env_logger::Builder::from_default_env();

    // If RUST_LOG is not set, use the CLI verbose level
    if std::env::var("RUST_LOG").is_err() {
        builder.filter_level(log_level);
    }

    builder
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .format_timestamp_secs()
        .format_module_path(true)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] [{}:{}] [{}] - {}",
                buf.timestamp(),
                record.level(),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.module_path().unwrap_or("unknown"),
                record.args()
            )
        })
        .init();

    info!(
        "Starting RAT (Rust Agent Terminal) v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Load configuration
    let mut config = match cli.config {
        Some(path) => {
            info!("Loading configuration from: {}", path);
            Config::from_file(&path).await?
        }
        None => {
            info!("Using default configuration");
            Config::default()
        }
    };

    // CLI overrides for effects
    if cli.no_effects {
        config.ui.effects.enabled = false;
    }
    if cli.no_intro {
        config.ui.effects.startup.enabled = false;
    }

    // Initialize and run the application
    // Build optional external agent spec from CLI
    let external = if let Some(cmd) = cli.agent_cmd.clone() {
        let name = cli.agent_name.clone().unwrap_or_else(|| "sim".to_string());
        Some(crate::adapters::ExternalAgentSpec {
            name,
            path: cmd,
            args: cli.agent_args.clone(),
            env: None,
        })
    } else {
        None
    };

    let mut app = App::new(config, external.clone()).await?;

    if let Some(agent_name) = cli.agent.or_else(|| external.as_ref().map(|e| e.name.clone())) {
        info!("Starting with agent: {}", agent_name);
        app.connect_agent(&agent_name).await?;
    }

    // Run the TUI
    app.run().await?;

    info!("RAT terminated successfully");
    Ok(())
}
