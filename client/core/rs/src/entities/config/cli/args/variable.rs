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

    /// Confirm you want to view a secret variable's value without warning
    #[arg(long, short = 'y', default_value_t = false)]
    yes: bool,
  },

  /// Create a new variable (aliases: `c`, `add`)
  #[clap(alias = "c", alias = "add")]
  Create {
    /// The name of the variable
    name: String,

    /// The value of the variable
    value: String,

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
