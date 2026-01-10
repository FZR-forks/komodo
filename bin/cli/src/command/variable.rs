use std::io::Read;

use anyhow::Context;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color, Table};
use komodo_client::{
  api::{
    read::{GetVariable, ListVariables},
    write::{CreateVariable, DeleteVariable},
  },
  entities::config::cli::{
    CliTableBorders,
    args::{
      CliFormat,
      variable::{Variable, VariableCommand},
    },
  },
};

use crate::config::cli_config;

pub async fn handle(variable: &Variable) -> anyhow::Result<()> {
  match &variable.command {
    VariableCommand::List { format } => list_variables(*format).await,
    VariableCommand::Get { name, yes } => {
      get_variable(name, *yes).await
    }
    VariableCommand::Create {
      name,
      value,
      secret,
      description,
      yes,
    } => {
      create_variable(name, value, *secret, description, *yes).await
    }
    VariableCommand::Delete { name, yes } => {
      delete_variable(name, *yes).await
    }
  }
}

async fn list_variables(format: CliFormat) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;

  let variables = client
    .read(ListVariables {})
    .await
    .context("Failed to list variables")?;

  if variables.is_empty() {
    println!("{}: No variables found", "INFO".green());
    return Ok(());
  }

  match format {
    CliFormat::Table => {
      let preset = {
        use comfy_table::presets::*;
        match cli_config().table_borders {
          None | Some(CliTableBorders::Horizontal) => {
            UTF8_HORIZONTAL_ONLY
          }
          Some(CliTableBorders::Vertical) => UTF8_FULL_CONDENSED,
          Some(CliTableBorders::Inside) => UTF8_NO_BORDERS,
          Some(CliTableBorders::Outside) => UTF8_BORDERS_ONLY,
          Some(CliTableBorders::All) => UTF8_FULL,
        }
      };

      let mut table = Table::new();
      table.load_preset(preset).set_header(
        ["Name", "Value", "Secret", "Description"]
          .map(|h| Cell::new(h).add_attribute(Attribute::Bold)),
      );

      for var in variables {
        let value_cell = if var.is_secret {
          Cell::new("********").fg(Color::DarkYellow)
        } else {
          Cell::new(&var.value)
        };

        let secret_cell = if var.is_secret {
          Cell::new("Yes").fg(Color::DarkYellow)
        } else {
          Cell::new("No")
        };

        table.add_row([
          Cell::new(&var.name).add_attribute(Attribute::Bold),
          value_cell,
          secret_cell,
          Cell::new(&var.description),
        ]);
      }

      println!("{table}");
    }
    CliFormat::Json => {
      println!(
        "{}",
        serde_json::to_string_pretty(&variables)
          .context("Failed to serialize variables to JSON")?
      );
    }
  }

  Ok(())
}

async fn get_variable(name: &str, yes: bool) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;

  let variable = client
    .read(GetVariable {
      name: name.to_string(),
    })
    .await
    .context("Failed to get variable")?;

  // Check if the variable is a secret and warn the user
  if variable.is_secret && !yes {
    println!(
      "\n{}: This variable is marked as {}!",
      "WARNING".yellow().bold(),
      "SECRET".red().bold()
    );
    println!("The value may contain sensitive information.\n");
    println!(
      "Press {} to reveal the value or {} to cancel\n",
      "ENTER".green(),
      "Ctrl+C".red()
    );
    let buffer = &mut [0u8];
    std::io::stdin()
      .read_exact(buffer)
      .context("failed to read ENTER")?;
  }

  println!("\n{}: {}", "Name".dimmed(), variable.name.bold());
  println!("{}: {}", "Value".dimmed(), variable.value);
  if variable.is_secret {
    println!("{}: {}", "Secret".dimmed(), "Yes".yellow());
  }
  if !variable.description.is_empty() {
    println!("{}: {}", "Description".dimmed(), variable.description);
  }

  Ok(())
}

async fn create_variable(
  name: &str,
  value: &str,
  secret: bool,
  description: &str,
  yes: bool,
) -> anyhow::Result<()> {
  println!("\n{}: Create Variable\n", "Mode".dimmed());
  println!(" - {}:  {name}", "Name".dimmed());
  println!(" - {}: {value}", "Value".dimmed());
  println!(" - {}: {secret}", "Is Secret".dimmed());
  if !description.is_empty() {
    println!(" - {}: {description}", "Description".dimmed());
  }

  crate::command::wait_for_enter("create variable", yes)?;

  let client = super::komodo_client().await?;

  let created = client
    .write(CreateVariable {
      name: name.to_string(),
      value: value.to_string(),
      is_secret: secret,
      description: description.to_string(),
    })
    .await
    .context("Failed to create variable")?;

  println!(
    "\n{}: Variable '{}' created successfully ✅",
    "SUCCESS".green(),
    created.name.bold()
  );

  Ok(())
}

async fn delete_variable(
  name: &str,
  yes: bool,
) -> anyhow::Result<()> {
  println!("\n{}: Delete Variable\n", "Mode".dimmed());
  println!(" - {}:  {name}", "Name".dimmed());

  crate::command::wait_for_enter("delete variable", yes)?;

  let client = super::komodo_client().await?;

  let deleted = client
    .write(DeleteVariable {
      name: name.to_string(),
    })
    .await
    .context("Failed to delete variable")?;

  println!(
    "\n{}: Variable '{}' deleted successfully ✅",
    "SUCCESS".green(),
    deleted.name.bold()
  );

  Ok(())
}
