use std::io::Write;

use anyhow::Context;
use futures_util::StreamExt;
use komodo_client::{
  api::write::{CreateTerminal, TerminalRecreateMode},
  entities::{
    KOMODO_EXIT_CODE,
    config::cli::args::terminal::{
      ContainerTerminal, HostTerminal, Terminal, TerminalCommand,
    },
  },
};

#[derive(Debug, thiserror::Error)]
#[error("Terminal command exited with non-zero code: {0}")]
pub struct TerminalCommandExit(pub i32);

pub async fn handle(terminal: &Terminal) -> anyhow::Result<()> {
  match &terminal.command {
    TerminalCommand::Host(host) => execute_host(host).await,
    TerminalCommand::Container(container) => {
      execute_container(container).await
    }
  }
}

async fn execute_host(
  HostTerminal {
    server,
    terminal,
    command,
  }: &HostTerminal,
) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;
  let command = command.join(" ");

  let stream = match client
    .execute_terminal(
      server.to_string(),
      terminal.to_string(),
      command.clone(),
    )
    .await
  {
    Ok(stream) => stream,
    Err(error)
      if error
        .to_string()
        .contains(&format!("No terminal at {terminal}")) =>
    {
      // Host command execution needs a server terminal. If it's missing,
      // create it once and retry.
      client
        .write(CreateTerminal {
          server: server.to_string(),
          name: terminal.to_string(),
          command: String::from("bash"),
          recreate: TerminalRecreateMode::DifferentCommand,
        })
        .await
        .with_context(|| {
          format!(
            "Terminal '{terminal}' does not exist on server '{server}'. Failed to create it automatically"
          )
        })?;

      client
        .execute_terminal(
          server.to_string(),
          terminal.to_string(),
          command,
        )
        .await
        .context("Failed to execute command on host terminal after creating it")?
    }
    Err(error) => return Err(error),
  };

  print_stream(stream).await
}

async fn execute_container(
  ContainerTerminal {
    server,
    container,
    command,
    shell,
  }: &ContainerTerminal,
) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;
  let stream = client
    .execute_container_exec(
      server.to_string(),
      container.to_string(),
      shell.to_string(),
      command.join(" "),
    )
    .await?;
  print_stream(stream).await
}

async fn print_stream(
  response: komodo_client::terminal::TerminalStreamResponse,
) -> anyhow::Result<()> {
  let mut stream = response.into_line_stream();
  let mut exit_code = None;

  while let Some(line) = stream.next().await {
    let line =
      line.context("Failed to parse terminal response line")?;
    if let Some(code) = line
      .trim_end()
      .strip_prefix(KOMODO_EXIT_CODE)
      .and_then(|n| n.parse::<i32>().ok())
    {
      exit_code = Some(code);
      continue;
    }
    print!("{line}");
    std::io::stdout()
      .flush()
      .context("Failed to flush terminal output")?;
  }

  match exit_code {
    Some(0) => Ok(()),
    Some(code) => Err(anyhow::Error::new(TerminalCommandExit(code))),
    None => Err(anyhow::anyhow!(
      "Terminal output closed without an exit code marker"
    )),
  }
}
