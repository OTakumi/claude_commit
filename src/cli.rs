//! CLI argument definitions and subcommand implementations

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::DEFAULT_CONFIG_CONTENT;
use crate::git::get_git_root;

/// Command-line arguments
#[derive(Parser)]
#[command(name = "claude_commit")]
#[command(about = "Generate git commit messages using Claude AI", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output in JSON format (git commit will not be executed)
    #[arg(long)]
    pub json: bool,

    /// Path to the prompt configuration file (TOML format).
    /// If omitted, searches: ~/.config/claude_commit/config.toml → <git root>/.claude_commit.toml → ./.claude_commit.toml
    #[arg(long)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a default configuration file
    Init {
        /// Output path for the config file (default: ~/.config/claude_commit/config.toml)
        #[arg(long)]
        output: Option<String>,

        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
    },
}

/// Create a default configuration file at the specified path
///
/// When `output_path` is `None`, defaults to `~/.config/claude_commit/config.toml`.
/// Parent directories are created automatically if they do not exist.
/// Refuses to overwrite an existing file unless `force` is true.
pub fn run_init(output_path: Option<&str>, force: bool) -> Result<()> {
    let path = match output_path {
        Some(p) => PathBuf::from(p),
        None => {
            let home = std::env::var("HOME").map_err(|_| {
                anyhow::anyhow!("$HOME is not set. Use --output to specify a path.")
            })?;
            PathBuf::from(home)
                .join(".config")
                .join("claude_commit")
                .join("config.toml")
        }
    };

    if path.exists() && !force {
        eprintln!("Error: '{}' already exists.", path.display());
        eprintln!("Use --force to overwrite.");
        std::process::exit(1);
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            anyhow::anyhow!("Failed to create directory '{}': {}", parent.display(), e)
        })?;
    }

    std::fs::write(&path, DEFAULT_CONFIG_CONTENT)
        .map_err(|e| anyhow::anyhow!("Failed to write config file '{}': {}", path.display(), e))?;

    println!("Created config file: {}", path.display());
    println!("Edit the 'prompt' field to customize the commit message style.");
    Ok(())
}

/// Find a config file by searching in standard locations
///
/// Search order:
/// 1. `~/.config/claude_commit/config.toml` (recommended)
/// 2. `<git root>/.claude_commit.toml`
/// 3. `./.claude_commit.toml`
pub fn find_config_file() -> Option<PathBuf> {
    // 1. ~/.config/claude_commit/config.toml (recommended)
    if let Ok(home) = std::env::var("HOME") {
        let home_config = PathBuf::from(home)
            .join(".config")
            .join("claude_commit")
            .join("config.toml");
        if home_config.exists() {
            return Some(home_config);
        }
    }

    // 2. Git repository root
    if let Ok(root) = get_git_root() {
        let git_root_config = root.join(".claude_commit.toml");
        if git_root_config.exists() {
            return Some(git_root_config);
        }
    }

    // 3. Current directory
    let local = PathBuf::from(".claude_commit.toml");
    if local.exists() {
        return Some(local);
    }

    None
}
