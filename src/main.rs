use anyhow::Result;
use clap::Parser;
use log::{info, warn};

mod app;
mod acp;
mod adapters;
mod config;
mod effects;
mod ui;
mod utils;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = match cli.verbose {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    env_logger::Builder::from_default_env()
        .filter_level(log_level)
        .init();

    info!("Starting RAT (Rust Agent Terminal) v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = match cli.config {
        Some(path) => {
            info!("Loading configuration from: {}", path);
            Config::from_file(&path).await?
        }
        None => {
            info!("Using default configuration");
            Config::default()
        }
    };

    // Initialize and run the application
    let mut app = App::new(config).await?;

    if let Some(agent_name) = cli.agent {
        info!("Starting with agent: {}", agent_name);
        app.connect_agent(&agent_name).await?;
    }

    // Run the TUI
    app.run().await?;

    info!("RAT terminated successfully");
    Ok(())
}
