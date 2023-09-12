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

pub async fn render(
  State((index_html_before, index_html_after)): State<(String, String)>,
) -> impl IntoResponse {
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
