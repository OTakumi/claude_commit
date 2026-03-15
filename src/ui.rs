//! User interaction: spinner display and interactive commit flow

use anyhow::Result;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, sleep};

use crate::claude::generate_message;
use crate::config::Config;
use crate::git::{run_git_commit, run_git_commit_direct, write_commit_message};

/// Run the interactive commit flow
///
/// Generates a commit message and prompts the user to:
/// - [A]ccept: commit directly without opening an editor
/// - [E]dit: open the git commit editor to review/modify before committing
/// - [R]egenerate: discard the message and generate a new one
/// - [Q]uit: cancel the commit
pub async fn interactive_commit(diff: &str, config: &Config) -> Result<()> {
    loop {
        let message = generate_with_spinner(diff, config).await?;

        println!("\nGenerated commit message:");
        println!("─────────────────────────────────────");
        println!("{}", message);
        println!("─────────────────────────────────────");

        loop {
            print!("\n[A]ccept  [E]dit  [R]egenerate  [Q]uit > ");
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().to_lowercase().as_str() {
                "a" | "accept" => {
                    let msg_file = write_commit_message(&message)?;
                    run_git_commit_direct(&msg_file)?;
                    return Ok(());
                }
                "e" | "edit" => {
                    let msg_file = write_commit_message(&message)?;
                    println!("Launching git commit editor...\n");
                    run_git_commit(&msg_file)?;
                    return Ok(());
                }
                "r" | "regenerate" => break, // break inner loop → regenerate
                "q" | "quit" => {
                    println!("Commit cancelled.");
                    std::process::exit(0);
                }
                _ => {
                    println!("Invalid input. Please enter A, E, R, or Q.");
                }
            }
        }
    }
}

/// Generate a commit message with a spinner displayed while waiting
///
/// Shows a rotating spinner while Claude AI is generating the commit message.
/// The spinner automatically stops when generation is complete.
pub async fn generate_with_spinner(diff: &str, config: &Config) -> Result<String> {
    let spinner_running = Arc::new(AtomicBool::new(true));
    let spinner_running_clone = Arc::clone(&spinner_running);

    let spinner_task = tokio::spawn(async move {
        let spinner_chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut idx = 0;

        while spinner_running_clone.load(Ordering::Relaxed) {
            print!("\r{} Claude is generating...", spinner_chars[idx]);
            let _ = io::stdout().flush();
            idx = (idx + 1) % spinner_chars.len();
            sleep(Duration::from_millis(80)).await;
        }

        print!("\r\x1b[K");
        let _ = io::stdout().flush();
    });

    let message = generate_message(diff, config).await?;

    spinner_running.store(false, Ordering::Relaxed);
    let _ = spinner_task.await;

    println!("✓ コミットメッセージの生成が完了しました");

    Ok(message)
}
