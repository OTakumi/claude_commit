//! Claude Commit - Automatic Git Commit Message Generator
//!
//! This library provides functionality to automatically generate git commit messages
//! using Claude AI by analyzing git diffs.
//!
//! # Modules
//!
//! - [`config`] - Configuration file loading and parsing
//! - [`output`] - Output structures for JSON formatting
//! - [`claude`] - Claude AI integration for message generation
//! - [`git`] - Git operations (diff, commit, etc.)
//! - [`validation`] - Input validation for size limits
//!
//! # Example
//!
//! ```no_run
//! use claude_commit::{config::load_config, git::get_git_diff, claude::generate_message};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let config = load_config("prompt.toml")?;
//! let diff = get_git_diff()?;
//! let message = generate_message(&diff, &config).await?;
//! println!("Generated message: {}", message);
//! # Ok(())
//! # }
//! ```

pub mod claude;
pub mod config;
pub mod git;
pub mod output;
pub mod validation;
