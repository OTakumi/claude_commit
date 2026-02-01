//! Claude AI integration for commit message generation
//!
//! This module handles communication with Claude AI to generate
//! commit messages based on git diffs and prompt templates.

use anyhow::{Context, Result};
use tokio::process::Command;

use crate::config::Config;
use crate::prompt::build_prompt;

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
    let prompt = build_prompt(diff, &config.prompt)?;

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

