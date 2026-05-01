use std::{
  future::Future,
  time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, anyhow};
use colored::Colorize;
use futures_util::StreamExt;
use komodo_client::{
  api::{
    read::{GetServer, ListAllDockerContainers, ListServers},
    terminal::{
      ConnectTerminalQuery, ExecuteTerminalBody, InitTerminal,
    },
    write::DeleteTerminal,
  },
  entities::{
    KOMODO_EXIT_CODE,
    config::cli::args::terminal::{
      Attach, Exec, ExecContainer, ExecHost, ExecTarget, Ssh,
      SshContainer, SshHost, SshTarget,
    },
    server::ServerQuery,
    terminal::{
      ContainerTerminalMode, TerminalRecreateMode,
      TerminalResizeMessage, TerminalStdinMessage, TerminalTarget,
    },
  },
  ws::terminal::TerminalWebsocket,
};
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub(crate) struct RemoteCommandExit {
  code: i32,
}

impl RemoteCommandExit {
  pub(crate) fn new(code: i32) -> Self {
    Self { code }
  }

  pub(crate) fn code(&self) -> i32 {
    self.code
  }
}

impl std::fmt::Display for RemoteCommandExit {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "remote command exited with code {}", self.code)
  }
}

impl std::error::Error for RemoteCommandExit {}

pub async fn handle_ssh(ssh: &Ssh) -> anyhow::Result<()> {
  match &ssh.target {
    SshTarget::Host(host) => connect_host(host).await,
    SshTarget::Container(container) => {
      connect_container(container).await
    }
  }
}

pub async fn handle_exec(exec: &Exec) -> anyhow::Result<()> {
  match &exec.target {
    ExecTarget::Host(host) => execute_host(host).await,
    ExecTarget::Container(container) => {
      execute_container(container).await
    }
  }
}

pub async fn handle_attach(
  Attach {
    server,
    container,
    terminal,
    recreate,
  }: &Attach,
) -> anyhow::Result<()> {
  let server = get_server(server.clone(), container).await?;
  connect_terminal(
    TerminalTarget::Container {
      server: server.clone(),
      container: container.to_string(),
    },
    format!("{server}/{container} (attach)"),
    terminal.to_string(),
    InitTerminal {
      command: None,
      recreate: recreate_mode(*recreate),
      mode: Some(ContainerTerminalMode::Attach),
    },
  )
  .await
}

async fn connect_host(
  SshHost {
    server,
    terminal,
    shell,
    recreate,
  }: &SshHost,
) -> anyhow::Result<()> {
  connect_terminal(
    TerminalTarget::Server {
      server: Some(server.to_string()),
    },
    server.to_string(),
    terminal.to_string(),
    InitTerminal {
      command: shell.clone(),
      recreate: recreate_mode(*recreate),
      mode: None,
    },
  )
  .await
}

async fn connect_container(
  SshContainer {
    server,
    container,
    terminal,
    shell,
    recreate,
  }: &SshContainer,
) -> anyhow::Result<()> {
  let server = get_server(server.clone(), container).await?;
  connect_terminal(
    TerminalTarget::Container {
      server: server.clone(),
      container: container.to_string(),
    },
    format!("{server}/{container}"),
    terminal.to_string(),
    InitTerminal {
      command: Some(shell.to_string()),
      recreate: recreate_mode(*recreate),
      mode: Some(ContainerTerminalMode::Exec),
    },
  )
  .await
}

async fn execute_host(
  ExecHost {
    server,
    command,
    terminal,
    shell,
    recreate,
  }: &ExecHost,
) -> anyhow::Result<()> {
  execute_target(
    TerminalTarget::Server {
      server: Some(server.to_string()),
    },
    command.to_string(),
    terminal.to_string(),
    InitTerminal {
      command: shell.clone(),
      recreate: recreate_mode(*recreate),
      mode: None,
    },
  )
  .await
}

async fn execute_container(
  ExecContainer {
    server,
    container,
    command,
    terminal,
    shell,
    recreate,
  }: &ExecContainer,
) -> anyhow::Result<()> {
  let server = get_server(server.clone(), container).await?;
  execute_target(
    TerminalTarget::Container {
      server,
      container: container.to_string(),
    },
    command.to_string(),
    terminal.to_string(),
    InitTerminal {
      command: Some(shell.to_string()),
      recreate: recreate_mode(*recreate),
      mode: Some(ContainerTerminalMode::Exec),
    },
  )
  .await
}

