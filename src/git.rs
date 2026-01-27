//! Git operations for commit message generation
//!
//! This module provides functions to interact with git:
//! - Get staged diffs
//! - Write commit messages
//! - Execute git commit with editor

use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

/// Get git diff from the staging area
///
/// Executes `git diff --cached` to retrieve all staged changes.
///
/// # Returns
///
/// * `Result<String>` - Output of git diff --cached
///
/// # Errors
///
/// * Git command fails to execute
/// * Not in a git repository
///
/// # Example
///
/// ```no_run
/// use claude_commit::git::get_git_diff;
///
/// # fn main() -> anyhow::Result<()> {
/// let diff = get_git_diff()?;
/// println!("Staged changes:\n{}", diff);
/// # Ok(())
/// # }
/// ```
pub fn get_git_diff() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .context("Failed to execute git command. Make sure git is installed and in PATH")?;

    if !output.status.success() {
        anyhow::bail!(
            "Git diff command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string())
}

/// Write the commit message to .git/COMMIT_MSG_GENERATED
///
/// This creates a temporary file in the git directory that will be
/// used as the default message when launching the git commit editor.
///
/// # Arguments
///
/// * `message` - Generated commit message content
///
/// # Returns
///
/// * `Result<String>` - Path to the written file
///
/// # Errors
///
/// * .git directory does not exist (not a git repository)
/// * Failed to write file (permission issues)
///
/// # Example
///
/// ```no_run
/// use claude_commit::git::write_commit_message;
///
/// # fn main() -> anyhow::Result<()> {
/// let message = "feat: add new feature\n\nDetailed description here.";
/// let path = write_commit_message(message)?;
/// println!("Message written to: {}", path);
/// # Ok(())
/// # }
/// ```
pub fn write_commit_message(message: &str) -> Result<String> {
    let commit_msg_path = ".git/COMMIT_MSG_GENERATED";
    fs::write(commit_msg_path, message).context(
        "Failed to write to .git/COMMIT_MSG_GENERATED. Make sure you are in a git repository.",
    )?;
    Ok(commit_msg_path.to_string())
}

/// Execute git commit -v -e -F to launch an editor
///
/// This function executes the git commit command with the generated message,
/// allowing the user to review and edit it in their configured editor.
///
/// # Arguments
///
/// * `msg_file` - Path to the commit message file
///
/// # Returns
///
/// * `Result<()>` - Ok if commit succeeds, Err otherwise
///
/// # Errors
///
/// * Failed to execute git command
/// * Git not found in PATH
/// * User aborted the commit
/// * Commit validation failed
///
/// # Example
///
/// ```no_run
/// use claude_commit::git::run_git_commit;
///
/// # fn main() -> anyhow::Result<()> {
/// let msg_file = ".git/COMMIT_MSG_GENERATED";
/// run_git_commit(msg_file)?;
/// println!("Commit successful!");
/// # Ok(())
/// # }
/// ```
pub fn run_git_commit(msg_file: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["commit", "-v", "-e", "-F", msg_file])
        .status()
        .context("Failed to execute git commit command")?;

    if !status.success() {
        anyhow::bail!(
            "Git commit command failed with exit code: {:?}",
            status.code()
        );
    }

    Ok(())
}

// Note: No tests for this module as all functions depend on external git commands
// and system state. Integration tests would be more appropriate for testing these.
