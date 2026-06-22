//! CLI arguments for variable operations

#[derive(Debug, Clone, clap::Parser)]
pub struct Variable {
  /// The subcommand to run
  #[command(subcommand)]
  pub command: VariableCommand,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum VariableCommand {
  /// List all variables (aliases: `ls`, `l`)
  #[clap(alias = "ls", alias = "l")]
  List {
    /// Specify the format of the output.
    #[arg(long, short = 'f', default_value_t = super::CliFormat::Table)]
    format: super::CliFormat,
  },

  /// Get a specific variable's value (alias: `g`)
  #[clap(alias = "g")]
  Get {
    /// The name of the variable to get
    name: String,
  },

  /// Create a new variable (aliases: `c`, `add`)
  #[clap(alias = "c", alias = "add")]
  Create {
    /// The name of the variable
    name: String,

    /// The value of the variable. Required unless --from-command is used.
    #[arg(required_unless_present = "from_command")]
    value: Option<String>,

    /// Get the value by running a shell command and capturing its stdout.
    /// Example: --from-command "openssl rand -hex 32"
    #[arg(
      long = "from-command",
      short = 'c',
      conflicts_with = "value"
    )]
    from_command: Option<String>,

    /// Whether this variable should be a secret
    #[arg(long, short = 's', default_value_t = false)]
    secret: bool,

    /// Optional description for the variable
    #[arg(long, short = 'd', default_value = "")]
    description: String,

    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },

  /// Delete a variable (aliases: `d`, `rm`, `remove`)
  #[clap(alias = "d", alias = "rm", alias = "remove")]
  Delete {
    /// The name of the variable to delete
    name: String,

    /// Always continue on user confirmation prompts.
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },
}
