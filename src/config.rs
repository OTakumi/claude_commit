//! Configuration management for Claude Commit
//!
//! This module handles loading and parsing configuration files in TOML format.
//! The configuration contains the prompt template to be sent to Claude AI.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

/// Prompt configuration file structure
///
/// # Example TOML
///
/// ```toml
/// prompt = """
/// Generate a concise git commit message based on the following diff.
/// Use conventional commits format (feat:, fix:, docs:, etc.).
/// """
///
/// # Optional: Maximum combined size of prompt + diff in bytes (default: 1,000,000)
/// max_prompt_size = 1000000
/// ```
#[derive(Deserialize)]
pub struct Config {
    /// Prompt template to send to Claude
    pub prompt: String,
    /// Maximum combined size of prompt template and git diff in bytes
    /// Defaults to 1MB (1,000,000 bytes)
    #[serde(default = "default_max_prompt_size")]
    pub max_prompt_size: usize,
}

/// Default maximum prompt size: 1MB
fn default_max_prompt_size() -> usize {
    1_000_000
}

/// Load configuration from a TOML file
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
///
/// # Returns
///
/// * `Result<Config>` - Parsed configuration
///
/// # Errors
///
/// * File does not exist
/// * Invalid TOML format
/// * Missing required fields
/// * Prompt field is empty or whitespace-only
///
/// # Example
///
/// ```no_run
/// use claude_commit::config::load_config;
///
/// # fn main() -> anyhow::Result<()> {
/// let config = load_config("prompt.toml")?;
/// println!("Prompt: {}", config.prompt);
/// # Ok(())
/// # }
/// ```
pub fn load_config(config_path: &str) -> Result<Config> {
    let content = fs::read_to_string(config_path)
        .context(format!("Failed to read config file: {}", config_path))?;
    let config: Config = toml::from_str(&content).context("Failed to parse config file as TOML")?;

    // Validate prompt is not empty or whitespace-only
    if config.prompt.trim().is_empty() {
        anyhow::bail!(
            "Configuration error: 'prompt' field cannot be empty or whitespace-only. \
             Please provide a valid prompt template in {}",
            config_path
        );
    }

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config_deserialize_valid_toml() {
        // Arrange - valid TOML string
        let toml_str = r#"
prompt = "Generate a concise commit message:"
"#;

        // Act
        let result: Result<Config, _> = toml::from_str(toml_str);

        // Assert - should parse successfully
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.prompt, "Generate a concise commit message:");
    }

    #[test]
    fn test_config_deserialize_missing_prompt_field() {
        // Arrange - TOML without prompt field
        let toml_str = r#"
other_field = "value"
"#;

        // Act
        let result: Result<Config, _> = toml::from_str(toml_str);

        // Assert - should return error (prompt is required)
        assert!(result.is_err());
    }

    #[test]
    fn test_config_deserialize_invalid_toml() {
        // Arrange - invalid TOML format
        let toml_str = r#"
prompt = "unclosed quote
invalid syntax here
"#;

        // Act
        let result: Result<Config, _> = toml::from_str(toml_str);

        // Assert - should return error
        assert!(result.is_err());
    }

    #[test]
    fn test_config_deserialize_empty_prompt() {
        // Arrange - empty prompt string (but field exists)
        let toml_str = r#"
prompt = ""
"#;

        // Act
        let result: Result<Config, _> = toml::from_str(toml_str);

        // Assert - should parse successfully (empty string is valid)
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.prompt, "");
    }

    #[test]
    fn test_config_deserialize_multiline_prompt() {
        // Arrange - multiline prompt using TOML multiline string
        let toml_str = r#"
prompt = """
Line 1: Generate a commit message
Line 2: Based on the following diff
Line 3: Use conventional commits format
"""
"#;

        // Act
        let result: Result<Config, _> = toml::from_str(toml_str);

        // Assert - should parse successfully with newlines preserved
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.prompt.contains("Line 1"));
        assert!(config.prompt.contains("Line 2"));
        assert!(config.prompt.contains("Line 3"));
        assert!(config.prompt.contains('\n'));
    }

    #[test]
    fn test_config_deserialize_special_characters_in_prompt() {
        // Arrange - prompt with special characters
        let toml_str = r#"
prompt = "Use 日本語 and emojis 🎉 in message. Escape \"quotes\" and \ttabs."
"#;

        // Act
        let result: Result<Config, _> = toml::from_str(toml_str);

        // Assert - special characters should be preserved
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.prompt.contains("日本語"));
        assert!(config.prompt.contains("🎉"));
        assert!(config.prompt.contains("\"quotes\""));
    }
}
