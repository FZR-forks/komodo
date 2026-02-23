#[derive(Debug, Clone, clap::Parser)]
pub struct Shell {
  /// The shell command target.
  #[command(subcommand)]
  pub command: ShellCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ShellCommand {
  /// Execute a command on a host.
  #[clap(alias = "h")]
  Host(HostShell),
  /// Execute a command in a container.
  #[clap(alias = "c")]
  Container(ContainerShell),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct HostShell {
  /// Server id or name.
  pub server: String,
  /// Command to execute.
  #[arg(long, short = 'x', required = true)]
  pub command: String,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct ContainerShell {
  /// Server id or name.
  pub server: String,
  /// Container name.
  pub container: String,
  /// Command to execute.
  #[arg(long, short = 'x', required = true)]
  pub command: String,
}
