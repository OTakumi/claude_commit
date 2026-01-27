//! CLI tool to automatically generate git commit messages from git diff using Claude Code

use anyhow::{Context, Ok, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Command-line arguments
#[derive(Parser)]
struct Args {
    /// Output in JSON format (git commit will not be executed)
    #[arg(long)]
    json: bool,

    /// Path to the prompt configuration file (TOML format)
    #[arg(long)]
    config: String,
}

/// Commit message structure for JSON output
#[derive(Serialize)]
struct CommitMessage {
    message: String,
}

/// Prompt configuration file structure
#[derive(Deserialize)]
struct Config {
    /// Prompt template to send to Claude
    prompt: String,
}

/// Get git diff from the staging area
///
/// # Returns
/// * `Result<String>` - Output of git diff --cached
fn get_git_diff() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .expect("failed to get git diff");

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Load configuration from a TOML file
///
/// # Arguments
/// * `config_path` - Path to the configuration file
///
/// # Returns
/// * `Result<Config>` - Parsed configuration
///
/// # Errors
/// * File does not exist
/// * Invalid TOML format
fn load_config(config_path: &str) -> Result<Config> {
    let content = fs::read_to_string(config_path)
        .context(format!("Failed to read config file: {}", config_path))?;
    let config: Config = toml::from_str(&content).context("Failed to parse config file as TOML")?;
    Ok(config)
}

/// Generate a commit message using Claude Code
///
/// # Arguments
/// * `diff` - Git diff content
/// * `config` - Prompt configuration
///
/// # Returns
/// * `Result<String>` - Generated commit message
fn generate_message(diff: &str, config: &Config) -> Result<String> {
    let prompt = build_prompt(diff, config);

    let output = Command::new("claude")
        .args(["-p", &prompt])
        .output()
        .expect("Claude failed");

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Build a prompt by combining the prompt template and git diff
///
/// # Arguments
/// * `diff` - Git diff content
/// * `config` - Prompt configuration
///
/// # Returns
/// * `String` - Complete prompt to send to Claude
fn build_prompt(diff: &str, config: &Config) -> String {
    format!("{}\n\n{}", config.prompt, diff)
}

/// Write the commit message to .git/COMMIT_MSG_GENERATED
///
/// # Arguments
/// * `message` - Generated commit message
///
/// # Returns
/// * `Result<String>` - Path to the written file
///
/// # Errors
/// * .git directory does not exist
/// * Failed to write file
fn write_commit_message(message: &str) -> Result<String> {
    let commit_msg_path = ".git/COMMIT_MSG_GENERATED";
    fs::write(commit_msg_path, message).context(
        "Failed to write to .git/COMMIT_MSG_GENERATED. Make sure you are in a git repository.",
    )?;
    Ok(commit_msg_path.to_string())
}

/// Execute git commit -v -e -F to launch an editor
///
/// This function replaces the current process with the git command,
/// so it does not return on success.
///
/// # Arguments
/// * `msg_file` - Path to the commit message file
///
/// # Returns
/// * `Result<()>` - Only returns if an error occurs
///
/// # Note
/// Unix-like systems only (uses CommandExt::exec)
fn run_git_commit(msg_file: &str) -> Result<()> {
    let err = Command::new("git")
        .args(["commit", "-v", "-e", "-F", msg_file])
        .exec();

    // exec() does not return on success, so reaching here means an error
    Err(anyhow::anyhow!("Failed to execute git commit: {}", err))
}

/// Main entry point
///
/// # Process flow
/// 1. Parse command-line arguments
/// 2. Load configuration file
/// 3. Get git diff
/// 4. Generate commit message using Claude Code
/// 5. Output as JSON or write to .git/COMMIT_MSG_GENERATED and execute git commit
fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration file (required)
    let config = load_config(&args.config)?;

    let diff = get_git_diff()?;
    let message = generate_message(&diff, &config)?;

    if args.json {
        let output = CommitMessage { message };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        let msg_file = write_commit_message(&message)?;
        println!("Commit message has been written to {}", msg_file);
        println!("Launching git commit...\n");
        run_git_commit(&msg_file)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Tests for build_prompt()
    // =============================================================================

    #[test]
    fn test_build_prompt_basic() {
        // Arrange - setup test data
        let diff = "diff --git a/file.txt b/file.txt\n+new line";
        let config = Config {
            prompt: "Generate a commit message:".to_string(),
        };

        // Act - execute the function
        let result = build_prompt(diff, &config);

        // Assert - verify the result
        assert_eq!(
            result,
            "Generate a commit message:\n\ndiff --git a/file.txt b/file.txt\n+new line"
        );
    }

    #[test]
    fn test_build_prompt_empty_diff() {
        // Arrange - empty diff
        let diff = "";
        let config = Config {
            prompt: "Generate a commit message:".to_string(),
        };

        // Act
        let result = build_prompt(diff, &config);

        // Assert - should still include prompt with empty diff
        assert_eq!(result, "Generate a commit message:\n\n");
    }

    #[test]
    fn test_build_prompt_empty_prompt() {
        // Arrange - empty prompt
        let diff = "diff --git a/file.txt b/file.txt\n+new line";
        let config = Config {
            prompt: "".to_string(),
        };

        // Act
        let result = build_prompt(diff, &config);

        // Assert - should have two newlines before diff
        assert_eq!(result, "\n\ndiff --git a/file.txt b/file.txt\n+new line");
    }

    #[test]
    fn test_build_prompt_both_empty() {
        // Arrange - both empty
        let diff = "";
        let config = Config {
            prompt: "".to_string(),
        };

        // Act
        let result = build_prompt(diff, &config);

        // Assert - should be just two newlines
        assert_eq!(result, "\n\n");
    }

    #[test]
    fn test_build_prompt_special_characters() {
        // Arrange - special characters including newlines, Unicode, and emojis
        let diff = "diff --git a/日本語.txt b/日本語.txt\n+こんにちは 🎉\n+Special: \t\\n\"quotes\"";
        let config = Config {
            prompt: "Prompt with 絵文字 🚀 and\nmultiple\nlines".to_string(),
        };

        // Act
        let result = build_prompt(diff, &config);

        // Assert - all special characters should be preserved
        assert!(result.contains("絵文字 🚀"));
        assert!(result.contains("こんにちは 🎉"));
        assert!(result.contains("multiple\nlines"));
        assert!(result.contains("Special: \t\\n\"quotes\""));
    }

    #[test]
    fn test_build_prompt_multiline_prompt() {
        // Arrange - multiline prompt
        let diff = "+added line";
        let config = Config {
            prompt: "Line 1\nLine 2\nLine 3".to_string(),
        };

        // Act
        let result = build_prompt(diff, &config);

        // Assert - newlines in prompt should be preserved
        assert_eq!(result, "Line 1\nLine 2\nLine 3\n\n+added line");
    }

    #[test]
    fn test_build_prompt_very_long_input() {
        // Arrange - very long diff (simulate large file changes)
        let large_diff = "diff --git a/large.txt b/large.txt\n".to_string() + &"+".repeat(10000);
        let config = Config {
            prompt: "Generate commit:".to_string(),
        };

        // Act
        let result = build_prompt(&large_diff, &config);

        // Assert - should handle large inputs without panic
        assert!(result.starts_with("Generate commit:\n\ndiff --git"));
        assert!(result.len() > 10000);
        assert!(result.contains(&"+".repeat(100))); // verify content is there
    }

    // =============================================================================
    // Tests for Config deserialization
    // =============================================================================

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

    // =============================================================================
    // Tests for CommitMessage serialization
    // =============================================================================

    #[test]
    fn test_commit_message_serialize_basic() {
        // Arrange - basic commit message
        let commit = CommitMessage {
            message: "feat: add new feature".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - should serialize to valid JSON
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json, r#"{"message":"feat: add new feature"}"#);
    }

    #[test]
    fn test_commit_message_serialize_special_characters() {
        // Arrange - message with special characters
        let commit = CommitMessage {
            message: r#"fix: resolve "quote" issue and \backslash"#.to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - special characters should be properly escaped
        assert!(result.is_ok());
        let json = result.unwrap();
        // JSON should escape quotes and backslashes
        assert!(json.contains(r#"\"quote\""#));
        assert!(json.contains(r#"\\"#));
    }

    #[test]
    fn test_commit_message_serialize_empty_message() {
        // Arrange - empty message
        let commit = CommitMessage {
            message: "".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - should serialize to empty string in JSON
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json, r#"{"message":""}"#);
    }

    #[test]
    fn test_commit_message_serialize_multiline_message() {
        // Arrange - multiline commit message
        let commit = CommitMessage {
            message: "feat: add feature\n\nThis is a longer description.\nWith multiple lines.".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - newlines should be escaped as \n
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains(r#"\n"#));
        assert!(json.contains("feat: add feature"));
        assert!(json.contains("longer description"));
    }

    #[test]
    fn test_commit_message_serialize_unicode_and_emoji() {
        // Arrange - message with Unicode and emoji
        let commit = CommitMessage {
            message: "feat: 日本語サポート追加 🎉🚀".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - Unicode and emoji should be preserved
        assert!(result.is_ok());
        let json = result.unwrap();
        // serde_json preserves Unicode by default
        assert!(json.contains("日本語サポート追加"));
        assert!(json.contains("🎉"));
        assert!(json.contains("🚀"));
    }

    #[test]
    fn test_commit_message_deserialize_and_verify_structure() {
        // Arrange - serialize a message first
        let original = CommitMessage {
            message: "test: verify roundtrip".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();

        // Act - parse back to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Assert - should have correct structure
        assert!(parsed.is_object());
        assert_eq!(parsed["message"], "test: verify roundtrip");
    }
}
