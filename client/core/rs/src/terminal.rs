use anyhow::Context;
use futures::{Stream, StreamExt, TryStreamExt};

use crate::{
  KomodoClient,
  api::terminal::{ExecuteContainerExecBody, ExecuteTerminalBody},
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
  /// Executes command on a host terminal, and streams the output.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_terminal(
    &self,
    server: String,
    terminal: String,
    command: String,
  ) -> anyhow::Result<TerminalStreamResponse> {
    let req = self
      .reqwest
      .post(format!("{}/terminal/execute", self.address))
      .header("x-api-key", &self.key)
      .header("x-api-secret", &self.secret)
      .header("content-type", "application/json")
      .json(&ExecuteTerminalBody {
        server,
        terminal,
        command,
      });
    terminal_stream_response(req).await
  }

  /// Executes command in a container shell, and streams the output.
  #[tracing::instrument(level = "debug", skip(self))]
  pub async fn execute_container_exec(
    &self,
    server: String,
    container: String,
    shell: String,
    command: String,
  ) -> anyhow::Result<TerminalStreamResponse> {
    let req = self
      .reqwest
      .post(format!("{}/terminal/execute/container", self.address))
      .header("x-api-key", &self.key)
      .header("x-api-secret", &self.secret)
      .header("content-type", "application/json")
      .json(&ExecuteContainerExecBody {
        server,
        container,
        shell,
        command,
      });
    terminal_stream_response(req).await
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
    let error = serror::deserialize_error(text).context(status);
    Err(error)
  }
}