async fn connect_terminal(
  target: TerminalTarget,
  label: String,
  terminal: String,
  init: InitTerminal,
) -> anyhow::Result<()> {
  handle_terminal_forwarding(&label, async move {
    let query = ConnectTerminalQuery {
      target,
      terminal: Some(terminal),
      init: Some(init),
    };
    super::komodo_client().await?.connect_terminal(&query).await
  })
  .await
}

async fn execute_target(
  target: TerminalTarget,
  command: String,
  terminal: String,
  mut init: InitTerminal,
) -> anyhow::Result<()> {
  let client = super::komodo_client().await?;
  let terminal = temporary_exec_terminal_name(&terminal);
  init.recreate = TerminalRecreateMode::Always;

  let res = match client
    .execute_terminal(ExecuteTerminalBody {
      target: target.clone(),
      terminal: Some(terminal.clone()),
      command,
      init: Some(init),
    })
    .await
  {
    Ok(stream) => print_stream(stream).await,
    Err(e) => Err(e),
  };

  if let Err(e) = client
    .write(DeleteTerminal {
      target,
      terminal: terminal.clone(),
    })
    .await
  {
    eprintln!(
      "{}: failed to delete temporary terminal '{terminal}' | {e}",
      "WARN".yellow()
    );
  }

  res
}

async fn print_stream(
  response: komodo_client::terminal::TerminalStreamResponse,
) -> anyhow::Result<()> {
  let mut stream = response.into_line_stream();
  let mut output = Vec::new();

  while let Some(line) = stream.next().await {
    let line =
      line.context("Failed to parse terminal response line")?;
    output.extend_from_slice(line.as_bytes());
  }

  let marker = KOMODO_EXIT_CODE.as_bytes();
  let Some(marker_at) = find_last_subslice(&output, marker) else {
    tokio::io::stdout().write_all(&output).await?;
    return Err(anyhow!(
      "Terminal output closed without an exit code marker"
    ));
  };

  tokio::io::stdout()
    .write_all(&output[..marker_at])
    .await?;
  let _ = tokio::io::stdout().flush().await;

  let exit_code = output[marker_at + marker.len()..]
    .split(|byte| *byte == b'\n' || *byte == b'\r')
    .next()
    .and_then(|bytes| std::str::from_utf8(bytes).ok())
    .and_then(|code| code.parse::<i32>().ok());

  match exit_code {
    Some(0) => Ok(()),
    Some(code) => Err(RemoteCommandExit::new(code).into()),
    None => Err(anyhow!("Invalid terminal exit code marker")),
  }
}

fn find_last_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
  haystack
    .windows(needle.len())
    .rposition(|window| window == needle)
}

fn temporary_exec_terminal_name(prefix: &str) -> String {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_nanos())
    .unwrap_or_default();
  format!("{prefix}-exec-{}-{timestamp}", std::process::id())
}

async fn get_server(
  server: Option<String>,
  container: &str,
) -> anyhow::Result<String> {
  if let Some(server) = server {
    return Ok(server);
  }

  let client = super::komodo_client().await?;

  let mut containers = client
    .read(ListAllDockerContainers {
      servers: Default::default(),
      containers: vec![container.to_string()],
    })
    .await?;

  if containers.is_empty() {
    return Err(anyhow!(
      "Did not find any container matching {container}"
    ));
  }

  if containers.len() == 1 {
    let server_id = containers
      .pop()
      .context("Shouldn't happen")?
      .server_id
      .context("Container doesn't have server_id")?;
    let server_name =
      client.read(GetServer { server: server_id }).await?.name;
    return Ok(server_name);
  }

  let servers = containers
    .into_iter()
    .flat_map(|container| container.server_id)
    .collect::<Vec<_>>();

  let servers = client
    .read(ListServers {
      query: ServerQuery::builder().names(servers).build(),
    })
    .await?
    .into_iter()
    .map(|server| format!("\t- {}", server.name.bold()))
    .collect::<Vec<_>>()
    .join("\n");

  Err(anyhow!(
    "Multiple containers matching '{}' on Servers:\n{servers}",
    container.bold(),
  ))
}

fn recreate_mode(recreate: bool) -> TerminalRecreateMode {
  if recreate {
    TerminalRecreateMode::Always
  } else {
    TerminalRecreateMode::DifferentCommand
  }
}

async fn handle_terminal_forwarding<
  C: Future<Output = anyhow::Result<TerminalWebsocket>>,
