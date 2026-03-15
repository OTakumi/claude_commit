//! CLI tool to automatically generate git commit messages using Claude AI
//!
//! This tool analyzes staged git changes and uses Claude to generate
//! appropriate commit messages in conventional commits format.

use anyhow::Result;
use clap::Parser;

use claude_commit::{
    cli::{Args, Commands, find_config_file, run_init},
    claude::generate_message,
    config::load_config,
    git::{get_git_diff, run_pre_commit_hook},
    output::CommitMessage,
    ui::interactive_commit,
};

/// Main entry point
///
/// # Process flow
///
/// 1. Parse command-line arguments
/// 2. Resolve configuration file (explicit path or auto-search)
/// 3. Get git diff from staging area
/// 4. Run pre-commit hook (skip if not present)
/// 5. Re-fetch git diff (reflect formatter auto-fixes)
/// 6. JSON mode: generate message and print, then exit
///    Interactive mode: generate with spinner → [A]ccept / [E]dit / [R]egenerate / [Q]uit
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Handle subcommands
    if let Some(Commands::Init { output, force }) = args.command {
        return run_init(output.as_deref(), force);
    }

    // Resolve config file path
    let config_path = match args.config {
        Some(path) => path,
        None => match find_config_file() {
            Some(path) => path.to_string_lossy().to_string(),
            None => {
                eprintln!("Error: No configuration file found.");
                eprintln!("Searched locations:");
                eprintln!("  ~/.config/claude_commit/config.toml");
                eprintln!("  <git root>/.claude_commit.toml");
                eprintln!("  ./.claude_commit.toml");
                eprintln!();
                eprintln!("Run 'claude_commit init' to create a config file.");
                std::process::exit(1);
            }
        },
    };

    let config = load_config(&config_path)?;

    // Get staged changes
    let diff = get_git_diff()?;
    if diff.trim().is_empty() {
        eprintln!("Error: No staged changes found.");
        eprintln!("Please stage your changes with 'git add' before generating a commit message.");
        std::process::exit(1);
    }

    // Run pre-commit hook before calling Claude API
    run_pre_commit_hook()?;

    // Re-fetch diff to reflect any auto-fixes by formatters
    let diff = get_git_diff()?;
    if diff.trim().is_empty() {
        eprintln!("Error: No staged changes remain after pre-commit hook.");
        eprintln!("The pre-commit hook may have unstaged all changes.");
        std::process::exit(1);
    }

    if args.json {
        let message = generate_message(&diff, &config).await?;
        let output = CommitMessage { message };
        println!("{}", serde_json::to_string(&output)?);
    } else {
        interactive_commit(&diff, &config).await?;
    }

    Ok(())
}
