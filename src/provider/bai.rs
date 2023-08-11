use anyhow::Context;
use async_trait::async_trait;
use hyper::client::HttpConnector;
use hyper::{header, Body, Client, Method, Request};
use hyper_rustls::HttpsConnector;
use rand_user_agent::UserAgent;
use serde::Deserialize;
use serde_json::json;
use tokio::io::AsyncBufReadExt;
use tokio::sync::oneshot;
use tokio::task;
use tokio_util::io::StreamReader;
use tracing::error;

use crate::util::{new_rustls_connector, BodyStream};

#[derive(Deserialize)]
struct Message {
  delta: String,
  id: String,
}

pub struct Provider {
  client: Client<HttpsConnector<HttpConnector>>,
}

impl Provider {
  pub fn new() -> Self {
    let connector = new_rustls_connector();
    let client = Client::builder().build(connector);

    Self { client }
  }
}

#[async_trait]
impl super::Provider for Provider {
  async fn ask<'a>(
    &self,
    prompt: &str,
    state: Option<&str>,
  ) -> anyhow::Result<(Option<String>, Body)> {
    let body = if let Some(parent_msg_id) = state {
      json!({
        "prompt": prompt,
        "options": {
          "parentMessageId": parent_msg_id
        }
      })
    } else {
      json!({ "prompt": prompt })
    };

    let req = Request::builder()
      .method(Method::POST)
      .uri("https://beta.theb.ai/api/chat-process")
      .header(header::CONTENT_TYPE, "application/json")
      .header(header::USER_AGENT, UserAgent::random().to_string())
      .body(Body::from(serde_json::to_string(&body)?))?;

    let res = self.client.request(req).await?;
    let (mut tx, rx) = Body::channel();
    let (msg_id_tx, msg_id_rx) = oneshot::channel();
    let mut msg_id_tx = Some(msg_id_tx);

    task::spawn(async move {
      let mut reader = StreamReader::new(BodyStream::from(res.into_body()));
      let mut line = String::with_capacity(1 << 14);

      loop {
        match reader.read_line(&mut line).await {
          Ok(0) => break,
          Ok(_) => {
            match serde_json::from_str::<Message>(&line) {
              Ok(msg) => {
                if let Some(msg_id_tx) = msg_id_tx.take() {
                  drop(msg_id_tx.send(msg.id));
                }
                drop(tx.send_data(msg.delta.into()).await);
              }
              Err(err) => error!("failed to deserialize line: {err}"),
            }
            line.clear();
          }
          Err(err) => {
            error!("failed to read line: {err}");
            break;
          }
        }
      }
    });

    let msg_id = msg_id_rx.await.context("failed to receive msg id")?;

    Ok((Some(msg_id), rx))
  }
}
