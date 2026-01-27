//! CLI tool to automatically generate git commit messages using Claude AI
//!
//! This tool analyzes staged git changes and uses Claude to generate
//! appropriate commit messages in conventional commits format.

use anyhow::Result;
use clap::Parser;

use claude_commit::{
    claude::generate_message,
    config::load_config,
    git::{get_git_diff, run_git_commit, write_commit_message},
    output::CommitMessage,
};

/// Command-line arguments
#[derive(Parser)]
#[command(name = "claude_commit")]
#[command(about = "Generate git commit messages using Claude AI", long_about = None)]
struct Args {
    /// Output in JSON format (git commit will not be executed)
    #[arg(long)]
    json: bool,

    /// Path to the prompt configuration file (TOML format)
    #[arg(long)]
    config: String,
}

/// Main entry point
///
/// # Process flow
///
/// 1. Parse command-line arguments
/// 2. Load configuration file
/// 3. Get git diff from staging area
/// 4. Generate commit message using Claude
/// 5. Output as JSON or write to .git/COMMIT_MSG_GENERATED and execute git commit
///
/// # Errors
///
/// * Configuration file not found or invalid
/// * Not in a git repository
/// * No staged changes
/// * Claude command fails
/// * Git commit fails
fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration file (required)
    let config = load_config(&args.config)?;

    // Get staged changes
    let diff = get_git_diff()?;

    // Generate commit message using Claude
    let message = generate_message(&diff, &config)?;

    // Output based on mode
    if args.json {
        // JSON mode: print message and exit
        let output = CommitMessage { message };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        // Interactive mode: write message and launch git commit editor
        let msg_file = write_commit_message(&message)?;
        println!("Commit message has been written to {}", msg_file);
        println!("Launching git commit...\n");
        run_git_commit(&msg_file)?;
    }

    Ok(())
}
