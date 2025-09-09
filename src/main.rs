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
