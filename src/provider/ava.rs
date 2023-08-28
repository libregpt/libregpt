use async_trait::async_trait;
use hyper::{Body, Client, header, Method, Request};
use hyper::client::HttpConnector;
use hyper_rustls::HttpsConnector;
use rand_user_agent::UserAgent;
use serde::Deserialize;
use tokio::io::AsyncBufReadExt;
use tokio::task;
use tokio_util::io::StreamReader;
use tracing::error;
use crate::util::{BodyStream, new_rustls_connector};

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
  async fn ask<'a>(&self, prompt: &str, state: Option<&str>) -> anyhow::Result<(Option<String>, Body)> {
    let prompt = serde_json::to_string(prompt)?;
    let chat_len = state.map_or(2, |chat| chat.len() + 1) + 26 + prompt.len();
    let mut body = String::with_capacity(12 + chat_len + 1);

    body.push_str("{\"messages\":");

    if let Some(chat) = state {
      body.push_str(&chat[..chat.len() - 1]);
      body.push(',');
    } else {
      body.push('[');
    };

    body.push_str("{\"role\":\"user\",\"content\":");
    body.push_str(&prompt);
    body.push_str("}]}");

    let req = Request::builder()
      .method(Method::POST)
      .uri("https://ava-alpha-api.codelink.io/api/chat")
      .header(header::CONTENT_TYPE, "application/json")
      .header(header::USER_AGENT, &UserAgent::random().to_string())
      .body(Body::from(body))?;

    let res = self.client.request(req).await?;
    let (mut tx, rx) = Body::channel();

    task::spawn(async move {
      let mut reader = StreamReader::new(BodyStream::from(res.into_body()));
      let mut line = String::with_capacity(256);

      loop {
        match reader.read_line(&mut line).await {
          Ok(0) => break,
          Ok(_) => {
            match line.as_str() {
              "\n" => {}
              "data: [DONE]\n" => break,
              _ => {
                match serde_json::from_str::<Data>(&line[6..]) {
                  Ok(mut data) => {
                    if let Some(content) = data.choices.swap_remove(0).delta.content {
                      drop(tx.send_data(content.into()).await)
                    }
                  }
                  Err(err) => error!("failed to deserialize data line: {err}"),
                }
              }
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

    Ok((None, rx))
  }
}

#[derive(Deserialize)]
struct Data {
  choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
  delta: Delta,
}

#[derive(Deserialize)]
struct Delta {
  content: Option<String>,
}
