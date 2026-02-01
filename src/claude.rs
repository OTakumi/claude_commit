//! Claude AI integration for commit message generation
//!
//! This module handles communication with Claude AI to generate
//! commit messages based on git diffs and prompt templates.

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::config::Config;
use crate::validation::validate_prompt_size;

/// Generate a commit message using Claude Code
///
/// # Arguments
///
/// * `diff` - Git diff content from staged changes
/// * `config` - Prompt configuration with template
///
/// # Returns
///
/// * `Result<String>` - Generated commit message from Claude
///
/// # Errors
///
/// * Prompt size exceeds 1MB (combined diff + prompt template)
/// * Claude command execution fails
/// * Claude command returns non-zero exit code
/// * Unable to parse Claude output
///
/// # Example
///
/// ```no_run
/// use claude_commit::{claude::generate_message, config::Config};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let config = Config {
///     prompt: "Generate a commit message:".to_string(),
/// };
/// let diff = "diff --git a/file.txt b/file.txt\n+new line";
/// let message = generate_message(diff, &config).await?;
/// println!("Message: {}", message);
/// # Ok(())
/// # }
/// ```
pub async fn generate_message(diff: &str, config: &Config) -> Result<String> {
    // Validate prompt size BEFORE allocation to prevent excessive memory usage
    validate_prompt_size(&config.prompt, diff)?;

    let prompt = build_prompt(diff, config);

    let output = Command::new("claude")
        .args(["-p", &prompt])
        .output()
        .await
        .context("Failed to execute 'claude' command. Make sure Claude CLI is installed and in PATH")?;

    if !output.status.success() {
        anyhow::bail!(
            "Claude command failed with exit code {:?}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string())
}

/// Build a prompt by combining the prompt template and git diff
///
/// The final prompt structure is:
/// ```text
/// {prompt_template}
///
/// {git_diff}
/// ```
///
/// # Arguments
///
/// * `diff` - Git diff content
/// * `config` - Prompt configuration
///
/// # Returns
///
/// * `String` - Complete prompt to send to Claude
///
/// # Example
///
/// ```
/// use claude_commit::{claude::build_prompt, config::Config};
///
/// let config = Config {
///     prompt: "Generate a commit message:".to_string(),
/// };
/// let diff = "+added line";
/// let prompt = build_prompt(diff, &config);
/// assert_eq!(prompt, "Generate a commit message:\n\n+added line");
/// ```
pub fn build_prompt(diff: &str, config: &Config) -> String {
    format!("{}\n\n{}", config.prompt, diff)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let diff =
            "diff --git a/日本語.txt b/日本語.txt\n+こんにちは 🎉\n+Special: \t\\n\"quotes\"";
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
}
