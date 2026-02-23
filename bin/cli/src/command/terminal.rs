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
      if error_chain_contains(
        &error,
        &format!("No terminal at {terminal}"),
      ) =>
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

fn error_chain_contains(
  error: &anyhow::Error,
  pattern: &str,
) -> bool {
  error.chain().any(|cause| cause.to_string().contains(pattern))
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
  let mut stream = response.0.bytes_stream();

  // Keep a small tail buffer so we can detect and strip the exit-code marker
  // even if it arrives split across response chunks.
  let mut tail = String::new();
  let mut exit_code = None;

  while let Some(chunk) = stream.next().await {
    let chunk =
      chunk.context("Failed to read terminal response chunk")?;
    tail.push_str(&String::from_utf8_lossy(&chunk));

    while let Some(marker_start) = tail.find(KOMODO_EXIT_CODE) {
      let before = &tail[..marker_start];
      if !before.is_empty() {
        print!("{before}");
        std::io::stdout()
          .flush()
          .context("Failed to flush terminal output")?;
      }

      let after_marker =
        &tail[marker_start + KOMODO_EXIT_CODE.len()..];
      let digits_len = after_marker
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .count();

      if digits_len == 0 {
        // Marker split across chunks, wait for more bytes.
        tail = tail[marker_start..].to_string();
        break;
      }

      let code = after_marker[..digits_len]
        .parse::<i32>()
        .context("Failed to parse terminal exit code marker")?;
      exit_code = Some(code);

      // Drop everything up to the end of marker line.
      let rest = &after_marker[digits_len..];
      if let Some(newline_idx) = rest.find('\n') {
        tail = rest[newline_idx + 1..].to_string();
      } else {
        tail.clear();
      }
    }

    // Flush safe content while preserving enough look-behind for split marker.
    let keep = KOMODO_EXIT_CODE.len() + 16;
    if tail.len() > keep {
      let flush_len = tail.len() - keep;
      let flush = tail[..flush_len].to_string();
      print!("{flush}");
      std::io::stdout()
        .flush()
        .context("Failed to flush terminal output")?;
      tail = tail[flush_len..].to_string();
    }
  }

  // Flush any non-marker trailing output.
  if !tail.is_empty() && !tail.contains(KOMODO_EXIT_CODE) {
    print!("{tail}");
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
