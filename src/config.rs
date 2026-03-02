//! Configuration management

use crate::types::AgentConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub agent: AgentConfig,
    pub tools: ToolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    pub enabled: Vec<String>,
    pub config: serde_json::Value,
}

impl Config {
    /// Load configuration from file or environment
    pub fn load() -> Result<Self> {
        // Try to load from config file
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config from {:?}", config_path))?;
            let mut config: Config = toml::from_str(&content)
                .with_context(|| "Failed to parse config file")?;
            
            // Override with environment variables
            config.override_from_env()?;
            Ok(config)
        } else {
            // Create default config
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let config_dir = config_path.parent().unwrap();
        
        std::fs::create_dir_all(config_dir)
            .with_context(|| format!("Failed to create config directory {:?}", config_dir))?;
        
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        std::fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config to {:?}", config_path))?;
        
        Ok(())
    }

    /// Get configuration file path
    fn config_path() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .context("Failed to get home directory")?;
        Ok(home.join(".config").join("dex").join("config.toml"))
    }

    /// Override configuration from environment variables
    fn override_from_env(&mut self) -> Result<()> {
        // API Key
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            self.agent.api_key = api_key;
        }
        
        if let Ok(api_key) = std::env::var("DEX_API_KEY") {
            self.agent.api_key = api_key;
        }

        // Model
        if let Ok(model) = std::env::var("DEX_MODEL") {
            self.agent.model = model;
        }

        // API Base
        if let Ok(api_base) = std::env::var("DEX_API_BASE") {
            self.agent.api_base = Some(api_base);
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            agent: AgentConfig::default(),
            tools: ToolConfig::default(),
        }
    }
}

/// Initialize default configuration file
pub fn init_config() -> Result<()> {
    let config = Config::default();
    config.save()?;
    
    let config_path = Config::config_path()?;
    println!("✓ Created default config at: {}", config_path.display());
    println!("  Please edit it to add your API key.");
    
    Ok(())
}
