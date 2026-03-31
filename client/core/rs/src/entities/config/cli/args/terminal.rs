#[derive(Debug, Clone, clap::Parser)]
pub struct Ssh {
  /// The terminal target to connect to.
  #[command(subcommand)]
  pub target: SshTarget,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SshTarget {
  /// Open an interactive shell on a host terminal.
  #[clap(alias = "h")]
  Host(SshHost),
  /// Open an interactive shell in a container terminal.
  #[clap(alias = "c")]
  Container(SshContainer),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct SshHost {
  /// Server id or name.
  pub server: String,
  /// Terminal name on the host.
  #[arg(long, short = 't', default_value = "ssh")]
  pub terminal: String,
  /// Custom shell command to use to start the session, eg `bash`.
  /// Defaults to the Periphery default.
  #[arg(long, short = 's')]
  pub shell: Option<String>,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct SshContainer {
  /// The container (name) to connect to.
  /// Will error if matches multiple containers but no server is defined.
  pub container: String,
  /// Specify server.
  /// Required if multiple servers have the same container name.
  #[arg(long)]
  pub server: Option<String>,
  /// Terminal name on the container target.
  #[arg(long, short = 't', default_value = "ssh")]
  pub terminal: String,
  /// Shell to use inside the container.
  #[arg(long, short = 's', default_value = "sh")]
  pub shell: String,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Exec {
  /// The execution target.
  #[command(subcommand)]
  pub target: ExecTarget,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum ExecTarget {
  /// Execute a one-off command against a host terminal.
  #[clap(alias = "h")]
  Host(ExecHost),
  /// Execute a one-off command in a container shell.
  #[clap(alias = "c")]
  Container(ExecContainer),
}

#[derive(Debug, Clone, clap::Parser)]
pub struct ExecHost {
  /// Server id or name.
  pub server: String,
  /// The remote command string to execute.
  #[arg(long, short = 'x')]
  pub command: String,
  /// Terminal name on the host.
  #[arg(long, short = 't', default_value = "cli")]
  pub terminal: String,
  /// Shell command used when initializing the terminal.
  #[arg(long, short = 's')]
  pub shell: Option<String>,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct ExecContainer {
  /// The container (name) to execute within.
  /// Will error if matches multiple containers but no server is defined.
  pub container: String,
  /// The remote command string to execute.
  #[arg(long, short = 'x')]
  pub command: String,
  /// Specify server.
  /// Required if multiple servers have the same container name.
  #[arg(long)]
  pub server: Option<String>,
  /// Terminal name on the container target.
  #[arg(long, short = 't', default_value = "cli")]
  pub terminal: String,
  /// Shell to use inside the container.
  #[arg(long, short = 's', default_value = "sh")]
  pub shell: String,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct Attach {
  /// The container (name) to attach to.
  /// Will error if matches multiple containers but no server is defined.
  pub container: String,
  /// Specify server.
  /// Required if multiple servers have the same container name.
  #[arg(long)]
  pub server: Option<String>,
  /// Terminal name on the container target.
  #[arg(long, short = 't', default_value = "attach")]
  pub terminal: String,
  /// Force fresh terminal to replace existing one.
  #[arg(long, short = 'r', default_value_t = false)]
  pub recreate: bool,
}
