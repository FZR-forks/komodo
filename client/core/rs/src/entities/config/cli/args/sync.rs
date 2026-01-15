//! CLI arguments for resource sync operations

#[derive(Debug, Clone, clap::Parser)]
pub struct Sync {
  /// The sync name or id
  pub sync: String,

  /// The subcommand to run
  #[command(subcommand)]
  pub command: SyncCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SyncCommand {
  /// Get status of the sync (aliases: `stat`, `s`)
  #[clap(alias = "stat", alias = "s")]
  Status,

  /// View run history for the sync (aliases: `log`, `l`)
  /// Shows the most recent runs first. Use `run-log <ID>` to see detailed logs for a specific run.
  #[clap(alias = "log", alias = "l")]
  Logs {
    /// Number of most recent runs to show. Default: 10.
    #[arg(long, short = 'n', default_value = "10")]
    limit: u32,
  },

  /// View detailed logs for a specific sync run (aliases: `rl`)
  #[clap(alias = "rl")]
  RunLog {
    /// The update/run ID from the logs output
    id: String,
  },

  /// Show pending diffs for the sync (alias: `d`)
  #[clap(alias = "d")]
  Diff,
}
