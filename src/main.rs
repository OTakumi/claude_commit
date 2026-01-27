//! CLI tool to automatically generate git commit messages using Claude AI
//!
//! This tool analyzes staged git changes and uses Claude to generate
//! appropriate commit messages in conventional commits format.

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, sleep};

use claude_commit::{
    claude::generate_message,
    config::{Config, load_config},
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
/// 4. Generate commit message using Claude (with spinner display)
/// 5. Output as JSON or write to .git/COMMIT_MSG_GENERATED and execute git commit
///
/// # Errors
///
/// * Configuration file not found or invalid
/// * Not in a git repository
/// * No staged changes
/// * Claude command fails
/// * Git commit fails
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration file (required)
    let config = load_config(&args.config)?;

    // Get staged changes
    let diff = get_git_diff()?;

    // Generate commit message using Claude with spinner display
    let message = if args.json {
        // JSON mode: no spinner
        generate_message(&diff, &config).await?
    } else {
        // Interactive mode: show spinner while generating
        generate_with_spinner(&diff, &config).await?
    };

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

/// Generate commit message with spinner display
///
/// Shows a rotating spinner while Claude AI is generating the commit message.
/// The spinner automatically stops when generation is complete.
///
/// # Arguments
///
/// * `diff` - Git diff content
/// * `config` - Prompt configuration
///
/// # Returns
///
/// * `Result<String>` - Generated commit message
async fn generate_with_spinner(diff: &str, config: &Config) -> Result<String> {
    let spinner_running = Arc::new(AtomicBool::new(true));
    let spinner_running_clone = Arc::clone(&spinner_running);

    // Spawn spinner task
    let spinner_task = tokio::spawn(async move {
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut idx = 0;

        while spinner_running_clone.load(Ordering::Relaxed) {
            print!("\r{} Claude is generating...", spinner_chars[idx]);
            io::stdout().flush().unwrap();
            idx = (idx + 1) % spinner_chars.len();
            sleep(Duration::from_millis(80)).await;
        }

        // Clear spinner line
        print!("\r\x1b[K");
        io::stdout().flush().unwrap();
    });

    // Generate message
    let message = generate_message(diff, config).await?;

    // Stop spinner
    spinner_running.store(false, Ordering::Relaxed);
    spinner_task.await.unwrap();

    println!("✓ コミットメッセージの生成が完了しました");

    Ok(message)
}
