use std::borrow::Cow;

use async_trait::async_trait;
use boring::ssl::{SslConnector, SslMethod, SslVersion};
use hyper::client::HttpConnector;
use hyper::{header, Body, Client, Method, Request};
use hyper_boring::HttpsConnector;
use serde::Deserialize;
use tokio::io::AsyncBufReadExt;
use tokio::task;
use tokio_util::io::StreamReader;
use tracing::error;
use url::Url;
use uuid::Uuid;

use crate::util::BodyStream;

const CONNECTOR_CIPHER_LIST: &[&str] = &[
  "TLS_AES_128_GCM_SHA256",
  "TLS_AES_256_GCM_SHA384",
  "TLS_CHACHA20_POLY1305_SHA256",
  "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
  "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
  "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
  "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
  "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305_SHA256",
  "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305_SHA256",
  "TLS_ECDHE_RSA_WITH_AES_128_CBC_SHA",
  "TLS_ECDHE_RSA_WITH_AES_256_CBC_SHA",
  "TLS_RSA_WITH_AES_128_GCM_SHA256",
  "TLS_RSA_WITH_AES_256_GCM_SHA384",
  "TLS_RSA_WITH_AES_128_CBC_SHA",
  "TLS_RSA_WITH_AES_256_CBC_SHA",
];

const CONNECTOR_SIGNATURE_ALGORITHMS: &[&str] = &[
  "ecdsa_secp256r1_sha256",
  "rsa_pss_rsae_sha256",
  "rsa_pkcs1_sha256",
  "ecdsa_secp384r1_sha384",
  "rsa_pss_rsae_sha384",
  "rsa_pkcs1_sha384",
  "rsa_pss_rsae_sha512",
  "rsa_pkcs1_sha512",
];

#[derive(Deserialize)]
struct Data {
  #[serde(rename = "youChatToken")]
  token: String,
}

pub struct Provider {
  client: Client<HttpsConnector<HttpConnector>>,
}

impl Provider {
  pub fn new() -> Self {
    let mut connector = HttpConnector::new();
    connector.enforce_http(false);

    // https://github.com/4JX/reqwest-impersonate/blob/fa96a507f4163ee8875db38129e363384105b0d0/src/browser/chrome/ver/v108.rs

    let mut builder = SslConnector::builder(SslMethod::tls()).unwrap();
    builder.enable_ocsp_stapling();
    builder.enable_signed_cert_timestamps();
    builder.set_alpn_protos(b"\x02h2\x08http/1.1").unwrap();
    builder
      .set_cipher_list(&CONNECTOR_CIPHER_LIST.join(":"))
      .unwrap();
    builder.set_grease_enabled(true);
    builder
      .set_max_proto_version(Some(SslVersion::TLS1_3))
      .unwrap();
    builder
      .set_min_proto_version(Some(SslVersion::TLS1_2))
      .unwrap();
    builder
      .set_sigalgs_list(&CONNECTOR_SIGNATURE_ALGORITHMS.join(":"))
      .unwrap();

    let connector = HttpsConnector::with_connector(connector, builder).unwrap();
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
    let mut url = Url::parse("https://you.com/api/streamingSearch").unwrap();
    let (chat_id, chat) = state
      .and_then(|state| {
        if state.len() < 38 {
          None
        } else {
          Some((Cow::Borrowed(&state[..36]), &state[36..]))
        }
      })
      .unwrap_or_else(|| (Uuid::new_v4().to_string().into(), "[]"));

    {
      let mut query = url.query_pairs_mut();
      query.append_pair("q", prompt);
      query.append_pair("page", "1");
      query.append_pair("count", "10");
      query.append_pair("safeSearch", "Moderate");
      query.append_pair("onShoppingPage", "false");
      query.append_pair("mkt", "");
      query.append_pair(
        "responseFilter",
        "WebPages,Translations,TimeZone,Computation,RelatedSearches",
      );
      query.append_pair("queryTraceId", &chat_id);
      query.append_pair("domain", "youchat");
      query.append_pair("chat", chat);
      query.append_pair("chatId", &chat_id);
    }

    let req = Request::builder()
      .method(Method::GET)
      .uri(url.as_str())
      .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/108.0.0.0 Safari/537.36")
      .header(header::ACCEPT, "text/event-stream")
      .header("referer", "https://you.com/search?q=who+are+you&tbm=youchat")
      .header("sec-ch-ua", r#""Not_A Brand";v="8", "Chromium";v="108", "Google Chrome";v="108""#)
      .header("sec-ch-ua-mobile", "?0")
      .header("sec-ch-ua-platform", r#""Windows""#)
      .header("sec-fetch-dest", "document")
      .header("sec-fetch-mode", "navigate")
      .header("sec-fetch-site", "none")
      .header("sec-fetch-user", "?1")
      .header(header::COOKIE, format!("safesearch_guest=Moderate; uuid_guest={}", Uuid::new_v4().to_string()))
      .body(Body::empty())?;

    let res = self.client.request(req).await?;
    let (mut tx, rx) = Body::channel();

    task::spawn(async move {
      let mut reader = StreamReader::new(BodyStream::from(res.into_body()));
      let mut line = String::with_capacity(1 << 14);

      loop {
        match reader.read_line(&mut line).await {
          Ok(0) => break,
          Ok(_) => {
            if line.starts_with(r#"data: {"youChatToken"#) {
              match serde_json::from_str::<Data>(&line[6..]) {
                Ok(data) => drop(tx.send_data(data.token.into()).await),
                Err(err) => error!("failed to deserialize data line: {err}"),
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

    Ok((Some(chat_id.into_owned()), rx))
  }
}
