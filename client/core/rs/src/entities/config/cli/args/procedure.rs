//! CLI arguments for procedure operations

#[derive(Debug, Clone, clap::Parser)]
pub struct Procedure {
  /// The procedure name or id
  pub procedure: String,

  /// The subcommand to run
  #[command(subcommand)]
  pub command: ProcedureCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ProcedureCommand {
  /// Get status of the procedure (aliases: `stat`, `s`)
  #[clap(alias = "stat", alias = "s")]
  Status,

  /// View run history for the procedure (aliases: `log`, `l`)
  /// Shows the most recent runs first. Use `run-log <ID>` to see detailed logs for a specific run.
  #[clap(alias = "log", alias = "l")]
  Logs {
    /// Number of most recent runs to show. Default: 10.
    #[arg(long, short = 'n', default_value = "10")]
    limit: u32,
  },

  /// View detailed logs for a specific procedure run (aliases: `rl`)
  #[clap(alias = "rl")]
  RunLog {
    /// The update/run ID from the logs output
    id: String,
  },
}
