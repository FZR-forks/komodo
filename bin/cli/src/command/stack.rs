use anyhow::Context;
use colored::Colorize;
use comfy_table::{Attribute, Cell, Color, Table};
use komodo_client::{
  api::read::{GetStackLog, ListStackServices},
  entities::config::cli::args::stack::{
    Stack, StackCommand, StackLogs,
  },
};

use crate::config::cli_config;

pub async fn handle(stack: &Stack) -> anyhow::Result<()> {
  match &stack.command {
    StackCommand::Logs(logs) => show_logs(&stack.stack, logs).await,
    StackCommand::Services => list_services(&stack.stack).await,
  }
}

async fn list_services(stack: &str) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;

  let services = client
    .read(ListStackServices {
      stack: stack.to_string(),
    })
    .await
    .context("Failed to list stack services")?;

  if services.is_empty() {
    println!(
      "{}: No services found for stack '{}'",
      "INFO".green(),
      stack.bold()
    );
    return Ok(());
  }

  let preset = {
    use comfy_table::presets::*;
    use komodo_client::entities::config::cli::CliTableBorders;
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
    ["Service", "Image", "Container", "State"]
      .map(|h| Cell::new(h).add_attribute(Attribute::Bold)),
  );

  for service in services {
    let (container_name, state, color) = if let Some(container) =
      &service.container
    {
      let state = container.state.to_string();
      let color = match container.state {
          komodo_client::entities::docker::container::ContainerStateStatusEnum::Running => {
            Color::Green
          }
          komodo_client::entities::docker::container::ContainerStateStatusEnum::Paused => {
            Color::DarkYellow
          }
          komodo_client::entities::docker::container::ContainerStateStatusEnum::Empty => {
            Color::Grey
          }
          _ => Color::Red,
        };
      (container.name.clone(), state, color)
    } else {
      (String::from("-"), String::from("Not running"), Color::Grey)
    };

    table.add_row([
      Cell::new(&service.service).add_attribute(Attribute::Bold),
      Cell::new(&service.image),
      Cell::new(&container_name),
      Cell::new(&state).fg(color).add_attribute(Attribute::Bold),
    ]);
  }

  println!("{table}");
  Ok(())
}

async fn show_logs(
  stack: &str,
  logs: &StackLogs,
) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;

  // Cap tail at 5000
  let tail = logs.tail.min(5000);

  let log = client
    .read(GetStackLog {
      stack: stack.to_string(),
      services: logs.services.clone(),
      tail,
      timestamps: logs.timestamps,
    })
    .await
    .context("Failed to get stack logs")?;

  if log.stdout.is_empty() && log.stderr.is_empty() {
    println!(
      "{}: No logs found for stack '{}'",
      "INFO".green(),
      stack.bold()
    );
    return Ok(());
  }

  // Print stdout
  if !log.stdout.is_empty() {
    print!("{}", log.stdout);
  }

  // Print stderr if present
  if !log.stderr.is_empty() {
    eprint!("{}", log.stderr.red());
  }

  Ok(())
}