>(
  label: &str,
  connect: C,
) -> anyhow::Result<()> {
  let (write_tx, mut write_rx) =
    tokio::sync::mpsc::channel::<TerminalStdinMessage>(1024);

  let mut sigwinch = tokio::signal::unix::signal(
    tokio::signal::unix::SignalKind::window_change(),
  )
  .context("failed to register SIGWINCH handler")?;

  write_tx.send(resize_message()?).await?;

  let cancel = CancellationToken::new();

  let forward_resize = async {
    while future_or_cancel(sigwinch.recv(), &cancel)
      .await
      .flatten()
      .is_some()
    {
      if let Ok(resize_message) = resize_message()
        && write_tx.send(resize_message).await.is_err()
      {
        break;
      }
    }
    cancel.cancel();
  };

  let forward_stdin = async {
    let mut stdin = tokio::io::stdin();
    let mut buf = [0u8; 8192];
    while let Some(Ok(n)) =
      future_or_cancel(stdin.read(&mut buf), &cancel).await
    {
      if n == 0 {
        break;
      }
      let bytes = &buf[..n];
      if bytes == [197, 147] {
        break;
      }
      if write_tx
        .send(TerminalStdinMessage::Forward(bytes.to_vec()))
        .await
        .is_err()
      {
        break;
      };
    }
    cancel.cancel();
  };

  let (mut ws_write, mut ws_read) = connect.await?.split();

  let forward_write = async {
    while let Some(message) =
      future_or_cancel(write_rx.recv(), &cancel).await.flatten()
    {
      if let Err(e) = ws_write.send_stdin_message(message).await {
        cancel.cancel();
        return Some(e);
      };
    }
    cancel.cancel();
    None
  };

  let forward_read = async {
    let mut stdout = tokio::io::stdout();

    if let Err(e) = write_connection_message(&mut stdout, label)
      .await
      .context("Failed to write text to stdout")
    {
      cancel.cancel();
      return Some(e);
    }

    while let Some(msg) =
      future_or_cancel(ws_read.receive_stdout(), &cancel).await
    {
      let bytes = match msg {
        Ok(Some(bytes)) => bytes,
        Ok(None) => break,
        Err(e) => {
          cancel.cancel();
          return Some(e.context("Websocket read error"));
        }
      };
      if let Err(e) = stdout
        .write_all(&bytes)
        .await
        .context("Failed to write text to stdout")
      {
        cancel.cancel();
        return Some(e);
      }
      let _ = stdout.flush().await;
    }
    cancel.cancel();
    None
  };

  let guard = RawModeGuard::enable_raw_mode()?;

  let (_, _, write_error, read_error) = tokio::join!(
    forward_resize,
    forward_stdin,
    forward_write,
    forward_read
  );

  drop(guard);

  if let Some(e) = write_error {
    eprintln!("\nFailed to forward stdin | {e:#}");
  }

  if let Some(e) = read_error {
    eprintln!("\nFailed to forward stdout | {e:#}");
  }

  println!("\n\n{} {}", "connection".bold(), "closed".red().bold());

  std::process::exit(0)
}

async fn write_connection_message(
  stdout: &mut tokio::io::Stdout,
  label: &str,
) -> anyhow::Result<()> {
  let message_clean = format!("# Connected to {label} (km) #");
  let border = "=".repeat(message_clean.chars().count());

  let message = format!(
    "# {} to {} {} #",
    "Connected".green().bold(),
    label.bold(),
    "(km)".dimmed()
  );

  stdout
    .write_all(
      format!("\n{border}\r\n{message}\r\n{border}\r\n").as_bytes(),
    )
    .await?;
  let _ = stdout.flush().await;

  Ok(())
}

fn resize_message() -> anyhow::Result<TerminalStdinMessage> {
  let (cols, rows) = crossterm::terminal::size()
    .context("Failed to get terminal size")?;
  Ok(TerminalStdinMessage::Resize(TerminalResizeMessage {
    rows,
    cols,
  }))
}

struct RawModeGuard;

impl RawModeGuard {
  fn enable_raw_mode() -> anyhow::Result<Self> {
    crossterm::terminal::enable_raw_mode()
      .context("Failed to enable terminal raw mode")?;
    Ok(Self)
  }
}

impl Drop for RawModeGuard {
  fn drop(&mut self) {
    if let Err(e) = crossterm::terminal::disable_raw_mode() {
      eprintln!("Failed to disable terminal raw mode | {e:?}");
    }
  }
}

async fn future_or_cancel<T, F: Future<Output = T>>(
  fut: F,
  cancel: &CancellationToken,
) -> Option<T> {
  tokio::select! {
    res = fut => Some(res),
    _ = cancel.cancelled() => None
  }
}
