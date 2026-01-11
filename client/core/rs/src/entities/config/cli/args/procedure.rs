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

  /// View logs for the procedure's runs (aliases: `log`, `l`)
  #[clap(alias = "log", alias = "l")]
  Logs {
    /// Number of log entries to show. Default: 10.
    #[arg(long, short = 'n', default_value = "10")]
    limit: u32,
  },
}
