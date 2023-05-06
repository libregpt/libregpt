mod provider;
mod routes;

use std::convert::Infallible;
use std::env;
use std::sync::Arc;

use hyper::{service, Body, Request, Response, Server};
use tracing::{error, info};

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let port = match env::var("PORT").map_or(Ok(80), |p| p.parse()) {
    Ok(port) => port,
    Err(err) => return error!("invalid port: {err}"),
  };

  let addr = ([0, 0, 0, 0], port).into();
  let providers = Arc::new(provider::s());
  let make_service = service::make_service_fn(|_| {
    let providers = providers.clone();
    async move {
      Ok::<_, Infallible>(service::service_fn(move |req| {
        let providers = providers.clone();
        async move { Ok::<_, Infallible>(handle_request(providers, req).await) }
      }))
    }
  });

  let server = Server::bind(&addr).serve(make_service);

  info!("listening on {addr}");

  if let Err(err) = server.await {
    error!("server died: {err}");
  }
}

async fn handle_request(providers: Arc<provider::Map>, req: Request<Body>) -> Response<Body> {
  match req.uri().path() {
    "/" => routes::root(),
    "/api/ask" => routes::ask(providers, req).await,
    "/index.css" => Response::new(Body::from(include_str!("../static/index.css"))),
    "/index.js" => Response::new(Body::from(include_str!("../static/index.js"))),
    _ => routes::default(),
  }
}
