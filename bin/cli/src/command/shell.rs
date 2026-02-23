use std::{
  io::Write,
  sync::atomic::{AtomicU64, Ordering},
  time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, anyhow, bail};
use futures_util::StreamExt;
use komodo_client::{
  api::write::{CreateTerminal, DeleteTerminal, TerminalRecreateMode},
  entities::{
    KOMODO_EXIT_CODE,
    config::cli::args::shell::{
      ContainerShell, HostShell, Shell, ShellCommand,
    },
  },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellKind {
  Bash,
  Sh,
}

impl ShellKind {
  fn as_str(self) -> &'static str {
    match self {
      Self::Bash => "bash",
      Self::Sh => "sh",
    }
  }
}

pub struct ShellCommandExit(pub i32);

impl std::fmt::Display for ShellCommandExit {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    write!(f, "Shell command exited with non-zero code: {}", self.0)
  }
}

impl std::fmt::Debug for ShellCommandExit {
  fn fmt(
    &self,
    f: &mut std::fmt::Formatter<'_>,
  ) -> std::fmt::Result {
    f.debug_tuple("ShellCommandExit").field(&self.0).finish()
  }
}

impl std::error::Error for ShellCommandExit {}

pub async fn handle(
  shell_kind: ShellKind,
  shell: &Shell,
) -> anyhow::Result<()> {
  match &shell.command {
    ShellCommand::Host(host) => execute_host(shell_kind, host).await,
    ShellCommand::Container(container) => {
      execute_container(shell_kind, container).await
    }
  }
}

async fn execute_host(
  shell_kind: ShellKind,
  HostShell { server, command }: &HostShell,
) -> anyhow::Result<()> {
  validate_command(command)?;

  let terminal_shell = match shell_kind {
    ShellKind::Bash => "bash",
    // Reliability over strict shell identity for host execution:
    // use the stable host terminal path.
    ShellKind::Sh => "bash",
  };
  let execute_command = match shell_kind {
    ShellKind::Bash => command.to_string(),
    ShellKind::Sh => command.to_string(),
  };

  let terminal = next_terminal_name(server, shell_kind);
  let client = super::komodo_client().await?;

  client
    .write(CreateTerminal {
      server: server.to_string(),
      name: terminal.clone(),
      command: terminal_shell.to_string(),
      recreate: TerminalRecreateMode::Never,
    })
    .await
    .with_context(|| {
      format!(
        "Failed to create temporary '{}' terminal on host '{}'",
        shell_kind.as_str(),
        server
      )
    })?;

  let execute_result = async {
    let stream = client
      .execute_terminal(
        server.to_string(),
        terminal.clone(),
        execute_command.clone(),
      )
      .await
      .with_context(|| {
        format!(
          "Failed to execute command on host '{server}' terminal '{terminal}'"
        )
      })?;
    print_stream(stream).await
  }
  .await;

  let delete_result = client
    .write(DeleteTerminal {
      server: server.to_string(),
      terminal: terminal.clone(),
    })
    .await;

  match (execute_result, delete_result) {
    (Ok(()), Ok(_)) => Ok(()),
    (Ok(()), Err(delete_error)) => Err(delete_error).with_context(|| {
      format!(
        "Command completed, but failed to delete temporary terminal '{terminal}' on host '{server}'"
      )
    }),
    (Err(execute_error), Ok(_)) => Err(execute_error),
    (Err(execute_error), Err(delete_error)) => {
      warn!(
        "failed to delete temporary terminal '{terminal}' on host '{server}' after command error: {delete_error:#}"
      );
      Err(execute_error)
    }
  }
}

async fn execute_container(
  shell_kind: ShellKind,
  ContainerShell {
    server,
    container,
    command,
  }: &ContainerShell,
) -> anyhow::Result<()> {
  validate_command(command)?;
  let client = super::komodo_client().await?;

  match execute_container_with_shell(
    client,
    server,
    container,
    shell_kind.as_str(),
    command,
  )
  .await
  {
    Ok(()) => Ok(()),
    Err(error)
      if shell_kind == ShellKind::Bash
        && should_retry_with_sh(&error) =>
    {
      eprintln!(
        "bash unavailable in container '{container}', retrying with sh"
      );
      execute_container_with_shell(
        client,
        server,
        container,
        "sh",
        command,
      )
      .await
      .with_context(|| {
        format!(
          "Command failed using both bash and sh in container '{container}' on host '{server}'"
        )
      })
    }
    Err(error) => Err(error),
  }
}

