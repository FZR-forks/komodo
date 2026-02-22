#[derive(Debug, Clone, clap::Parser)]
pub struct Terminal {
  /// The terminal command to run.
  #[command(subcommand)]
  pub command: TerminalCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum TerminalCommand {
  /// Execute a command against a host terminal.
  #[clap(alias = "h")]
  Host(HostTerminal),
  /// Execute a command in a container shell.
  #[clap(alias = "c")]
  Container(ContainerTerminal),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct HostTerminal {
  /// Server id or name.
  pub server: String,
  /// Terminal name on the server.
  #[arg(long, short = 't', default_value = "cli")]
  pub terminal: String,
  /// Command to execute on the host.
  ///
  /// Supports multiple arguments without quoting:
  /// `km terminal host server-1 --command apt update`
  #[arg(long, short = 'x', num_args = 1..)]
  pub command: Vec<String>,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct ContainerTerminal {
  /// Server id or name.
  pub server: String,
  /// Container name.
  pub container: String,
  /// Command to execute in the container.
  ///
  /// Supports multiple arguments without quoting:
  /// `km terminal container server-1 my-container --command apt update`
  #[arg(long, short = 'x', num_args = 1..)]
  pub command: Vec<String>,
  /// Shell to use inside the container.
  #[arg(long, short = 's', default_value = "sh")]
  pub shell: String,
}
