//! CLI arguments for stack operations

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
  /// View logs for a stack (aliases: `log`, `l`)
  #[clap(alias = "log", alias = "l")]
  Logs(StackLogs),

  /// List services of a stack (aliases: `service`, `svc`, `sv`)
  #[clap(alias = "service", alias = "svc", alias = "sv")]
  Services,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct StackLogs {
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
}
