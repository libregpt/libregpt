use std::borrow::Cow;

use anyhow::Context;
use async_trait::async_trait;
use hyper::body::HttpBody;
use hyper::client::HttpConnector;
use hyper::{header, Body, Client, Method, Request};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use rand_user_agent::UserAgent;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::oneshot;
use tokio::task;
use tracing::error;

const MSG_PREFIX: &str = r#"{"role":"assistant","id":"chatcmpl-"#;

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
    let connector = HttpsConnectorBuilder::new()
      .with_native_roots()
      .https_only()
      .enable_http1()
      .enable_http2()
      .build();

    let client = Client::builder().build(connector);

    Self { client }
  }
}

#[async_trait]
impl super::Provider for Provider {
  async fn ask<'a>(
    &self,
    prompt: &str,
    state: Option<Cow<'a, str>>,
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
      .uri("https://chatbot.theb.ai/api/chat-process")
      .header(header::CONTENT_TYPE, "application/json")
      .header(header::USER_AGENT, UserAgent::random().to_string())
      .body(Body::from(serde_json::to_string(&body)?))?;

    let res = self.client.request(req).await?;
    let (mut tx, rx) = Body::channel();
    let (msg_id_tx, msg_id_rx) = oneshot::channel();
    let mut msg_id_tx = Some(msg_id_tx);

    task::spawn(async move {
      let mut body = res.into_body();

      while let Some(Ok(chunk)) = body.data().await {
        let chunk = match String::from_utf8(chunk.into()) {
          Ok(chunk) => chunk,
          Err(_) => {
            error!("invalid utf-8 chunk");
            continue;
          }
        };

        for partial_msg in chunk.split(MSG_PREFIX) {
          if !partial_msg.is_empty() && partial_msg != "\n" {
            let mut raw_msg = String::with_capacity(MSG_PREFIX.len() + partial_msg.len());
            raw_msg.push_str(MSG_PREFIX);
            raw_msg.push_str(partial_msg);

            match serde_json::from_str::<Message>(&raw_msg) {
              Ok(msg) => {
                if let Some(msg_id_tx) = msg_id_tx.take() {
                  drop(msg_id_tx.send(msg.id));
                }
                drop(tx.send_data(msg.delta.into()).await);
              }
              Err(err) => error!("failed to deserialize chunk: {err}"),
            }
          }
        }
      }
    });

    let msg_id = msg_id_rx.await.context("failed to receive msg id")?;

    Ok((Some(msg_id), rx))
  }
}
