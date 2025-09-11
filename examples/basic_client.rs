use anyhow::Result;
use rat::app::App;
use rat::config::Config;

/// Basic example showing how to use RAT programmatically
#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    // Load or create default configuration
    let (config, config_file) = Config::load_or_create_default().await?;
    println!("Using configuration from: {:?}", config_file);

    // Create and run the application
    let mut app = App::new(config, None).await?;

    // You could programmatically connect to agents here
    // app.connect_agent("claude-code").await?;

    println!("Starting RAT application...");
    println!("Press 'q' to quit or Ctrl+C to force exit");

    // Run the TUI
    app.run().await?;

    println!("RAT application terminated");
    Ok(())
}
