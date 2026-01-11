use anyhow::Context;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color, Table};
use komodo_client::{
  api::read::{GetProcedure, ListUpdates},
  entities::{
    Operation,
    config::cli::{
      CliTableBorders,
      args::procedure::{Procedure, ProcedureCommand},
    },
  },
};

use crate::config::cli_config;

pub async fn handle(procedure: &Procedure) -> anyhow::Result<()> {
  match &procedure.command {
    ProcedureCommand::Status => {
      show_status(&procedure.procedure).await
    }
    ProcedureCommand::Logs { limit } => {
      show_logs(&procedure.procedure, *limit).await
    }
  }
}

async fn show_status(name: &str) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;

  let procedure = client
    .read(GetProcedure {
      procedure: name.to_string(),
    })
    .await
    .context("Failed to get procedure")?;

  println!("\n{}: {}", "Name".dimmed(), procedure.name.bold());
  println!("{}: {}", "ID".dimmed(), procedure.id);

  let schedule_enabled = if procedure.config.schedule_enabled {
    "true".green()
  } else {
    "false".dimmed()
  };
  println!("{}: {}", "Schedule Enabled".dimmed(), schedule_enabled);

  if !procedure.config.schedule.is_empty() {
    println!(
      "{}: {}",
      "Schedule".dimmed(),
      procedure.config.schedule
    );
  }

  println!(
    "{}: {}",
    "Stages".dimmed(),
    procedure.config.stages.len()
  );

  if !procedure.description.is_empty() {
    println!("{}: {}", "Description".dimmed(), procedure.description);
  }

  Ok(())
}

async fn show_logs(name: &str, limit: u32) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;

  // First get the procedure to get its ID
  let procedure = client
    .read(GetProcedure {
      procedure: name.to_string(),
    })
    .await
    .context("Failed to get procedure")?;

  // Query updates for this procedure
  let updates = client
    .read(ListUpdates {
      query: Some(bson::doc! {
        "target.type": "Procedure",
        "target.id": &procedure.id,
        "operation": { "$in": [
          Operation::RunProcedure.to_string()
        ]}
      }),
      page: 0,
    })
    .await
    .context("Failed to list procedure runs")?;

  if updates.updates.is_empty() {
    println!(
      "{}: No runs found for procedure '{}'",
      "INFO".green(),
      name.bold()
    );
    return Ok(());
  }

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
    ["ID", "Status", "Started", "Operator"]
      .map(|h| Cell::new(h).add_attribute(Attribute::Bold)),
  );

  for (i, update) in updates.updates.iter().enumerate() {
    if i >= limit as usize {
      break;
    }

    let status_color = if update.success {
      Color::Green
    } else {
      Color::Red
    };

    let status = if update.success { "Success" } else { "Failed" };

    let started = super::format_timetamp(update.start_ts)
      .unwrap_or_else(|_| "-".to_string());

    table.add_row([
      Cell::new(&update.id),
      Cell::new(status)
        .fg(status_color)
        .add_attribute(Attribute::Bold),
      Cell::new(&started),
      Cell::new(&update.operator),
    ]);
  }

  println!("{table}");
  Ok(())
}
