use std::borrow::Cow;

use async_trait::async_trait;
use hyper::client::HttpConnector;
use hyper::{header, Body, Client, Method, Request};
use hyper_rustls::HttpsConnector;
use rand::Rng;
use rand_user_agent::UserAgent;

use crate::util::new_rustls_connector;

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
    state: Option<Cow<'a, str>>,
  ) -> anyhow::Result<(Option<String>, Body)> {
    let user_agent = UserAgent::random().to_string();
    let api_key = generate_api_key(&user_agent);
    let boundary = String::from_iter(
      rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .map(|b| b as char)
        .take(6),
    );

    let mut content_type = String::with_capacity(30 + boundary.len());
    content_type.push_str("multipart/form-data; boundary=");
    content_type.push_str(&boundary);

    let chat_len = state.as_ref().map_or(2, |chat| chat.len() + 1) + 28 + prompt.len();
    let mut body = String::with_capacity(2 + boundary.len() * 3 + 61 + 54 + chat_len + 4 + 4);

    body.push_str("--");
    body.push_str(&boundary);
    body.push_str("Content-Disposition: form-data; name=\"chat_style\"\r\n\r\nchat\r\n--");
    body.push_str(&boundary);
    body.push_str("Content-Disposition: form-data; name=\"chatHistory\"\r\n\r\n");

    if let Some(chat) = state {
      body.push_str(&chat[..chat.len() - 1]);
      body.push(',');
    } else {
      body.push('[');
    };

    body.push_str("{\"role\":\"user\",\"content\":\"");
    body.push_str(prompt);
    body.push_str("\"}]\r\n--");
    body.push_str(&boundary);
    body.push_str("--\r\n");

    let req = Request::builder()
      .method(Method::POST)
      .uri("https://api.deepai.org/chat_response")
      .header(header::USER_AGENT, user_agent)
      .header("api-key", api_key)
      .header(header::CONTENT_TYPE, content_type)
      .body(Body::from(body))?;

    let res = self.client.request(req).await?;

    Ok((None, res.into_body()))
  }
}

fn md5hex<T: AsRef<[u8]>>(data: T) -> String {
  hex::encode(md5::compute(data).0)
}

fn generate_api_key(user_agent: &str) -> String {
  let mut n_buf = itoa::Buffer::new();
  let n = n_buf.format(rand::thread_rng().gen_range(0..u64::pow(10, 11)));
  let mut buf = String::with_capacity(user_agent.len() + 32);

  buf.push_str(user_agent);
  buf.push_str(n);
  buf.push('x');

  let mut digest = md5hex(&buf);

  buf.truncate(user_agent.len());
  buf.extend(digest.chars().rev());

  digest = md5hex(&buf);

  buf.truncate(user_agent.len());
  buf.extend(digest.chars().rev());

  digest = md5hex(&buf);

  let mut api_key = String::with_capacity(6 + n.len() + 1 + 32);
  api_key.push_str("tryit-");
  api_key.push_str(n);
  api_key.push('-');
  api_key.extend(digest.chars().rev());

  api_key
}
