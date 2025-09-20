use rat::config::{AgentConfig, Config};
use tempfile::TempDir;
use tokio;

#[tokio::test]
async fn test_default_config() {
    let config = Config::default();

    assert!(config.agents.claude_code.enabled);
    assert!(config.agents.gemini.enabled);
    assert_eq!(config.agents.default_agent, "claude-code");
    assert!(config.ui.theme.syntax_highlighting);
    assert!(config.ui.effects.enabled);
    assert!(config.general.auto_save_sessions);
}

#[tokio::test]
async fn test_config_validation() {
    let config = Config::default();
    assert!(config.validate().is_ok());

    // Test invalid config
    let mut invalid_config = Config::default();
    invalid_config.general.max_session_history = 0;
    assert!(invalid_config.validate().is_err());
}

#[tokio::test]
async fn test_config_save_load() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("test_config.toml");

    let original_config = Config::default();
    original_config.save_to_file(&config_path).await.unwrap();

    let loaded_config = Config::from_file(&config_path).await.unwrap();
    assert_eq!(
        original_config.agents.default_agent,
        loaded_config.agents.default_agent
    );
    assert_eq!(original_config.ui.theme.name, loaded_config.ui.theme.name);
}

#[tokio::test]
async fn test_agent_config_merge() {
    let mut base_config = AgentConfig::default();
    let mut override_config = AgentConfig::default();
    override_config.default_agent = "gemini".to_string();

    base_config.merge_with(override_config);
    assert_eq!(base_config.default_agent, "gemini");
}