async fn execute_container_with_shell(
  client: &komodo_client::KomodoClient,
  server: &str,
  container: &str,
  shell: &str,
  command: &str,
) -> anyhow::Result<()> {
  let stream = client
    .execute_container_exec(
      server.to_string(),
      container.to_string(),
      shell.to_string(),
      command.to_string(),
    )
    .await
    .with_context(|| {
      format!(
        "Failed to execute command in container '{container}' on host '{server}' using shell '{shell}'"
      )
    })?;
  print_stream(stream).await
}

fn should_retry_with_sh(error: &anyhow::Error) -> bool {
  // Cover both low-level process startup errors and common shell not found messages.
  const PATTERNS: &[&str] = &[
    "child process exited immediately with code 126",
    "child process exited immediately with code 127",
    "executable file not found",
    "no such file or directory",
    "bash: not found",
    "connection closed",
    "failed to create terminal for container exec",
    "failed to init terminal",
  ];
  let chain = error
    .chain()
    .map(ToString::to_string)
    .collect::<Vec<_>>()
    .join("\n")
    .to_lowercase();
  PATTERNS.iter().any(|pattern| chain.contains(pattern))
}

fn validate_command(command: &str) -> anyhow::Result<()> {
  if command.trim().is_empty() {
    bail!("The --command value cannot be empty");
  }
  Ok(())
}

fn next_terminal_name(server: &str, shell_kind: ShellKind) -> String {
  static COUNTER: AtomicU64 = AtomicU64::new(0);

  let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
  let ts_ms = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis();
  let server_slug = server
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
        c
      } else {
        '-'
      }
    })
    .collect::<String>();

  format!(
    "km-{}-{}-{}-{}",
    shell_kind.as_str(),
    server_slug,
    ts_ms,
    counter
  )
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
      chunk.context("Failed to read shell response chunk")?;
    tail.push_str(&String::from_utf8_lossy(&chunk));

    while let Some(marker_start) = tail.find(KOMODO_EXIT_CODE) {
      let before = &tail[..marker_start];
      if !before.is_empty() {
        print!("{before}");
        std::io::stdout()
          .flush()
          .context("Failed to flush shell output")?;
      }

      let after_marker =
        &tail[marker_start + KOMODO_EXIT_CODE.len()..];

      let Some(first_after_marker) = after_marker.chars().next()
      else {
        // Marker split across chunks, wait for more bytes.
        tail = tail[marker_start..].to_string();
        break;
      };

      if !first_after_marker.is_ascii_digit()
        && first_after_marker != '-'
      {
        // False positive marker text in streamed output (for example,
        // shell-echoed command containing `__KOMODO_EXIT_CODE:%d`).
        print!("{KOMODO_EXIT_CODE}");
        std::io::stdout()
          .flush()
          .context("Failed to flush shell output")?;
        tail = after_marker.to_string();
        continue;
      }

      let digits_len = after_marker
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '-')
        .count();

      let code = after_marker[..digits_len]
        .parse::<i32>()
        .context("Failed to parse shell exit code marker")?;
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
        .context("Failed to flush shell output")?;
      tail = tail[flush_len..].to_string();
    }
  }

  // Flush any non-marker trailing output.
  if !tail.is_empty() && !tail.contains(KOMODO_EXIT_CODE) {
    print!("{tail}");
    std::io::stdout()
      .flush()
      .context("Failed to flush shell output")?;
  }

  match exit_code {
    Some(0) => Ok(()),
    Some(code) => Err(anyhow::Error::new(ShellCommandExit(code))),
    None => Err(anyhow!(
      "Shell output closed without an exit code marker"
    )),
  }
}
