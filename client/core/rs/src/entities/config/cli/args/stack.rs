//! CLI arguments for stack operations

use super::CliFormat;

#[derive(Debug, Clone, clap::Parser)]
pub struct Stack {
  /// The stack name or id
  pub stack: String,

  /// The subcommand to run
  #[command(subcommand)]
  pub command: StackCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum StackCommand {
  /// Get status of the stack (aliases: `stat`, `s`)
  #[clap(alias = "stat", alias = "s")]
  Status {
    /// Specify the output format.
    #[arg(long, short = 'f', default_value_t = CliFormat::Table)]
    format: CliFormat,
  },

  /// View container logs for the stack (aliases: `log`, `l`)
  #[clap(alias = "log", alias = "l")]
  Logs(StackLogsOptions),

  /// List services of the stack (aliases: `service`, `svc`, `sv`)
  #[clap(alias = "service", alias = "svc", alias = "sv")]
  Services {
    /// Specify the output format.
    #[arg(long, short = 'f', default_value_t = CliFormat::Table)]
    format: CliFormat,
  },

  /// View deployment history for the stack (aliases: `dep`, `deployments`)
  #[clap(alias = "dep", alias = "deployments")]
  Deploys {
    /// Number of entries to show. Default: 10.
    #[arg(long, short = 'n', default_value = "10")]
    limit: u32,

    /// Specify the output format.
    #[arg(long, short = 'f', default_value_t = CliFormat::Table)]
    format: CliFormat,
  },

  /// View logs for a specific deployment (alias: `dl`)
  #[clap(alias = "dl")]
  DeployLog {
    /// The update/deployment ID
    id: String,

    /// Specify the output format.
    #[arg(long, short = 'f', default_value_t = CliFormat::Table)]
    format: CliFormat,
  },
}

#[derive(Debug, Clone, clap::Parser)]
pub struct StackLogsOptions {
  /// Filter logs to specific services.
  /// Can be specified multiple times.
  #[arg(name = "service", long, short = 's')]
  pub services: Vec<String>,

  /// Number of lines to show from the log tail. Default: 100, Max: 5000.
  #[arg(long, short = 'n', default_value = "100")]
  pub tail: u64,

  /// Show timestamps in the logs.
  #[arg(long, short = 't', default_value_t = false)]
  pub timestamps: bool,

  /// Specify the output format.
  #[arg(long, short = 'f', default_value_t = CliFormat::Table)]
  pub format: CliFormat,
}
