use anyhow::Context;
use futures_util::{Stream, StreamExt, TryStreamExt};
use mogh_error::deserialize_error;

use crate::{
  KomodoClient,
  api::terminal::{ExecuteTerminalBody, InitTerminal},
  entities::terminal::TerminalTarget,
};

pub struct TerminalStreamResponse(pub reqwest::Response);

impl TerminalStreamResponse {
  pub fn into_line_stream(
    self,
  ) -> impl Stream<Item = Result<String, tokio_util::codec::LinesCodecError>>
  {
    tokio_util::codec::FramedRead::new(
      tokio_util::io::StreamReader::new(
        self.0.bytes_stream().map_err(std::io::Error::other),
      ),
      tokio_util::codec::LinesCodec::new(),
    )
    .map(|line| line.map(|line| line + "\n"))
  }
}

impl KomodoClient {
  /// Executes a command against a terminal target and streams the output.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_terminal(
    &self,
    request: ExecuteTerminalBody,
  ) -> anyhow::Result<TerminalStreamResponse> {
    let req = self
      .reqwest
      .post(format!("{}/terminal/execute", self.address))
      .header("x-api-key", &self.key)
      .header("x-api-secret", &self.secret)
      .header("content-type", "application/json")
      .json(&request);
    terminal_stream_response(req).await
  }

  /// Executes a command on a host terminal and streams the output.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_server_terminal(
    &self,
    server: String,
    terminal: Option<String>,
    command: String,
    init: Option<InitTerminal>,
  ) -> anyhow::Result<TerminalStreamResponse> {
    self
      .execute_terminal(ExecuteTerminalBody {
        target: TerminalTarget::Server {
          server: Some(server),
        },
        terminal,
        command,
        init,
      })
      .await
  }

  /// Executes a command inside a container terminal and streams the output.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_container_terminal(
    &self,
    server: String,
    container: String,
    terminal: Option<String>,
    command: String,
    init: Option<InitTerminal>,
  ) -> anyhow::Result<TerminalStreamResponse> {
    self
      .execute_terminal(ExecuteTerminalBody {
        target: TerminalTarget::Container { server, container },
        terminal,
        command,
        init,
      })
      .await
  }
}

async fn terminal_stream_response(
  req: reqwest::RequestBuilder,
) -> anyhow::Result<TerminalStreamResponse> {
  let res = req
    .send()
    .await
    .context("Failed at request to core terminal api")?;
  let status = res.status();
  if status.is_success() {
    Ok(TerminalStreamResponse(res))
  } else {
    let text = res
      .text()
      .await
      .context("Failed to convert response to text")?;
    let error = deserialize_error(text).context(status);
    Err(error)
  }
}
