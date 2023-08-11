#[cfg(feature = "ssr")]
mod provider;
#[cfg(feature = "ssr")]
mod routes;
#[cfg(feature = "ssr")]
mod util;

#[cfg(feature = "hydration")]
fn main() {
  wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
  yew::Renderer::<libregpt::App>::new().hydrate();
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
  use std::{env, fs};
  use std::sync::Arc;
  use axum::{Router, routing, Server};
  use axum::handler::HandlerWithoutStateExt;
  use tower::ServiceBuilder;
  use tower_http::compression::CompressionLayer;
  use tower_http::services::ServeDir;
  use tracing::{error, info};

  tracing_subscriber::fmt::init();

  let port = match env::var("PORT").map_or(Ok(80), |p| p.parse()) {
    Ok(port) => port,
    Err(err) => return error!("invalid port: {err}"),
  };

  let index_html = fs::read_to_string("dist/index.html").expect("failed to read index.html");
  let (index_html_before, index_html_after) = index_html.split_once("<body>").unwrap();

  let mut index_html_before = index_html_before.to_owned();
  index_html_before.push_str("<body>");

  let serve_dist_dir = ServiceBuilder::new().layer(CompressionLayer::new()).service(
    ServeDir::new("dist")
      .append_index_html_on_directories(false)
      .not_found_service(routes::default.into_service()),
  );

  let render = routing::get(routes::render)
    .with_state((index_html_before, index_html_after.to_owned()));

  let ask = routing::get(routes::ask)
    .with_state(Arc::new(provider::s()));

  let router = Router::new()
    .route("/", render)
    .nest_service("/pkg", serve_dist_dir)
    .route("/api/ask", ask)
    .fallback(routes::default);

  let addr = ([0, 0, 0, 0], port).into();
  let server = Server::bind(&addr).serve(router.into_make_service());

  info!("listening on {addr}");

  if let Err(err) = server.await {
    error!("server died: {err}");
  }
}
