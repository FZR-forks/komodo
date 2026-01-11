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

  /// View logs for the sync's runs (aliases: `log`, `l`)
  #[clap(alias = "log", alias = "l")]
  Logs {
    /// Number of log entries to show. Default: 10.
    #[arg(long, short = 'n', default_value = "10")]
    limit: u32,
  },

  /// Show pending diffs for the sync (alias: `d`)
  #[clap(alias = "d")]
  Diff,
}
