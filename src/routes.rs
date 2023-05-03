use std::sync::Arc;

use hyper::{Body, header, Request, Response, StatusCode};
use tracing::error;
use url::form_urlencoded;

use crate::provider;

pub fn root() -> Response<Body> {
  Response::builder()
    .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
    .body(Body::from(include_str!("../static/index.html")))
    .unwrap()
}

pub async fn ask(providers: Arc<provider::Map>, req: Request<Body>) -> Response<Body> {
  let Some(query) = req.uri().query() else {
    return bad_request("empty query");
  };

  let mut query = form_urlencoded::parse(query.as_bytes());
  let mut provider_name = None;
  let mut parent_msg_id = None;
  let mut prompt = None;

  while let Some((key, value)) = query.next() {
    match key.as_ref() {
      "provider" => provider_name = Some(value),
      "pmid" => parent_msg_id = Some(value),
      "prompt" => prompt = Some(value),
      _ => {}
    }
  }

  let Some(provider_name) = provider_name else {
    return bad_request("missing provider param");
  };

  let Some(prompt) = prompt else {
    return bad_request("missing prompt param");
  };

  let Some(provider) = providers.get(provider_name.as_ref()) else {
    return bad_request("invalid provider param");
  };

  match provider.ask(&prompt, parent_msg_id).await {
    Ok((msg_id, body)) => Response::builder()
      .header(header::CONTENT_TYPE, "application/octet-stream")
      .header("msg-id", msg_id)
      .body(body)
      .unwrap(),
    Err(err) => {
      error!("failed to ask to provider {provider_name}: {err}");
      Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("unexpected error"))
        .unwrap()
    }
  }
}

pub fn default() -> Response<Body> {
  Response::builder()
    .status(StatusCode::NOT_FOUND)
    .body(Body::from("nothing to see here"))
    .unwrap()
}

fn bad_request(body: &'static str) -> Response<Body> {
  Response::builder()
    .status(StatusCode::BAD_REQUEST)
    .body(Body::from(body))
    .unwrap()
}
