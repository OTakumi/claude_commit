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
