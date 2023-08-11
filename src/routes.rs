use std::convert::Infallible;
use std::sync::Arc;
use axum::body::StreamBody;
use axum::extract::{Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use futures::stream::{self, StreamExt};
use hyper::Body;
use serde::Deserialize;
use tracing::error;
use yew::ServerRenderer;
use crate::provider;

pub async fn render(State((index_html_before, index_html_after)): State<(String, String)>) -> impl IntoResponse {
  let renderer = ServerRenderer::<libregpt::App>::new();

  StreamBody::new(
    stream::once(async move { index_html_before })
      .chain(renderer.render_stream())
      .chain(stream::once(async move { index_html_after }))
      .map(Result::<_, Infallible>::Ok),
  )
}

pub async fn default() -> (StatusCode, &'static str) {
  (StatusCode::NOT_FOUND, "nothing to see here")
}

#[derive(Deserialize)]
pub struct AskParams {
  provider: Box<str>,
  prompt: Box<str>,
  state: Option<Box<str>>,
}

pub async fn ask(
  State(providers): State<Arc<provider::Map>>,
  Query(params): Query<AskParams>,
) -> Response<Body> {
  let Some(provider) = providers.get(params.provider.as_ref()) else {
    return Response::builder()
      .status(StatusCode::BAD_REQUEST)
      .body(Body::from("invalid provider param"))
      .unwrap();
  };

  match provider.ask(&params.prompt, params.state.as_deref()).await {
    Ok((msg_id, body)) => {
      let mut builder =
        Response::builder().header(header::CONTENT_TYPE, "application/octet-stream");

      if let Some(msg_id) = msg_id {
        builder = builder.header("msg-id", msg_id);
      }

      builder.body(body).unwrap()
    }
    Err(err) => {
      error!("failed to ask to provider {}: {err}", params.provider);
      Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("unexpected error"))
        .unwrap()
    }
  }
}

/*pub async fn ask(providers: Arc<provider::Map>, req: Request<Body>) -> Response<BoxBody> {
  let Some(query) = req.uri().query() else {
    return bad_request("empty query");
  };

  let mut query = form_urlencoded::parse(query.as_bytes());
  let mut provider_name = None;
  let mut prompt = None;
  let mut state = None;

  while let Some((key, value)) = query.next() {
    match key.as_ref() {
      "provider" => provider_name = Some(value),
      "prompt" => prompt = Some(value),
      "state" => state = Some(value),
      _ => {}
    }
  }

  let Some(provider_name) = provider_name else {
    return bad_request("missing provider param");
  };

  let Some(prompt) = prompt.as_ref().map(|p| p.trim()) else {
    return bad_request("missing prompt param");
  };

  if prompt.is_empty() {
    return bad_request("prompt param is empty");
  };

  let Some(provider) = providers.get(provider_name.as_ref()) else {
    return bad_request("invalid provider param");
  };

  match provider.ask(prompt, state).await {
    Ok((msg_id, body)) => {
      let mut builder =
        Response::builder().header(header::CONTENT_TYPE, "application/octet-stream");

      if let Some(msg_id) = msg_id {
        builder = builder.header("msg-id", msg_id);
      }

      builder.body(body).unwrap()
    }
    Err(err) => {
      error!("failed to ask to provider {provider_name}: {err}");
      Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("unexpected error"))
        .unwrap()
    }
  }
}*/
