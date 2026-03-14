use clap::{ArgAction, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "glint",
    version,
    about = "Analyze and clean PrismLauncher disk usage"
)]
pub struct Cli {
    /// PrismLauncher root path. Uses OS default when omitted.
    #[arg(long, global = true)]
    pub path: Option<PathBuf>,

    /// Emit JSON output
    #[arg(long, global = true, action = ArgAction::SetTrue)]
    pub json: bool,

    /// Enable verbose output
    #[arg(long, short = 'v', global = true, action = ArgAction::SetTrue)]
    pub verbose: bool,

    /// Output language
    #[arg(long, global = true, value_enum, default_value_t = Language::En)]
    pub lang: Language,

    /// Log level
    #[arg(long, global = true, value_enum, default_value_t = LogLevel::Warn)]
    pub log_level: LogLevel,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, ValueEnum)]
pub enum Language {
    En,
    Ja,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl LogLevel {
    pub fn as_filter(self) -> log::LevelFilter {
        match self {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Analyze reclaimable storage
    Scan,

    /// Clean targets (dry-run by default)
    Clean {
        /// Explicitly force dry-run mode
        #[arg(long, action = ArgAction::SetTrue)]
        dry_run: bool,

        /// Actually delete files (move to trash)
        #[arg(long, action = ArgAction::SetTrue)]
        apply: bool,

        /// Skip confirmation prompt
        #[arg(long, short = 'y', action = ArgAction::SetTrue)]
        yes: bool,

        /// Include detected unused libraries as clean candidates
        #[arg(long, action = ArgAction::SetTrue)]
        include_unused_libraries: bool,

        /// Include detected unused assets as clean candidates
        #[arg(long, action = ArgAction::SetTrue)]
        include_unused_assets: bool,
    },

    /// Detect duplicate mods across instances
    Mods,

    /// Analyze world sizes
    Worlds,

    /// Show per-instance usage
    Usage,
}

#[derive(Debug, Clone, Copy)]
pub struct CleanMode {
    pub dry_run: bool,
    pub yes: bool,
    pub include_unused_libraries: bool,
    pub include_unused_assets: bool,
}
