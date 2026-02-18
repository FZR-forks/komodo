use anyhow::Context;
use futures_util::StreamExt;
use komodo_client::entities::{
  KOMODO_EXIT_CODE,
  config::cli::args::terminal::{
    ContainerTerminal, HostTerminal, Terminal, TerminalCommand,
  },
};

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
  let stream = client
    .execute_terminal(
      server.to_string(),
      terminal.to_string(),
      command.to_string(),
    )
    .await?;
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
      command.to_string(),
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
  }

  match exit_code {
    Some(0) => Ok(()),
    Some(code) => Err(anyhow::anyhow!(
      "Terminal command exited with non-zero code: {code}"
    )),
    None => Err(anyhow::anyhow!(
      "Terminal output closed without an exit code marker"
    )),
  }
}
