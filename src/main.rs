//! jj-starship - Unified Git/JJ Starship prompt module

#[cfg(not(any(feature = "git", feature = "jj")))]
compile_error!("At least one of 'git' or 'jj' features must be enabled");

mod color;
mod config;
mod detect;
mod error;
#[cfg(feature = "git")]
mod git;
#[cfg(feature = "jj")]
mod jj;
mod output;

use clap::{Parser, Subcommand};
use config::{Config, DisplayFlags};
use detect::RepoType;
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "jj-starship")]
#[command(about = "Unified Git/JJ Starship prompt module")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Override working directory
    #[arg(long, global = true)]
    cwd: Option<PathBuf>,

    /// Max length for branch/bookmark name (0 = unlimited)
    #[arg(long, global = true)]
    truncate_name: Option<usize>,

    /// Length of `change_id/commit` hash to display (default: 8)
    #[arg(long, global = true)]
    id_length: Option<usize>,

    /// Symbol prefix for JJ repos (default: "󰶛 ")
    #[cfg(feature = "jj")]
    #[arg(long, global = true)]
    jj_symbol: Option<String>,

    /// Symbol prefix for Git repos (default: " ")
    #[cfg(feature = "git")]
    #[arg(long, global = true)]
    git_symbol: Option<String>,

    /// Disable symbol prefix entirely
    #[arg(long, global = true)]
    no_symbol: bool,

    // JJ display flags
    /// Hide "on {symbol}" prefix for JJ repos
    #[cfg(feature = "jj")]
    #[arg(long, global = true)]
    no_jj_prefix: bool,
    /// Hide bookmark name for JJ repos
    #[cfg(feature = "jj")]
    #[arg(long, global = true)]
    no_jj_name: bool,
    /// Hide `change_id` for JJ repos
    #[cfg(feature = "jj")]
    #[arg(long, global = true)]
    no_jj_id: bool,
    /// Hide [status] for JJ repos
    #[cfg(feature = "jj")]
    #[arg(long, global = true)]
    no_jj_status: bool,

    // Git display flags
    /// Hide "on {symbol}" prefix for Git repos
    #[cfg(feature = "git")]
    #[arg(long, global = true)]
    no_git_prefix: bool,
    /// Hide branch name for Git repos
    #[cfg(feature = "git")]
    #[arg(long, global = true)]
    no_git_name: bool,
    /// Hide (commit) for Git repos
    #[cfg(feature = "git")]
    #[arg(long, global = true)]
    no_git_id: bool,
    /// Hide [status] for Git repos
    #[cfg(feature = "git")]
    #[arg(long, global = true)]
    no_git_status: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Output prompt string (default)
    Prompt,
    /// Exit 0 if in repo, 1 otherwise (for starship "when" condition)
    Detect,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let Some(cwd) = cli.cwd.or_else(|| env::current_dir().ok()) else {
        return ExitCode::FAILURE;
    };
    #[cfg(feature = "jj")]
    let jj_symbol = cli.jj_symbol;
    #[cfg(not(feature = "jj"))]
    let jj_symbol: Option<String> = None;

    #[cfg(feature = "git")]
    let git_symbol = cli.git_symbol;
    #[cfg(not(feature = "git"))]
    let git_symbol: Option<String> = None;

    #[cfg(feature = "jj")]
    let jj_flags = DisplayFlags {
        no_prefix: cli.no_jj_prefix,
        no_name: cli.no_jj_name,
        no_id: cli.no_jj_id,
        no_status: cli.no_jj_status,
    };
    #[cfg(not(feature = "jj"))]
    let jj_flags = DisplayFlags::default();

    #[cfg(feature = "git")]
    let git_flags = DisplayFlags {
        no_prefix: cli.no_git_prefix,
        no_name: cli.no_git_name,
        no_id: cli.no_git_id,
        no_status: cli.no_git_status,
    };
    #[cfg(not(feature = "git"))]
    let git_flags = DisplayFlags::default();

    let config = Config::new(
        cli.truncate_name,
        cli.id_length,
        jj_symbol,
        git_symbol,
        cli.no_symbol,
        jj_flags,
        git_flags,
    );

    match cli.command.unwrap_or(Command::Prompt) {
        Command::Prompt => {
            if let Some(output) = run_prompt(&cwd, &config) {
                print!("{output}");
            }
            ExitCode::SUCCESS
        }
        Command::Detect => {
            if detect::in_repo(&cwd) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
    }
}

/// Run prompt generation, returning None on error (silent fail for prompts)
#[allow(unreachable_patterns)]
fn run_prompt(cwd: &Path, config: &Config) -> Option<String> {
    let result = detect::detect(cwd);

    match result.repo_type {
        #[cfg(feature = "jj")]
        RepoType::Jj | RepoType::JjColocated => {
            let repo_root = result.repo_root?;
            let info = jj::collect(&repo_root, config.id_length).ok()?;
            Some(output::format_jj(&info, config))
        }
        // Colocated repos fall back to git when jj feature disabled
        #[cfg(all(feature = "git", not(feature = "jj")))]
        RepoType::JjColocated => {
            let repo_root = result.repo_root?;
            let info = git::collect(&repo_root, config.id_length).ok()?;
            Some(output::format_git(&info, config))
        }
        #[cfg(feature = "git")]
        RepoType::Git => {
            let repo_root = result.repo_root?;
            let info = git::collect(&repo_root, config.id_length).ok()?;
            Some(output::format_git(&info, config))
        }
        RepoType::None => None,
        // Catch disabled variants
        _ => None,
    }
}
